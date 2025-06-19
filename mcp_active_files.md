# IDE 文件检测工具开发指南

## 项目概述

开发一个 Rust 命令行工具，通过系统 API 检测各种 IDE 中当前打开的文件，为 MCP (Model Context Protocol) 提供上下文信息。

## 设计模式：策略模式

使用策略模式支持多种 IDE，每个 IDE 作为一个独立的检测策略，便于扩展和维护。

### 架构设计

```
IDE Detector (Context)
    ├── GoLand Strategy
    ├── PyCharm Strategy  
    ├── VSCode Strategy
    ├── IntelliJ IDEA Strategy
    └── Visual Studio Strategy
```

## 技术栈

- **语言：** Rust
- **设计模式：** Strategy Pattern + Factory Pattern
- **系统 API：** winapi (Windows), cocoa/core-foundation (macOS), x11 (Linux)
- **CLI 框架：** clap
- **JSON 处理：** serde, serde_json

## 开发阶段规划

### 阶段 1：项目初始化和策略模式框架

**目标：** 创建基础项目结构，实现策略模式框架

**验收标准：**
```bash
cargo run -- --help           # 显示帮助信息
cargo run -- --ide=goland     # 指定检测 GoLand
cargo run -- --list-ides      # 列出支持的 IDE
```

#### 步骤 1.1：创建项目

```bash
cargo new ide-files --bin
cd ide-files
```

#### 步骤 1.2：配置 Cargo.toml

```toml
[package]
name = "ide-files"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.0"
thiserror = "1.0"

# 平台特定依赖
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winuser", "processthreadsapi", "handleapi", "tlhelp32"] }

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
cocoa = "0.24"

[target.'cfg(target_os = "linux")'.dependencies]
x11 = { version = "2.18", features = ["xlib"] }
```

#### 步骤 1.3：定义核心数据结构

```rust
// src/types.rs
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
```

#### 步骤 1.4：定义策略模式接口

```rust
// src/detector.rs
use crate::types::{DetectionResult, ProcessInfo, SupportedIDE};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("No {ide} processes found")]
    NoProcessFound { ide: String },
    #[error("Failed to parse window information: {message}")]
    WindowParseError { message: String },
    #[error("System API error: {message}")]
    SystemError { message: String },
    #[error("Unsupported IDE: {ide}")]
    UnsupportedIDE { ide: String },
}

pub type DetectionResult<T> = Result<T, DetectionError>;

/// IDE 检测策略 trait
pub trait IDEDetector {
    /// 获取 IDE 类型
    fn ide_type(&self) -> SupportedIDE;
    
    /// 检测进程是否为目标 IDE
    fn is_target_process(&self, process: &ProcessInfo) -> bool;
    
    /// 从进程信息中提取文件信息
    fn extract_files(&self, processes: &[ProcessInfo]) -> DetectionResult<crate::types::DetectionResult>;
    
    /// 获取 IDE 显示名称
    fn display_name(&self) -> &'static str {
        self.ide_type().display_name()
    }
}

/// IDE 检测器管理器
pub struct IDEDetectorManager {
    detectors: Vec<Box<dyn IDEDetector>>,
}

impl IDEDetectorManager {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    pub fn register_detector(&mut self, detector: Box<dyn IDEDetector>) {
        self.detectors.push(detector);
    }

    pub fn detect_ide(&self, ide_type: SupportedIDE) -> DetectionResult<crate::types::DetectionResult> {
        let detector = self.detectors.iter()
            .find(|d| d.ide_type() == ide_type)
            .ok_or_else(|| DetectionError::UnsupportedIDE { 
                ide: ide_type.display_name().to_string() 
            })?;

        let processes = crate::process::find_all_processes()?;
        let target_processes: Vec<_> = processes.iter()
            .filter(|p| detector.is_target_process(p))
            .cloned()
            .collect();

        if target_processes.is_empty() {
            return Err(DetectionError::NoProcessFound { 
                ide: detector.display_name().to_string() 
            });
        }

        detector.extract_files(&target_processes)
    }

    pub fn auto_detect(&self) -> DetectionResult<crate::types::DetectionResult> {
        let processes = crate::process::find_all_processes()?;
        
        for detector in &self.detectors {
            let target_processes: Vec<_> = processes.iter()
                .filter(|p| detector.is_target_process(p))
                .cloned()
                .collect();

            if !target_processes.is_empty() {
                return detector.extract_files(&target_processes);
            }
        }

        Err(DetectionError::NoProcessFound { 
            ide: "any supported IDE".to_string() 
        })
    }

    pub fn list_supported_ides(&self) -> Vec<&'static str> {
        self.detectors.iter()
            .map(|d| d.display_name())
            .collect()
    }
}
```

