# IDE Files Detection Tool

A Rust command-line tool that detects currently open files in various IDEs and editors, designed for integration with Model Context Protocol (MCP) systems.

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Example Output](#example-output)
- [Supported IDEs](#supported-ides)
- [Development](#development)
- [Architecture](#architecture)
- [MCP Integration](#mcp-integration)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Features

- 🔍 **Multi-IDE Support**: Detects files from GoLand, PyCharm, IntelliJ IDEA, VSCode, Vim, Nano, and more
- 🎯 **Strategy Pattern**: Extensible architecture for adding new IDE detectors
- 🖥️ **Cross-Platform**: Works on Linux, macOS, and Windows
- 📊 **Multiple Output Formats**: JSON, plain text, or file paths only
- ⚡ **Fast Detection**: Efficient process scanning and file extraction
- 🔧 **CLI Interface**: Rich command-line interface with verbose modes

## Quick Start

```bash
# List all supported IDEs
./ide-files --list-ides

# Auto-detect any running IDE
./ide-files --auto --verbose

# Check specific IDE
./ide-files --ide=vim --format=json

# Get only file paths
./ide-files --auto --format=paths

# Get only the active file
./ide-files --auto --active
```

## Installation

### Prerequisites

See **[SETUP.md](SETUP.md)** for detailed environment setup instructions for your platform.

### Quick Install

```bash
# Clone and build
git clone <repository-url>
cd ide-files
make build

# Install system-wide to /usr/local/bin/ (requires sudo)
make install

# Set up shell auto-completion (bash/zsh)
make install-completion

# Test installation
ide-files --help
ide-files --list-ides
```

## Usage

### Basic Commands

```bash
# Show help
./ide-files --help

# List supported IDEs
./ide-files --list-ides

# Auto-detect running IDEs
./ide-files --auto

# Detect specific IDE
./ide-files --ide=goland
./ide-files --ide=vim
./ide-files --ide=vscode
```

### Output Formats

```bash
# JSON format (default)
./ide-files --auto --format=json

# Plain text format
./ide-files --auto --format=plain

# File paths only
./ide-files --auto --format=paths
```

### Filtering Options

```bash
# Get only active file
./ide-files --auto --active

# Verbose output with detection details
./ide-files --auto --verbose

# Debug mode - list all processes
./ide-files --debug-processes
```

## Example Output

### JSON Format
```json
{
  "timestamp": "2025-06-19T16:45:45.409259279+00:00",
  "ide_name": "Vim",
  "ide_version": null,
  "active_file": "/tmp/ide-test/test.go",
  "open_files": [
    {
      "path": "/tmp/ide-test/test.go",
      "name": "test.go",
      "is_active": true,
      "is_modified": false,
      "tab_index": 0,
      "project_name": null
    }
  ],
  "project_path": null
}
```

### Plain Format
```
*: /tmp/ide-test/test.go
```

### Paths Format
```
/tmp/ide-test/test.go
```

## Supported IDEs

| IDE | Status | Platform Support |
|-----|--------|-------------------|
| **JetBrains IDEs** | | |
| GoLand | ✅ Working | Linux, macOS, Windows |
| PyCharm | ✅ Working | Linux, macOS, Windows |
| IntelliJ IDEA | ✅ Working | Linux, macOS, Windows |
| WebStorm | ✅ Working | Linux, macOS, Windows |
| PhpStorm | ✅ Working | Linux, macOS, Windows |
| RubyMine | ✅ Working | Linux, macOS, Windows |
| CLion | ✅ Working | Linux, macOS, Windows |
| **Code Editors** | | |
| Visual Studio Code | 🚧 Planned | Linux, macOS, Windows |
| Visual Studio | 🚧 Planned | Windows |
| **Terminal Editors** | | |
| Vim/Neovim | ✅ Working | Linux, macOS, Windows |
| Nano | ✅ Working | Linux, macOS, Windows |

## Development

### Build Commands

```bash
# Debug build
make build

# Release build  
make release

# Run tests
make test

# Format code
make fmt

# Run linter
make clippy

# Complete development setup
make dev
```

### Project Structure

```
ide-files/
├── src/
│   ├── main.rs              # CLI interface
│   ├── types.rs             # Core data structures
│   ├── detector.rs          # Strategy pattern interface
│   ├── process.rs           # Process detection
│   └── detectors/
│       ├── jetbrains.rs     # JetBrains IDE detector
│       └── terminal.rs      # Terminal editor detector
├── Cargo.toml               # Rust dependencies
├── Makefile                 # Build automation
├── SETUP.md                 # Environment setup guide
└── README.md                # This file
```

## Architecture

The tool uses the **Strategy Pattern** for extensible IDE detection:

```
IDEDetectorManager
├── JetBrainsDetector (GoLand, PyCharm, IntelliJ, etc.)
├── TerminalEditorDetector (Vim, Nano)
├── VSCodeDetector (planned)
└── VisualStudioDetector (planned)
```

Each detector implements the `IDEDetector` trait:
- `ide_type()` - Returns the IDE type
- `is_target_process()` - Checks if a process belongs to this IDE
- `extract_files()` - Extracts file information from processes

## MCP Integration

This tool is designed for integration with Model Context Protocol systems:

```bash
# Get current context in JSON format
./ide-files --auto --format=json

# Get only active file for MCP
./ide-files --auto --active --format=paths

# Monitor specific IDE
./ide-files --ide=goland --format=json
```

## Contributing

1. **Fork the repository**
2. **Set up your environment**: Follow [SETUP.md](SETUP.md)
3. **Create a feature branch**: `git checkout -b feature/new-ide-detector`
4. **Make your changes**: Add new IDE detectors or improve existing ones
5. **Test thoroughly**: `make dev` runs all checks
6. **Submit a pull request**

### Adding New IDE Support

To add support for a new IDE:

1. Create a new detector in `src/detectors/`
2. Implement the `IDEDetector` trait
3. Register the detector in `src/main.rs`
4. Add the IDE to `SupportedIDE` enum in `src/types.rs`
5. Test on your target platform

## Troubleshooting

### Common Issues

- **No processes found**: Make sure the IDE is actually running
- **Permission denied**: Some systems require additional permissions for process scanning
- **Build failures**: Check [SETUP.md](SETUP.md) for platform-specific dependencies

### Debug Mode

Use debug mode to diagnose issues:

```bash
# List all running processes
./ide-files --debug-processes

# Verbose detection output
./ide-files --auto --verbose

# Check specific IDE with verbose output
./ide-files --ide=vim --verbose
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### v0.1.0
- Initial release
- Support for JetBrains IDEs (GoLand, PyCharm, IntelliJ IDEA, etc.)
- Support for terminal editors (Vim, Nano)
- Cross-platform process detection (Linux, macOS, Windows)
- Multiple output formats (JSON, plain, paths)
- Comprehensive CLI interface

## Project Files

- **[SETUP.md](SETUP.md)** - Detailed environment setup instructions for all platforms
- **[TODO.md](TODO.md)** - Development roadmap and planned features
- **[Makefile](Makefile)** - Build automation and development commands

---

For detailed setup instructions, see **[SETUP.md](SETUP.md)**.  
For planned features and development roadmap, see **[TODO.md](TODO.md)**.