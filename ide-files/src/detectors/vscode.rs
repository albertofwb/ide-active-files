use crate::detector::{DetectionResult, IDEDetector};
use crate::types::{FileInfo, ProcessInfo, SupportedIDE};
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
        // code /path/to/workspace
        // code /path/to/file1.txt /path/to/file2.txt
        // code --folder-uri file:///path/to/workspace
        // code --file-uri file:///path/to/file.txt

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

                if Path::new(&path).is_dir() {
                    workspace_path = Some(path);
                } else if Path::new(&path).exists() {
                    files.push(self.create_file_info(&path, false));
                }
            }
            
            i += 1;
        }

        // If we found a workspace, try to find recently opened files
        if let Some(ref workspace) = workspace_path {
            if let Ok(recent_files) = self.get_vscode_recent_files(workspace) {
                files.extend(recent_files);
            }
        }

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

    /// Try to get recently opened files from VSCode workspace state
    fn get_vscode_recent_files(&self, workspace_path: &str) -> Result<Vec<FileInfo>, std::io::Error> {
        let mut files = Vec::new();
        
        // Check .vscode/settings.json for any file references
        let vscode_dir = Path::new(workspace_path).join(".vscode");
        if vscode_dir.exists() {
            // Look for settings that might contain file references
            let settings_file = vscode_dir.join("settings.json");
            if settings_file.exists() {
                // For now, just mark the workspace as having unknown open files
                // In a full implementation, we'd parse the settings.json
                // and potentially use VSCode's API or extension
            }
        }

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

        for process in processes {
            if let Some(cmdline) = self.get_process_cmdline(process.pid) {
                if let Some((workspace, files)) = self.extract_vscode_info(&cmdline) {
                    if !workspace.is_empty() && project_path.is_none() {
                        project_path = Some(workspace);
                    }
                    
                    for file in files {
                        if file.is_active && active_file.is_none() {
                            active_file = Some(file.path.clone());
                        }
                        all_files.push(file);
                    }
                }
            }
        }

        if all_files.is_empty() && project_path.is_none() {
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