#### 步骤 1.5：实现进程检测模块

```rust
// src/process.rs
use crate::types::ProcessInfo;
use crate::detector::DetectionResult;

pub fn find_all_processes() -> DetectionResult<Vec<ProcessInfo>> {
    #[cfg(target_os = "windows")]
    return find_processes_windows();
    
    #[cfg(target_os = "macos")]
    return find_processes_macos();
    
    #[cfg(target_os = "linux")]
    return find_processes_linux();
}

#[cfg(target_os = "windows")]
fn find_processes_windows() -> DetectionResult<Vec<ProcessInfo>> {
    use winapi::um::tlhelp32::*;
    use winapi::um::handleapi::*;
    use winapi::shared::minwindef::*;
    use std::ffi::CStr;
    use std::mem;
    use crate::detector::DetectionError;

    let mut processes = Vec::new();
    
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(DetectionError::SystemError {
                message: "Failed to create process snapshot".to_string()
            });
        }

        let mut entry: PROCESSENTRY32 = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry) == TRUE {
            loop {
                let process_name = CStr::from_ptr(entry.szExeFile.as_ptr())
                    .to_string_lossy()
                    .to_string();
                
                let window_title = get_window_title_by_pid(entry.th32ProcessID);
                
                processes.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    name: process_name,
                    window_title,
                    executable_path: String::new(), // TODO: 获取完整路径
                });

                if Process32Next(snapshot, &mut entry) != TRUE {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }
    
    Ok(processes)
}

#[cfg(target_os = "windows")]
fn get_window_title_by_pid(pid: u32) -> String {
    use winapi::um::winuser::*;
    use std::mem;
    use std::ptr;

    unsafe {
        let mut window_title = String::new();
        
        EnumWindows(Some(enum_windows_proc), &mut window_title as *mut String as isize);
        
        window_title
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_proc(hwnd: winapi::shared::windef::HWND, lparam: isize) -> i32 {
    // 实现窗口枚举逻辑
    1
}

#[cfg(target_os = "macos")]
fn find_processes_macos() -> DetectionResult<Vec<ProcessInfo>> {
    // TODO: 实现 macOS 版本
    Ok(vec![])
}

#[cfg(target_os = "linux")]
fn find_processes_linux() -> DetectionResult<Vec<ProcessInfo>> {
    // TODO: 实现 Linux 版本
    Ok(vec![])
}
```

#### 步骤 1.6：实现 JetBrains 系列基础检测器

```rust
// src/detectors/jetbrains.rs
use crate::detector::{IDEDetector, DetectionResult};
use crate::types::{ProcessInfo, SupportedIDE, FileInfo};
use regex::Regex;

/// JetBrains 系列 IDE 的基础检测器
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
        // JetBrains IDE 窗口标题格式：
        // "filename.ext - project-name [/path/to/project] - IDE-Name 202X.X"
        // "filename.ext* - project-name [/path/to/project] - IDE-Name 202X.X" (修改状态)
        
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
                            is_active: true, // 窗口标题显示的文件通常是激活的
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
                
                // 尝试从项目名称推导项目路径
                if project_path.is_none() && file_info.project_name.is_some() {
                    // 这里可以进一步解析窗口标题中的路径信息
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
```

