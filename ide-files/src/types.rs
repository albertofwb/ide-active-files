use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_active: bool,
    pub is_modified: bool,
    pub tab_index: Option<usize>,
    pub project_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetectionResult {
    pub timestamp: String,
    pub ide_name: String,
    pub ide_version: Option<String>,
    pub active_file: Option<String>,
    pub open_files: Vec<FileInfo>,
    pub project_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub window_title: String,
    pub executable_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedIDE {
    GoLand,
    PyCharm,
    IntelliJIDEA,
    VSCode,
    VisualStudio,
    WebStorm,
    PhpStorm,
    RubyMine,
    CLion,
    Vim,
    Nano,
}

impl SupportedIDE {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedIDE::GoLand => "goland",
            SupportedIDE::PyCharm => "pycharm",
            SupportedIDE::IntelliJIDEA => "idea",
            SupportedIDE::VSCode => "vscode",
            SupportedIDE::VisualStudio => "vs",
            SupportedIDE::WebStorm => "webstorm",
            SupportedIDE::PhpStorm => "phpstorm",
            SupportedIDE::RubyMine => "rubymine",
            SupportedIDE::CLion => "clion",
            SupportedIDE::Vim => "vim",
            SupportedIDE::Nano => "nano",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SupportedIDE::GoLand => "GoLand",
            SupportedIDE::PyCharm => "PyCharm",
            SupportedIDE::IntelliJIDEA => "IntelliJ IDEA",
            SupportedIDE::VSCode => "Visual Studio Code",
            SupportedIDE::VisualStudio => "Visual Studio",
            SupportedIDE::WebStorm => "WebStorm",
            SupportedIDE::PhpStorm => "PhpStorm",
            SupportedIDE::RubyMine => "RubyMine",
            SupportedIDE::CLion => "CLion",
            SupportedIDE::Vim => "Vim",
            SupportedIDE::Nano => "Nano",
        }
    }

    pub fn all() -> Vec<SupportedIDE> {
        vec![
            SupportedIDE::GoLand,
            SupportedIDE::PyCharm,
            SupportedIDE::IntelliJIDEA,
            SupportedIDE::VSCode,
            SupportedIDE::VisualStudio,
            SupportedIDE::WebStorm,
            SupportedIDE::PhpStorm,
            SupportedIDE::RubyMine,
            SupportedIDE::CLion,
            SupportedIDE::Vim,
            SupportedIDE::Nano,
        ]
    }

    pub fn from_str(s: &str) -> Option<SupportedIDE> {
        SupportedIDE::all().into_iter()
            .find(|ide| ide.as_str().eq_ignore_ascii_case(s))
    }
}