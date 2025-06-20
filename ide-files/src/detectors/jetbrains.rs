use crate::detector::{DetectionResult, IDEDetector};
use crate::types::{FileInfo, ProcessInfo, SupportedIDE};
use regex::Regex;
use std::fs;
use std::path::Path;

/// JetBrains IDE base detector
pub struct JetBrainsDetector {
    ide_type: SupportedIDE,
    process_names: Vec<&'static str>,
}

impl JetBrainsDetector {
    pub fn new(ide_type: SupportedIDE) -> Self {
        let process_names = match ide_type {
            SupportedIDE::GoLand => vec!["goland", "goland.exe", "goland64.exe"],
            SupportedIDE::PyCharm => vec!["pycharm", "pycharm.exe", "pycharm64.exe"],
            SupportedIDE::IntelliJIDEA => vec!["idea", "idea.exe", "idea64.exe"],
            SupportedIDE::WebStorm => vec!["webstorm", "webstorm.exe", "webstorm64.exe"],
            SupportedIDE::PhpStorm => vec!["phpstorm", "phpstorm.exe", "phpstorm64.exe"],
            SupportedIDE::RubyMine => vec!["rubymine", "rubymine.exe", "rubymine64.exe"],
            SupportedIDE::CLion => vec!["clion", "clion.exe", "clion64.exe"],
            _ => vec![],
        };

        Self {
            ide_type,
            process_names,
        }
    }

    fn parse_jetbrains_window_title(&self, title: &str) -> Option<(FileInfo, Option<String>)> {
        // JetBrains IDE window title formats:
        // "filename.ext - project-name [/path/to/project] - IDE-Name 202X.X"
        // "filename.ext* - project-name [/path/to/project] - IDE-Name 202X.X" (modified)
        // "project-name - IDE-Name 202X.X" (no file open)
        // "filename.ext - project-name - IDE-Name 202X.X" (no project path)

        let patterns = [
            // Full format with project path
            r"^([^-]+?)\s*(\*)?\s*-\s*([^[]+?)\s*\[([^\]]+)\]\s*-\s*(\w+(?:\s+\w+)*)\s+([\d.]+)",
            // Format without project path
            r"^([^-]+?)\s*(\*)?\s*-\s*([^-]+?)\s*-\s*(\w+(?:\s+\w+)*)\s+([\d.]+)",
            // Simple format: "project – filename.ext" (PyCharm 2025.1)
            r"^([^–]+?)\s*–\s*(.+)$",
            // Project only (no file)
            r"^([^-]+?)\s*-\s*(\w+(?:\s+\w+)*)\s+([\d.]+)",
        ];

        for (i, pattern) in patterns.iter().enumerate() {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(title) {
                    match i {
                        0 => {
                            // Full format with project path
                            let filename = captures.get(1)?.as_str().trim();
                            let is_modified = captures.get(2).is_some();
                            let project_name = captures.get(3)?.as_str().trim();
                            let project_path = captures.get(4).map(|m| m.as_str().trim().to_string());
                            
                            if !filename.is_empty() && !filename.eq_ignore_ascii_case(project_name) {
                                let full_path = if let Some(ref path) = project_path {
                                    format!("{}/{}", path.trim_end_matches('/'), filename)
                                } else {
                                    filename.to_string()
                                };

                                return Some((FileInfo {
                                    path: full_path,
                                    name: filename.to_string(),
                                    is_active: true,
                                    is_modified,
                                    tab_index: None,
                                    project_name: Some(project_name.to_string()),
                                }, project_path));
                            }
                        }
                        1 => {
                            // Format without project path - try to find project directory
                            let filename = captures.get(1)?.as_str().trim();
                            let is_modified = captures.get(2).is_some();
                            let project_name = captures.get(3)?.as_str().trim();
                            
                            if !filename.is_empty() && !filename.eq_ignore_ascii_case(project_name) {
                                // Try to find project path
                                let project_path = self.find_project_path(project_name);
                                let full_path = if let Some(ref path) = project_path {
                                    format!("{}/{}", path.trim_end_matches('/'), filename)
                                } else {
                                    filename.to_string()
                                };
                                
                                return Some((FileInfo {
                                    path: full_path,
                                    name: filename.to_string(),
                                    is_active: true,
                                    is_modified,
                                    tab_index: None,
                                    project_name: Some(project_name.to_string()),
                                }, project_path));
                            }
                        }
                        2 => {
                            // Simple format: "project – filename.ext"
                            let project_name = captures.get(1)?.as_str().trim();
                            let filename = captures.get(2)?.as_str().trim();
                            
                            if !filename.is_empty() {
                                // Try to find project path
                                let project_path = self.find_project_path(project_name);
                                let full_path = if let Some(ref path) = project_path {
                                    format!("{}/{}", path.trim_end_matches('/'), filename)
                                } else {
                                    filename.to_string()
                                };
                                
                                return Some((FileInfo {
                                    path: full_path,
                                    name: filename.to_string(),
                                    is_active: true,
                                    is_modified: false,
                                    tab_index: None,
                                    project_name: Some(project_name.to_string()),
                                }, project_path));
                            }
                        }
                        3 => {
                            // Project only - no specific file detected
                            return None;
                        }
                        _ => {}
                    }
                }
            }
        }