#### 步骤 1.7：实现主函数和 CLI

```rust
// src/main.rs
mod types;
mod detector;
mod process;
mod detectors;

use clap::{Arg, Command};
use detector::IDEDetectorManager;
use detectors::jetbrains::JetBrainsDetector;
use types::SupportedIDE;
use std::process;

fn main() {
    let matches = Command::new("ide-files")
        .version("0.1.0")
        .author("Your Name")
        .about("Extract open files from various IDEs")
        .arg(Arg::new("ide")
            .long("ide")
            .value_name("IDE")
            .help("Specify IDE to detect (goland, pycharm, idea, vscode, etc.)"))
        .arg(Arg::new("list-ides")
            .long("list-ides")
            .action(clap::ArgAction::SetTrue)
            .help("List all supported IDEs"))
        .arg(Arg::new("auto")
            .long("auto")
            .action(clap::ArgAction::SetTrue)
            .help("Auto-detect any supported IDE"))
        .arg(Arg::new("format")
            .long("format")
            .value_name("FORMAT")
            .default_value("json")
            .help("Output format: json, plain, or paths"))
        .arg(Arg::new("active")
            .long("active")
            .action(clap::ArgAction::SetTrue)
            .help("Only return the currently active file"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .action(clap::ArgAction::SetTrue)
            .help("Enable verbose output"))
        .get_matches();

    // 初始化检测器管理器
    let mut manager = IDEDetectorManager::new();
    
    // 注册 JetBrains 系列检测器
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::GoLand)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::PyCharm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::IntelliJIDEA)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::WebStorm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::PhpStorm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::RubyMine)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::CLion)));

    // 注册终端编辑器检测器（便于测试）
    manager.register_detector(Box::new(detectors::terminal::TerminalEditorDetector::new(SupportedIDE::Vim)));
    manager.register_detector(Box::new(detectors::terminal::TerminalEditorDetector::new(SupportedIDE::Nano)));

    let verbose = matches.get_flag("verbose");

    // 处理列出 IDE 命令
    if matches.get_flag("list-ides") {
        println!("Supported IDEs:");
        for ide in SupportedIDE::all() {
            println!("  {} (--ide={})", ide.display_name(), ide.as_str());
        }
        return;
    }

    // 执行检测
    let result = if matches.get_flag("auto") {
        if verbose {
            eprintln!("Auto-detecting IDEs...");
        }
        manager.auto_detect()
    } else if let Some(ide_str) = matches.get_one::<String>("ide") {
        if let Some(ide_type) = SupportedIDE::from_str(ide_str) {
            if verbose {
                eprintln!("Detecting {}...", ide_type.display_name());
            }
            manager.detect_ide(ide_type)
        } else {
            eprintln!("Error: Unsupported IDE '{}'. Use --list-ides to see supported IDEs.", ide_str);
            process::exit(1);
        }
    } else {
        // 默认自动检测
        if verbose {
            eprintln!("Auto-detecting IDEs...");
        }
        manager.auto_detect()
    };

    match result {
        Ok(detection_result) => {
            if verbose {
                eprintln!("Successfully detected {}: {} open files", 
                    detection_result.ide_name, 
                    detection_result.open_files.len());
            }
            output_result(&matches, &detection_result);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn output_result(matches: &clap::ArgMatches, data: &types::DetectionResult) {
    let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("json");
    let active_only = matches.get_flag("active");

    match format {
        "plain" => {
            let files = if active_only {
                data.open_files.iter().filter(|f| f.is_active).collect::<Vec<_>>()
            } else {
                data.open_files.iter().collect::<Vec<_>>()
            };

            for file in files {
                println!("{}: {}", 
                    if file.is_active { "*" } else { " " }, 
                    file.path);
            }
        }
        "paths" => {
            let files = if active_only {
                data.open_files.iter().filter(|f| f.is_active).collect::<Vec<_>>()
            } else {
                data.open_files.iter().collect::<Vec<_>>()
            };

            for file in files {
                println!("{}", file.path);
            }
        }
        _ => {
            if active_only {
                if let Some(active_file) = &data.active_file {
                    let active_file_data = data.open_files.iter()
                        .find(|f| f.path == *active_file)
                        .cloned();
                    println!("{}", serde_json::to_string_pretty(&active_file_data).unwrap());
                } else {
                    println!("null");
                }
            } else {
                println!("{}", serde_json::to_string_pretty(data).unwrap());
            }
        }
    }
}
```

