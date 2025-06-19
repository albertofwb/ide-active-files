use crate::detector::{IDEDetector, DetectionResult};
use crate::types::{ProcessInfo, SupportedIDE, FileInfo};
use regex::Regex;

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

    fn parse_jetbrains_window_title(&self, title: &str) -> Option<FileInfo> {
        // JetBrains IDE window title formats:
        // "filename.ext - project-name [/path/to/project] - IDE-Name 202X.X"
        // "filename.ext* - project-name [/path/to/project] - IDE-Name 202X.X" (modified)
        
        let patterns = [
            r"^([^-]+?)\s*(\*)?\s*-\s*([^[]+?)\s*\[([^\]]+)\]\s*-\s*\w+\s+[\d.]+",
            r"^([^-]+?)\s*(\*)?\s*-\s*([^-]+?)\s*-\s*\w+\s+[\d.]+",
        ];

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(title) {
                    let filename = captures.get(1)?.as_str().trim();
                    let is_modified = captures.get(2).is_some();
                    let project_name = captures.get(3)?.as_str().trim();
                    let project_path = captures.get(4).map(|m| m.as_str().trim());

                    if !filename.is_empty() {
                        let full_path = if let Some(path) = project_path {
                            format!("{}/{}", path.trim_end_matches('/'), filename)
                        } else {
                            filename.to_string()
                        };

                        return Some(FileInfo {
                            path: full_path,
                            name: filename.to_string(),
                            is_active: true, // Window title shows the active file
                            is_modified,
                            tab_index: None,
                            project_name: Some(project_name.to_string()),
                        });
                    }
                }
            }
        }

        None
    }
}

impl IDEDetector for JetBrainsDetector {
    fn ide_type(&self) -> SupportedIDE {
        self.ide_type
    }

    fn is_target_process(&self, process: &ProcessInfo) -> bool {
        self.process_names.iter().any(|&name| 
            process.name.to_lowercase().contains(&name.to_lowercase())
        )
    }

    fn extract_files(&self, processes: &[ProcessInfo]) -> DetectionResult<crate::types::DetectionResult> {
        let mut open_files = Vec::new();
        let mut active_file = None;
        let mut project_path = None;
        let mut ide_version = None;

        for process in processes {
            if let Some(file_info) = self.parse_jetbrains_window_title(&process.window_title) {
                if file_info.is_active {
                    active_file = Some(file_info.path.clone());
                }
                
                // Try to extract project path from window title
                if project_path.is_none() && file_info.project_name.is_some() {
                    // This could be further parsed from window title
                    project_path = Some("/extracted/project/path".to_string());
                }

                open_files.push(file_info);
            }
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