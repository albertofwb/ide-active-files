use crate::detector::{DetectionResult, IDEDetector};
use crate::types::{FileInfo, ProcessInfo, SupportedIDE};
use rusqlite::{Connection, Result as SqliteResult};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;

/// Visual Studio Code detector
pub struct VSCodeDetector {
    process_names: Vec<&'static str>,
}

impl VSCodeDetector {
    pub fn new() -> Self {
        Self {
            process_names: vec![
                "code", 
                "code-oss", 
                "codium", 
                "code-insiders",
                "Code",
                "Code.exe",
                "code.exe"
            ],
        }
    }

    /// Get process command line arguments to find workspace/files
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn get_process_cmdline(&self, pid: u32) -> Option<Vec<String>> {
        #[cfg(target_os = "linux")]
        {
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            std::fs::read_to_string(&cmdline_path).ok().map(|content| {
                content
                    .split('\0')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect()
            })
        }

        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("ps")
                .args(&["-p", &pid.to_string(), "-o", "args="])
                .output()
                .ok()?;

            let cmdline = String::from_utf8_lossy(&output.stdout);
            Some(
                cmdline
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect(),
            )
        }
    }

    /// Windows process command line retrieval
    #[cfg(target_os = "windows")]
    fn get_process_cmdline(&self, pid: u32) -> Option<Vec<String>> {
        let output = std::process::Command::new("wmic")
            .args(&[
                "process",
                "where",
                &format!("ProcessId={}", pid),
                "get",
                "CommandLine",
                "/value",
            ])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.starts_with("CommandLine=") {
                let cmdline = line.trim_start_matches("CommandLine=");
                return Some(shell_words::split(cmdline).unwrap_or_default());
            }
        }

        None
    }

    /// Extract workspace and opened files from command line
    fn extract_vscode_info(&self, cmdline: &[String]) -> Option<(String, Vec<FileInfo>)> {
        if cmdline.is_empty() {
            return None;
        }

        let mut workspace_path = None;
        let mut files = Vec::new();
        
        // Parse VSCode command line arguments
        // Common formats:
        // 1. Folder/Workspace mode:
        //    code /path/to/workspace
        //    code --folder-uri file:///path/to/workspace
        // 2. Direct file mode:
        //    code /path/to/file1.txt /path/to/file2.txt
        //    code --file-uri file:///path/to/file.txt

        let mut i = 1; // Skip executable name
        while i < cmdline.len() {
            let arg = &cmdline[i];
            
            if arg.starts_with("--folder-uri") || arg.starts_with("--file-uri") {
                // Handle URI format
                if arg.contains('=') {
                    let uri = arg.split('=').nth(1).unwrap_or("");
                    if let Some(path) = self.decode_vscode_uri(uri) {
                        if arg.starts_with("--folder-uri") {
                            workspace_path = Some(path);
                        } else {
                            files.push(self.create_file_info(&path, false));
                        }
                    }
                } else if i + 1 < cmdline.len() {
                    // URI in next argument
                    i += 1;
                    if let Some(path) = self.decode_vscode_uri(&cmdline[i]) {
                        if arg.starts_with("--folder-uri") {
                            workspace_path = Some(path);
                        } else {
                            files.push(self.create_file_info(&path, false));
                        }
                    }
                }
            } else if !arg.starts_with('-') && arg.contains('/') {
                // Regular file/directory path
                let path = if arg.starts_with("file://") {
                    arg.strip_prefix("file://").unwrap_or(arg).to_string()
                } else {
                    arg.clone()
                };

                // Skip VS Code extension and internal files
                if path.contains("/.vscode/extensions/") || 
                   path.contains("/resources/app/extensions/") ||
                   path.contains("/CachedExtension") ||
                   path.contains("node_modules") ||
                   path.ends_with(".js") && (path.contains("server") || path.contains("bundle")) {
                    i += 1;
                    continue;
                }

                if Path::new(&path).is_dir() {
                    workspace_path = Some(path);
                } else if Path::new(&path).exists() {
                    files.push(self.create_file_info(&path, false));
                }
            }
            
            i += 1;
        }

        // Don't add VSCode session files here since they'll be added in extract_files
        // This avoids duplicate file detection

        // If we have workspace or files, return them
        if workspace_path.is_some() || !files.is_empty() {
            Some((workspace_path.unwrap_or_default(), files))
        } else {
            None
        }
    }

    /// Decode VSCode URI (file:// format)
    fn decode_vscode_uri(&self, uri: &str) -> Option<String> {
        if uri.starts_with("file://") {
            Some(uri.strip_prefix("file://").unwrap_or(uri).to_string())
        } else {
            Some(uri.to_string())
        }
    }

    /// Try to get opened files from VSCode workspace state database
    fn get_vscode_recent_files(&self, workspace_path: &str) -> Result<(Vec<FileInfo>, Option<String>), std::io::Error> {
        // First try to get files from VSCode workspace database
        if let Ok((files, detected_workspace)) = self.get_vscode_session_files(workspace_path) {
            if !files.is_empty() {
                return Ok((files, detected_workspace));
            }
        }

        // Fallback to heuristic method
        self.get_vscode_files_heuristic(workspace_path).map(|files| (files, None))
    }

    /// Get VSCode session files from SQLite database
    fn get_vscode_session_files(&self, workspace_path: &str) -> Result<(Vec<FileInfo>, Option<String>), std::io::Error> {
        let home_dir = env::var("HOME").map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        
        // Find VSCode workspace storage directory
        let workspace_storage_dir = format!("{}/.config/Code/User/workspaceStorage", home_dir);
        
        // Try to find workspace ID, but if not found, try all workspace directories
        if let Ok(workspace_id) = self.get_workspace_id(workspace_path, &workspace_storage_dir) {
            let db_path = format!("{}/{}/state.vscdb", workspace_storage_dir, workspace_id);
            let workspace_json_path = format!("{}/{}/workspace.json", workspace_storage_dir, workspace_id);
            
            if Path::new(&db_path).exists() {
                let files = self.parse_vscode_database(&db_path)?;
                let detected_workspace = self.extract_workspace_from_json(&workspace_json_path);
                return Ok((files, detected_workspace));
            }
        }
        
        // Fallback: try all workspace directories (for non-workspace VSCode sessions)
        self.scan_all_vscode_sessions(&workspace_storage_dir)
    }

    /// Extract workspace path from workspace.json
    fn extract_workspace_from_json(&self, json_path: &str) -> Option<String> {
        if let Ok(content) = fs::read_to_string(json_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(folder) = json.get("folder").and_then(|v| v.as_str()) {
                    // Remove file:// prefix if present
                    return Some(folder.strip_prefix("file://").unwrap_or(folder).to_string());
                }
            }
        }
        None
    }

    /// Scan all VSCode workspace directories for editor sessions
    fn scan_all_vscode_sessions(&self, storage_dir: &str) -> Result<(Vec<FileInfo>, Option<String>), std::io::Error> {
        if let Ok(entries) = fs::read_dir(storage_dir) {
            // Get the most recently modified workspace (likely the active one)
            let mut workspace_dirs: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_dir())
                .collect();
                
            // Sort by modification time (newest first)
            workspace_dirs.sort_by(|a, b| {
                let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
                let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
                b_time.cmp(&a_time)
            });
            
            // Try the most recent workspaces
            for workspace_dir in workspace_dirs.into_iter().take(2) { // Try top 2 recent workspaces
                let db_path = workspace_dir.path().join("state.vscdb");
                let workspace_json_path = workspace_dir.path().join("workspace.json");
                
                if db_path.exists() {
                    if let Ok(files) = self.parse_vscode_database(&db_path.to_string_lossy()) {
                        if !files.is_empty() {
                            let detected_workspace = self.extract_workspace_from_json(&workspace_json_path.to_string_lossy());
                            return Ok((files, detected_workspace));
                        }
                    }
                }
            }
        }
        
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No active VSCode sessions found"))
    }

    /// Find workspace ID from VSCode storage directory
    fn get_workspace_id(&self, workspace_path: &str, storage_dir: &str) -> Result<String, std::io::Error> {
        let workspace_uri = format!("file://{}", workspace_path);
        
        // Look through workspace storage directories to find matching workspace
        if let Ok(entries) = fs::read_dir(storage_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let workspace_dir = entry.path();
                    if workspace_dir.is_dir() {
                        // Check workspace.json for matching URI
                        let workspace_json = workspace_dir.join("workspace.json");
                        if workspace_json.exists() {
                            if let Ok(content) = fs::read_to_string(&workspace_json) {
                                if content.contains(&workspace_uri) {
                                    if let Some(dir_name) = workspace_dir.file_name() {
                                        if let Some(name_str) = dir_name.to_str() {
                                            return Ok(name_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "VSCode workspace ID not found"))
    }

    /// Parse VSCode SQLite database for editor state
    fn parse_vscode_database(&self, db_path: &str) -> Result<Vec<FileInfo>, std::io::Error> {
        let conn = Connection::open(db_path)
            .map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;

        let mut stmt = conn.prepare("SELECT value FROM ItemTable WHERE key = 'memento/workbench.parts.editor'")
            .map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;

        let rows: SqliteResult<Vec<String>> = stmt.query_map([], |row| {
            Ok(row.get(0)?)
        }).and_then(|mapped_rows| mapped_rows.collect());

        match rows {
            Ok(values) if !values.is_empty() => {
                let editor_state: Value = serde_json::from_str(&values[0])
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                
                self.parse_editor_state(editor_state)
            }
            _ => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No editor state found in database"))
        }
    }

    /// Parse VSCode editor state JSON to extract open files
    fn parse_editor_state(&self, editor_state: Value) -> Result<Vec<FileInfo>, std::io::Error> {
        let mut files = Vec::new();
        
        // Navigate through the JSON structure
        if let Some(editorpart) = editor_state.get("editorpart.state") {
            if let Some(serialized_grid) = editorpart.get("serializedGrid") {
                if let Some(root) = serialized_grid.get("root") {
                    if let Some(data) = root.get("data") {
                        if let Some(data_array) = data.as_array() {
                            for group in data_array {
                                if let Some(group_data) = group.get("data") {
                                    if let Some(editors) = group_data.get("editors") {
                                        if let Some(mru) = group_data.get("mru") {
                                            files.extend(self.parse_editor_group(editors, mru)?);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Parse individual editor group
    fn parse_editor_group(&self, editors: &Value, mru: &Value) -> Result<Vec<FileInfo>, std::io::Error> {
        let mut files = Vec::new();
        
        if let (Some(editors_array), Some(mru_array)) = (editors.as_array(), mru.as_array()) {
            // Get active file index (first in MRU order)
            let active_index = mru_array.get(0).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            
            for (index, editor) in editors_array.iter().enumerate() {
                if let Some(value_str) = editor.get("value").and_then(|v| v.as_str()) {
                    if let Ok(editor_data) = serde_json::from_str::<Value>(value_str) {
                        if let Some(resource) = editor_data.get("resourceJSON") {
                            if let Some(fs_path) = resource.get("fsPath").and_then(|v| v.as_str()) {
                                let is_active = index == active_index;
                                files.push(self.create_file_info(fs_path, is_active));
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Fallback heuristic method for getting workspace files
    fn get_vscode_files_heuristic(&self, workspace_path: &str) -> Result<Vec<FileInfo>, std::io::Error> {
        let mut files = Vec::new();
        
        // Try to find common file types in the workspace (simplified heuristic)
        if let Ok(entries) = fs::read_dir(workspace_path) {
            let mut found_files = 0;
            for entry in entries.take(10) { // Limit to first 10 files
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && found_files < 5 {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            // Only include common development files
                            if name.ends_with(".js") || name.ends_with(".ts") || 
                               name.ends_with(".py") || name.ends_with(".rs") ||
                               name.ends_with(".go") || name.ends_with(".java") ||
                               name.ends_with(".cpp") || name.ends_with(".c") ||
                               name.ends_with(".json") || name.ends_with(".md") {
                                files.push(self.create_file_info(
                                    &path.to_string_lossy().to_string(),
                                    found_files == 0 // Mark first file as potentially active
                                ));
                                found_files += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Create a FileInfo struct from a path
    fn create_file_info(&self, path: &str, is_active: bool) -> FileInfo {
        let file_name = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_string();

        FileInfo {
            path: path.to_string(),
            name: file_name,
            is_active,
            is_modified: false, // Cannot easily detect without VSCode API
            tab_index: None,
            project_name: None,
        }
    }
}

impl IDEDetector for VSCodeDetector {
    fn ide_type(&self) -> SupportedIDE {
        SupportedIDE::VSCode
    }

    fn is_target_process(&self, process: &ProcessInfo) -> bool {
        self.process_names.iter().any(|&name| {
            let process_name = process.name.to_lowercase();
            let exe_path = process.executable_path.to_lowercase();
            
            process_name == name.to_lowercase() || 
            process_name.starts_with(&name.to_lowercase()) ||
            exe_path.contains(&name.to_lowercase())
        })
    }

    fn extract_files(&self, processes: &[ProcessInfo]) -> DetectionResult<crate::types::DetectionResult> {
        let mut all_files = Vec::new();
        let mut active_file = None;
        let mut project_path = None;
        let mut found_cmdline_files = false;

        // First, check command line arguments for workspace/files
        for process in processes {
            if let Some(cmdline) = self.get_process_cmdline(process.pid) {
                if let Some((workspace, files)) = self.extract_vscode_info(&cmdline) {
                    if !workspace.is_empty() && project_path.is_none() {
                        project_path = Some(workspace.clone());
                    }
                    
                    // If files were passed directly via command line
                    if !files.is_empty() {
                        found_cmdline_files = true;
                        for file in files {
                            if file.is_active && active_file.is_none() {
                                active_file = Some(file.path.clone());
                            }
                            all_files.push(file);
                        }
                    }
                }
            }
        }

        // If VSCode opened a folder (no files in cmdline), get files from session database
        // Also try session database if no cmdline files were found
        if !found_cmdline_files {
            if let Ok((session_files, detected_workspace)) = self.get_vscode_recent_files(project_path.as_deref().unwrap_or("")) {
                // Update project path if detected from workspace.json
                if project_path.is_none() && detected_workspace.is_some() {
                    project_path = detected_workspace;
                }
                
                for session_file in session_files {
                    // Avoid duplicates
                    if !all_files.iter().any(|f| f.path == session_file.path) {
                        if session_file.is_active && active_file.is_none() {
                            active_file = Some(session_file.path.clone());
                        }
                        all_files.push(session_file);
                    }
                }
            }
        }

        if all_files.is_empty() {
            return Err(crate::detector::DetectionError::WindowParseError {
                message: "No workspace or files detected for VSCode".to_string(),
            });
        }

        Ok(crate::types::DetectionResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            ide_name: self.display_name().to_string(),
            ide_version: None, // Could be extracted from process info
            active_file,
            open_files: all_files,
            project_path,
        })
    }
}