#### 步骤 1.8：创建模块文件

```rust
// src/detectors/mod.rs
pub mod jetbrains;
pub mod terminal;
```

**验证步骤 1：**
```bash
cargo build
./target/debug/ide-files --help
./target/debug/ide-files --list-ides
./target/debug/ide-files --auto --verbose

# 测试终端编辑器检测
vim /tmp/test.txt                    # 在另一个终端打开
./target/debug/ide-files --ide=vim --verbose

nano /tmp/test.txt                   # 在另一个终端打开  
./target/debug/ide-files --ide=nano --verbose

./target/debug/ide-files --ide=goland --verbose
./target/debug/ide-files --ide=pycharm --format=plain
```

---

### 阶段 1.9：实现终端编辑器检测器

**目标：** 实现 vim 和 nano 等终端编辑器的文件检测，便于测试框架

**验收标准：**
```bash
# 在一个终端打开文件
vim /path/to/file.txt

# 在另一个终端检测
cargo run -- --ide=vim --verbose
# 应该检测到 vim 进程和正在编辑的文件
```

#### 步骤 1.9.1：创建终端编辑器检测器

```rust
// src/detectors/terminal.rs
use crate::detector::{IDEDetector, DetectionResult};
use crate::types::{ProcessInfo, SupportedIDE, FileInfo};
use std::process::Command;
use std::collections::HashMap;

/// 终端编辑器检测器
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

    /// 通过 /proc 文件系统获取进程命令行参数 (Linux/macOS)
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
            // macOS 使用 ps 命令获取进程参数
            let output = Command::new("ps")
                .args(&["-p", &pid.to_string(), "-o", "args="])
                .output()
                .ok()?;

            let cmdline = String::from_utf8_lossy(&output.stdout);
            Some(cmdline.trim().split_whitespace().map(|s| s.to_string()).collect())
        }
    }

    /// Windows 下获取进程命令行参数
    #[cfg(target_os = "windows")]
    fn get_process_cmdline(&self, pid: u32) -> Option<Vec<String>> {
        // Windows 实现较复杂，暂时使用简化版本
        // 可以使用 WMI 或 PowerShell 来获取完整命令行
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

        // 查找文件参数
        let file_path = match self.ide_type {
            SupportedIDE::Vim => {
                // vim 通常格式：vim /path/to/file.txt
                // nvim 格式：nvim /path/to/file.txt  
                // 可能有选项：vim -n /path/to/file.txt
                cmdline.iter()
                    .skip(1) // 跳过程序名
                    .find(|arg| !arg.starts_with('-') && !arg.is_empty())
                    .cloned()
            }
            SupportedIDE::Nano => {
                // nano 通常格式：nano /path/to/file.txt
                // 可能有选项：nano -w /path/to/file.txt
                cmdline.iter()
                    .skip(1) // 跳过程序名
                    .find(|arg| !arg.starts_with('-') && !arg.is_empty())
                    .cloned()
            }
            _ => None,
        }?;

        // 转换为绝对路径
        let absolute_path = if file_path.starts_with('/') {
            file_path
        } else {
            // 相对路径，尝试获取当前工作目录
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
            is_active: true, // 终端编辑器通常只编辑一个文件
            is_modified: false, // 无法简单检测修改状态
            tab_index: Some(0),
            project_name: None,
        })
    }

    /// 检查文件是否存在
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
                    // 验证文件是否真实存在
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
```

