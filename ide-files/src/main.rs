mod detector;
mod detectors;
mod process;
mod types;

use clap::{Arg, Command};
use detector::IDEDetectorManager;
use detectors::jetbrains::JetBrainsDetector;
use detectors::terminal::TerminalEditorDetector;
use std::process::exit;
use types::SupportedIDE;

fn main() {
    let matches = Command::new("ide-files")
        .version("0.1.0")
        .author("Your Name")
        .about("Extract open files from various IDEs")
        .arg(
            Arg::new("ide")
                .long("ide")
                .value_name("IDE")
                .help("Specify IDE to detect (goland, pycharm, idea, vscode, etc.)"),
        )
        .arg(
            Arg::new("list-ides")
                .long("list-ides")
                .action(clap::ArgAction::SetTrue)
                .help("List all supported IDEs"),
        )
        .arg(
            Arg::new("auto")
                .long("auto")
                .action(clap::ArgAction::SetTrue)
                .help("Auto-detect any supported IDE"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .default_value("json")
                .help("Output format: json, plain, or paths"),
        )
        .arg(
            Arg::new("active")
                .long("active")
                .action(clap::ArgAction::SetTrue)
                .help("Only return the currently active file"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        .arg(
            Arg::new("debug-processes")
                .long("debug-processes")
                .action(clap::ArgAction::SetTrue)
                .help("List all running processes (debug mode)"),
        )
        .get_matches();

    // Initialize detector manager
    let mut manager = IDEDetectorManager::new();

    // Register JetBrains detectors
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::GoLand)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::PyCharm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::IntelliJIDEA)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::WebStorm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::PhpStorm)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::RubyMine)));
    manager.register_detector(Box::new(JetBrainsDetector::new(SupportedIDE::CLion)));

    // Register terminal editor detectors (for testing)
    manager.register_detector(Box::new(TerminalEditorDetector::new(SupportedIDE::Vim)));
    manager.register_detector(Box::new(TerminalEditorDetector::new(SupportedIDE::Nano)));

    let verbose = matches.get_flag("verbose");

    // Handle debug processes
    if matches.get_flag("debug-processes") {
        if let Err(e) = crate::process::list_all_processes() {
            eprintln!("Error listing processes: {}", e);
        }
        return;
    }

    // Handle list IDEs command
    if matches.get_flag("list-ides") {
        println!("Supported IDEs:");
        for ide in SupportedIDE::all() {
            println!("  {} (--ide={})", ide.display_name(), ide.as_str());
        }
        return;
    }

    // Execute detection
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
            eprintln!(
                "Error: Unsupported IDE '{}'. Use --list-ides to see supported IDEs.",
                ide_str
            );
            return;
        }
    } else {
        // Default to auto-detect
        if verbose {
            eprintln!("Auto-detecting IDEs...");
        }
        manager.auto_detect()
    };

    match result {
        Ok(detection_result) => {
            if verbose {
                eprintln!(
                    "Successfully detected {}: {} open files",
                    detection_result.ide_name,
                    detection_result.open_files.len()
                );
            }
            output_result(&matches, &detection_result);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}

fn output_result(matches: &clap::ArgMatches, data: &types::DetectionResult) {
    let format = matches
        .get_one::<String>("format")
        .map(|s| s.as_str())
        .unwrap_or("json");
    let active_only = matches.get_flag("active");

    match format {
        "plain" => {
            let files = if active_only {
                data.open_files
                    .iter()
                    .filter(|f| f.is_active)
                    .collect::<Vec<_>>()
            } else {
                data.open_files.iter().collect::<Vec<_>>()
            };

            for file in files {
                println!("{}: {}", if file.is_active { "*" } else { " " }, file.path);
            }
        }
        "paths" => {
            let files = if active_only {
                data.open_files
                    .iter()
                    .filter(|f| f.is_active)
                    .collect::<Vec<_>>()
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
                    let active_file_data = data
                        .open_files
                        .iter()
                        .find(|f| f.path == *active_file)
                        .cloned();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&active_file_data).unwrap()
                    );
                } else {
                    println!("null");
                }
            } else {
                println!("{}", serde_json::to_string_pretty(data).unwrap());
            }
        }
    }
}