        None
    }

    /// Get process command line to find additional information
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

    /// Extract project path from command line arguments
    fn extract_project_from_cmdline(&self, cmdline: &[String]) -> Option<String> {
        if cmdline.is_empty() {
            return None;
        }

        // Look for project directory argument
        // JetBrains IDEs often launched with: goland /path/to/project
        for arg in cmdline.iter().skip(1) { // Skip executable name
            if !arg.starts_with('-') && Path::new(arg).is_dir() {
                return Some(arg.clone());
            }
        }

        None
    }

    /// Try to find project path by searching for .idea directories
    fn find_project_path(&self, project_name: &str) -> Option<String> {
        // Common locations to search for projects
        let search_paths = vec![
            format!("/home/{}/codes", std::env::var("USER").unwrap_or_default()),
            format!("/home/{}/projects", std::env::var("USER").unwrap_or_default()),
            format!("/home/{}/workspace", std::env::var("USER").unwrap_or_default()),
            format!("/home/{}/dev", std::env::var("USER").unwrap_or_default()),
            format!("/home/{}/Documents", std::env::var("USER").unwrap_or_default()),
            format!("/home/{}/Dropbox/dev", std::env::var("USER").unwrap_or_default()),
            format!("/home/{}", std::env::var("USER").unwrap_or_default()),
        ];

        // First, try exact match with project name
        for base_path in &search_paths {
            let potential_path = format!("{}/{}", base_path, project_name);
            let idea_path = Path::new(&potential_path).join(".idea");
            if idea_path.exists() && idea_path.is_dir() {
                return Some(potential_path);
            }
        }

        // If not found, search recursively (limited depth)
        for base_path in &search_paths {
            if let Ok(path) = self.find_project_in_directory(Path::new(base_path), project_name, 3) {
                return Some(path);
            }
        }

        None
    }

    /// Recursively search for project directory with .idea folder
    fn find_project_in_directory(&self, base: &Path, project_name: &str, max_depth: u32) -> Result<String, std::io::Error> {
        if max_depth == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Max depth reached"));
        }

        if !base.exists() || !base.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Base path not found"));
        }

        // Check if current directory matches
        if let Some(dir_name) = base.file_name().and_then(|n| n.to_str()) {
            if dir_name.eq_ignore_ascii_case(project_name) {
                let idea_path = base.join(".idea");
                if idea_path.exists() && idea_path.is_dir() {
                    return Ok(base.to_string_lossy().to_string());
                }
            }
        }

        // Search subdirectories
        for entry in fs::read_dir(base)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden directories and common non-project directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') && 
                           !name.eq_ignore_ascii_case("node_modules") &&
                           !name.eq_ignore_ascii_case("target") &&
                           !name.eq_ignore_ascii_case("build") &&
                           !name.eq_ignore_ascii_case("dist") {
                            if let Ok(found) = self.find_project_in_directory(&path, project_name, max_depth - 1) {
                                return Ok(found);
                            }
                        }
                    }
                }
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Project not found"))
    }

    /// Try to find opened files in JetBrains workspace
    fn get_jetbrains_recent_files(&self, project_path: &str) -> Result<Vec<FileInfo>, std::io::Error> {
        let mut files = Vec::new();
        
        // JetBrains stores file information in .idea directory
        let idea_dir = Path::new(project_path).join(".idea");
        if !idea_dir.exists() {
            return Ok(files);
        }

        // Try both workspace.xml and workspace_with_tabs.xml
        let workspace_files = vec![
            idea_dir.join("workspace.xml"),
            idea_dir.join("workspace_with_tabs.xml"),
        ];

        for workspace_file in workspace_files {
            if workspace_file.exists() {
                if let Ok(content) = fs::read_to_string(&workspace_file) {
                    // Parse FileEditorManager component for open tabs
                    if let Some(editor_manager_start) = content.find("<component name=\"FileEditorManager\">") {
                        if let Some(editor_manager_end) = content[editor_manager_start..].find("</component>") {
                            let editor_section = &content[editor_manager_start..editor_manager_start + editor_manager_end];
                            
                            // Regex to find file entries with tab status
                            if let Ok(regex) = Regex::new(r#"<file[^>]*current-in-tab="([^"]*)"[^>]*>\s*<entry file="file://\$PROJECT_DIR\$([^"]+)""#) {
                                for cap in regex.captures_iter(editor_section) {
                                    if let (Some(is_current), Some(path_match)) = (cap.get(1), cap.get(2)) {
                                        let relative_path = path_match.as_str();
                                        let full_path = format!("{}{}", project_path, relative_path);
                                        let is_active = is_current.as_str() == "true";
                                        
                                        if Path::new(&full_path).exists() {
                                            let file_name = Path::new(relative_path)
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or(relative_path)
                                                .to_string();

                                            files.push(FileInfo {
                                                path: full_path,
                                                name: file_name,
                                                is_active,
                                                is_modified: false,
                                                tab_index: None,
                                                project_name: None,
                                            });
                                        }
                                    }
                                }
                            }
                            
                            // If we found files in this workspace file, return early
                            if !files.is_empty() {
                                break;
                            }
                        }
                    }
                    
                    // Fallback: Simple regex to find file paths in XML (for older formats)
                    if files.is_empty() {
                        if let Ok(regex) = Regex::new(r#"file://\$PROJECT_DIR\$([^"]+)"#) {
                            for cap in regex.captures_iter(&content) {
                                if let Some(path_match) = cap.get(1) {
                                    let relative_path = path_match.as_str();
                                    let full_path = format!("{}{}", project_path, relative_path);
                                    
                                    if Path::new(&full_path).exists() {
                                        let file_name = Path::new(relative_path)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or(relative_path)
                                            .to_string();

                                        files.push(FileInfo {
                                            path: full_path,
                                            name: file_name,
                                            is_active: false,
                                            is_modified: false,
                                            tab_index: None,
                                            project_name: None,
                                        });

                                        if files.len() >= 10 { // Limit number of files
                                            break;
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
}

impl IDEDetector for JetBrainsDetector {
    fn ide_type(&self) -> SupportedIDE {
        self.ide_type
    }

    fn is_target_process(&self, process: &ProcessInfo) -> bool {
        self.process_names
            .iter()
            .any(|&name| process.name.to_lowercase().contains(&name.to_lowercase()))
    }

    fn extract_files(
        &self,
        processes: &[ProcessInfo],
    ) -> DetectionResult<crate::types::DetectionResult> {
        let mut open_files = Vec::new();
        let mut active_file = None;
        let mut project_path = None;
        let ide_version = None;

        for process in processes {
            // Try to extract info from window title
            if let Some((file_info, extracted_project_path)) = self.parse_jetbrains_window_title(&process.window_title) {
                if file_info.is_active {
                    active_file = Some(file_info.path.clone());
                }

                if let Some(path) = extracted_project_path {
                    project_path = Some(path);
                }

                open_files.push(file_info);
            }

            // Also try to extract project path from command line
            if project_path.is_none() {
                if let Some(cmdline) = self.get_process_cmdline(process.pid) {
                    if let Some(cmd_project_path) = self.extract_project_from_cmdline(&cmdline) {
                        project_path = Some(cmd_project_path);
                    }
                }
            }
        }

        // If we found a project path, try to get opened files from workspace
        if let Some(ref proj_path) = project_path {
            if let Ok(workspace_files) = self.get_jetbrains_recent_files(proj_path) {
                if !workspace_files.is_empty() {
                    // Replace window title detection with workspace file info
                    open_files.clear();
                    active_file = None;
                    
                    for workspace_file in workspace_files {
                        if workspace_file.is_active {
                            active_file = Some(workspace_file.path.clone());
                        }
                        open_files.push(workspace_file);
                    }
                } else if open_files.is_empty() || open_files.len() == 1 {
                    // Fallback to old behavior for older IDE versions
                    if let Ok(recent_files) = self.get_jetbrains_recent_files(proj_path) {
                        for recent_file in recent_files {
                            // Avoid duplicates
                            if !open_files.iter().any(|f| f.path == recent_file.path) {
                                open_files.push(recent_file);
                            }
                        }
                    }
                }
            }
        }

        if open_files.is_empty() {
            return Err(crate::detector::DetectionError::WindowParseError {
                message: format!("No files detected for {}", self.display_name()),
            });
        }

        Ok(crate::types::DetectionResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            ide_name: self.display_name().to_string(),
            ide_version,
            active_file,
            open_files,
            project_path,
        })
    }
}