#### 步骤 1.9.2：添加依赖

更新 `Cargo.toml`：

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.0"
thiserror = "1.0"
shell-words = "1.1"  # 新增：用于解析命令行参数

# 平台特定依赖保持不变...
```

#### 步骤 1.9.3：更新主函数

在 `src/main.rs` 中添加终端编辑器检测器的注册：

```rust
use detectors::terminal::TerminalEditorDetector;

fn main() {
    // ... 之前的代码 ...
    
    // 注册 JetBrains 系列检测器
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::GoLand)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::PyCharm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::IntelliJIDEA)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::WebStorm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::PhpStorm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::RubyMine)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::CLion)));

    // 注册终端编辑器检测器（便于测试）
    manager.register_detector(Box::new(TerminalEditorDetector::new(SupportedIDE::Vim)));
    manager.register_detector(Box::new(TerminalEditorDetector::new(SupportedIDE::Nano)));

    // ... 剩余代码 ...
}
```

#### 步骤 1.9.4：创建测试脚本

创建 `test_terminal_editors.sh` 脚本便于测试：

```bash
#!/bin/bash
# test_terminal_editors.sh

echo "=== 测试终端编辑器检测 ==="

# 创建测试文件
mkdir -p /tmp/ide-test
echo "console.log('Hello from test file');" > /tmp/ide-test/test.js
echo "print('Hello from Python test')" > /tmp/ide-test/test.py
echo "package main\n\nfunc main() {\n    println(\"Hello from Go test\")\n}" > /tmp/ide-test/test.go

echo "测试文件已创建在 /tmp/ide-test/"

echo ""
echo "请在不同终端中运行以下命令："
echo "1. vim /tmp/ide-test/test.go"
echo "2. nano /tmp/ide-test/test.py"
echo ""
echo "然后运行以下检测命令："
echo "cargo run -- --ide=vim --verbose"
echo "cargo run -- --ide=nano --verbose" 
echo "cargo run -- --auto --verbose"
echo ""
echo "预期结果：应该检测到对应的编辑器和正在编辑的文件"
```

#### 步骤 1.9.5：添加进程检测增强

为了更好地支持终端编辑器，增强进程检测功能：

```rust
// 在 src/process.rs 中添加
pub fn find_processes_by_name(name: &str) -> DetectionResult<Vec<ProcessInfo>> {
    let all_processes = find_all_processes()?;
    
    Ok(all_processes.into_iter()
        .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
        .collect())
}

// 添加调试辅助函数
pub fn list_all_processes() -> DetectionResult<()> {
    let processes = find_all_processes()?;
    
    println!("All running processes:");
    for process in processes {
        if !process.name.is_empty() {
            println!("PID: {}, Name: {}, Title: {}", 
                process.pid, process.name, 
                if process.window_title.is_empty() { "<no title>" } else { &process.window_title });
        }
    }
    
    Ok(())
}
```

并在主函数中添加调试选项：

```rust
// 在 src/main.rs 中添加
.arg(Arg::new("debug-processes")
    .long("debug-processes")
    .action(clap::ArgAction::SetTrue)
    .help("List all running processes (debug mode)"))

