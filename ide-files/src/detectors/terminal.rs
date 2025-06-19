use crate::detector::{IDEDetector, DetectionResult};
use crate::types::{ProcessInfo, SupportedIDE, FileInfo};
use std::process::Command;
use std::collections::HashMap;

/// Terminal editor detector
pub struct TerminalEditorDetector {
    ide_type: SupportedIDE,
    process_names: Vec<&'static str>,
}

impl TerminalEditorDetector {
    pub fn new(ide_type: SupportedIDE) -> Self {
        let process_names = match ide_type {
            SupportedIDE::Vim => vec!["vim", "nvim", "gvim"],
            SupportedIDE::Nano => vec!["nano"],
            _ => vec![],
        };

        Self {
            ide_type,
            process_names,
        }
    }

    /// Get process command line arguments via /proc filesystem (Linux/macOS)
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn get_process_cmdline(&self, pid: u32) -> Option<Vec<String>> {
        #[cfg(target_os = "linux")]
        {
            let cmdline_path = format!("/proc/{}/cmdline", pid);
            std::fs::read_to_string(&cmdline_path)
                .ok()
                .map(|content| {
                    content.split('\0')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect()
                })
        }

        #[cfg(target_os = "macos")]
        {
            // macOS uses ps command to get process arguments
            let output = Command::new("ps")
                .args(&["-p", &pid.to_string(), "-o", "args="])
                .output()
                .ok()?;

            let cmdline = String::from_utf8_lossy(&output.stdout);
            Some(cmdline.trim().split_whitespace().map(|s| s.to_string()).collect())
        }
    }

    /// Windows process command line retrieval
    #[cfg(target_os = "windows")]
    fn get_process_cmdline(&self, pid: u32) -> Option<Vec<String>> {
        // Windows implementation - simplified version
        // Could use WMI or PowerShell for full command line
        let output = Command::new("wmic")
            .args(&["process", "where", &format!("ProcessId={}", pid), 
                   "get", "CommandLine", "/value"])
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

    fn extract_file_from_cmdline(&self, cmdline: &[String]) -> Option<FileInfo> {
        if cmdline.is_empty() {
            return None;
        }

        // Find file argument
        let file_path = match self.ide_type {
            SupportedIDE::Vim => {
                // vim format: vim /path/to/file.txt
                // nvim format: nvim /path/to/file.txt  
                // May have options: vim -n /path/to/file.txt
                cmdline.iter()
                    .skip(1) // Skip program name
                    .find(|arg| !arg.starts_with('-') && !arg.is_empty())
                    .cloned()
            }
            SupportedIDE::Nano => {
                // nano format: nano /path/to/file.txt
                // May have options: nano -w /path/to/file.txt
                cmdline.iter()
                    .skip(1) // Skip program name
                    .find(|arg| !arg.starts_with('-') && !arg.is_empty())
                    .cloned()
            }
            _ => None,
        }?;

        // Convert to absolute path
        let absolute_path = if file_path.starts_with('/') {
            file_path
        } else {
            // Relative path, try to get current working directory
            std::env::current_dir()
                .ok()
                .and_then(|cwd| cwd.join(&file_path).to_str().map(|s| s.to_string()))
                .unwrap_or(file_path)
        };

        let file_name = std::path::Path::new(&absolute_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&absolute_path)
            .to_string();

        Some(FileInfo {
            path: absolute_path,
            name: file_name,
            is_active: true, // Terminal editors usually edit one file
            is_modified: false, // Can't easily detect modification status
            tab_index: Some(0),
            project_name: None,
        })
    }

    /// Check if file exists
    fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

impl IDEDetector for TerminalEditorDetector {
    fn ide_type(&self) -> SupportedIDE {
        self.ide_type
    }

    fn is_target_process(&self, process: &ProcessInfo) -> bool {
        self.process_names.iter().any(|&name| {
            let process_name = process.name.to_lowercase();
            process_name == name || process_name.starts_with(name)
        })
    }

    fn extract_files(&self, processes: &[ProcessInfo]) -> DetectionResult<crate::types::DetectionResult> {
        let mut open_files = Vec::new();
        let mut active_file = None;

        for process in processes {
            if let Some(cmdline) = self.get_process_cmdline(process.pid) {
                if let Some(file_info) = self.extract_file_from_cmdline(&cmdline) {
                    // Verify file actually exists
                    if self.file_exists(&file_info.path) {
                        if file_info.is_active {
                            active_file = Some(file_info.path.clone());
                        }
                        open_files.push(file_info);
                    }
                }
            }
        }

        if open_files.is_empty() {
            return Err(crate::detector::DetectionError::WindowParseError {
                message: format!("No valid files found in {} processes", self.display_name()),
            });
        }

        Ok(crate::types::DetectionResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            ide_name: self.display_name().to_string(),
            ide_version: None,
            active_file,
            open_files,
            project_path: None,
        })
    }
}