// 在主函数逻辑中添加
if matches.get_flag("debug-processes") {
    if let Err(e) = crate::process::list_all_processes() {
        eprintln!("Error listing processes: {}", e);
    }
    return;
}
```

---

### 阶段 2：完善 Windows 平台检测

**目标：** 完整实现 Windows 平台的进程和窗口检测

**验收标准：**
```bash
# 在 Windows 上启动 GoLand，然后运行
cargo run -- --ide=goland --verbose
# 应该正确检测到 GoLand 进程和当前打开的文件
```

#### 步骤 2.1：完善 Windows 进程检测

```rust
// 更新 src/process.rs 中的 Windows 实现
#[cfg(target_os = "windows")]
fn get_window_title_by_pid(pid: u32) -> String {
    use winapi::um::winuser::*;
    use winapi::shared::windef::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    struct EnumData {
        target_pid: u32,
        window_titles: Vec<String>,
    }

    unsafe {
        let mut enum_data = EnumData {
            target_pid: pid,
            window_titles: Vec::new(),
        };

        EnumWindows(
            Some(enum_windows_proc), 
            &mut enum_data as *mut EnumData as isize
        );

        enum_data.window_titles.into_iter()
            .find(|title| !title.trim().is_empty())
            .unwrap_or_else(|| format!("Process {}", pid))
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: isize) -> i32 {
    use winapi::um::winuser::*;
    use winapi::um::processthreadsapi::*;
    use std::mem;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let enum_data = &mut *(lparam as *mut EnumData);
    
    let mut window_pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut window_pid);
    
    if window_pid == enum_data.target_pid {
        // 获取窗口标题
        let mut title_buffer: [u16; 512] = mem::zeroed();
        let title_len = GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), 512);
        
        if title_len > 0 {
            let title = OsString::from_wide(&title_buffer[..title_len as usize])
                .to_string_lossy()
                .to_string();
            
            if !title.trim().is_empty() {
                enum_data.window_titles.push(title);
            }
        }
    }
    
    1 // 继续枚举
}

struct EnumData {
    target_pid: u32,
    window_titles: Vec<String>,
}
```

#### 步骤 2.2：改进 JetBrains 窗口标题解析

```rust
// 更新 src/detectors/jetbrains.rs
impl JetBrainsDetector {
    fn parse_jetbrains_window_title(&self, title: &str) -> Vec<FileInfo> {
        let mut files = Vec::new();
        
        // JetBrains IDE 可能有多个窗口标题格式
        let patterns = [
            // 标准格式：filename.ext - project-name [/path/to/project] - IDE-Name 202X.X
            r"^([^-]+?)\s*(\*)?\s*-\s*([^[]+?)\s*\[([^\]]+)\]\s*-\s*(\w+)\s+([\d.]+)",
            // 简化格式：filename.ext - project-name - IDE-Name
            r"^([^-]+?)\s*(\*)?\s*-\s*([^-]+?)\s*-\s*(\w+)(?:\s+([\d.]+))?",
            // 无项目格式：filename.ext - IDE-Name
            r"^([^-]+?)\s*(\*)?\s*-\s*(\w+)(?:\s+([\d.]+))?",
        ];

        for (i, pattern) in patterns.iter().enumerate() {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(title) {
                    let filename = captures.get(1).unwrap().as_str().trim();
                    let is_modified = captures.get(2).is_some();
                    
                    let (project_name, project_path, ide_name, ide_version) = match i {
                        0 => {
                            // 完整格式
                            let project_name = captures.get(3).map(|m| m.as_str().trim().to_string());
                            let project_path = captures.get(4).map(|m| m.as_str().trim().to_string());
                            let ide_name = captures.get(5).map(|m| m.as_str().trim().to_string());
                            let ide_version = captures.get(6).map(|m| m.as_str().trim().to_string());
                            (project_name, project_path, ide_name, ide_version)
                        }
                        1 => {
                            // 简化格式
                            let project_name = captures.get(3).map(|m| m.as_str().trim().to_string());
                            let ide_name = captures.get(4).map(|m| m.as_str().trim().to_string());
                            let ide_version = captures.get(5).map(|m| m.as_str().trim().to_string());
                            (project_name, None, ide_name, ide_version)
                        }
                        2 => {
                            // 无项目格式
                            let ide_name = captures.get(3).map(|m| m.as_str().trim().to_string());
                            let ide_version = captures.get(4).map(|m| m.as_str().trim().to_string());
                            