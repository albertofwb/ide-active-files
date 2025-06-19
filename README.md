# IDE Active Files Detection Tool

A Rust-based command-line tool that detects currently open files in various IDEs through system APIs, designed to provide context information for MCP (Model Context Protocol).

## Features

- **Multi-IDE Support**: Detects open files in JetBrains IDEs (GoLand, PyCharm, IntelliJ IDEA, WebStorm, PhpStorm, RubyMine, CLion), Visual Studio Code, and terminal editors (Vim, Nano)
- **Cross-Platform**: Supports Windows, macOS, and Linux
- **Strategy Pattern**: Extensible architecture for adding new IDE detectors
- **Multiple Output Formats**: JSON, plain text, or file paths only
- **Active File Detection**: Identifies the currently focused file

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Platform-specific dependencies:
  - **Linux**: X11 development libraries (`sudo apt install libx11-dev`)
  - **macOS**: Xcode command line tools
  - **Windows**: Visual Studio Build Tools

### Build from Source

```bash
cd ide-files
cargo build --release
```

The binary will be available at `target/release/ide-files`

## Usage

### Basic Commands

```bash
# Auto-detect any supported IDE
./ide-files --auto

# Detect specific IDE
./ide-files --ide=goland
./ide-files --ide=pycharm
./ide-files --ide=vscode

# List all supported IDEs
./ide-files --list-ides

# Get help
./ide-files --help
```

### Output Formats

```bash
# JSON output (default)
./ide-files --auto --format=json

# Plain text with active file indicator
./ide-files --auto --format=plain

# File paths only
./ide-files --auto --format=paths

# Only the currently active file
./ide-files --auto --active
```

### Verbose Mode

```bash
# Enable detailed logging
./ide-files --auto --verbose
```

## Supported IDEs

- **JetBrains Family**:
  - GoLand (`--ide=goland`)
  - PyCharm (`--ide=pycharm`)
  - IntelliJ IDEA (`--ide=idea`)
  - WebStorm (`--ide=webstorm`)
  - PhpStorm (`--ide=phpstorm`)
  - RubyMine (`--ide=rubymine`)
  - CLion (`--ide=clion`)

- **Terminal Editors**:
  - Vim/NeoVim (`--ide=vim`)
  - Nano (`--ide=nano`)

- **Other IDEs** (planned):
  - Visual Studio Code (`--ide=vscode`)
  - Visual Studio (`--ide=vs`)

## Output Format

### JSON Output Example

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "ide_name": "GoLand",
  "ide_version": "2023.3",
  "active_file": "/path/to/project/main.go",
  "open_files": [
    {
      "path": "/path/to/project/main.go",
      "name": "main.go",
      "is_active": true,
      "is_modified": false,
      "tab_index": 0,
      "project_name": "my-project"
    }
  ],
  "project_path": "/path/to/project"
}
```

### Plain Text Output Example

```
* /path/to/project/main.go
  /path/to/project/utils.go
  /path/to/project/config.yaml
```

(The `*` indicates the currently active file)

## Testing

### Test Terminal Editors

Run the included test script to verify terminal editor detection:

```bash
cd ide-files
chmod +x test_terminal_editors.sh
./test_terminal_editors.sh
```

This will create test files and provide instructions for manual testing with Vim and Nano.

### Debug Mode

List all running processes (useful for debugging):

```bash
./ide-files --debug-processes
```

## Development

### Architecture

The tool uses a **Strategy Pattern** for IDE detection:

- `IDEDetector` trait defines the interface for all detectors
- `IDEDetectorManager` manages and coordinates different detectors
- Each IDE has its own detector implementation in `src/detectors/`

### Adding New IDE Support

1. Create a new detector in `src/detectors/`
2. Implement the `IDEDetector` trait
3. Register the detector in `src/main.rs`
4. Add the IDE to `SupportedIDE` enum in `src/types.rs`

### Project Structure

```
ide-files/
├── src/
│   ├── main.rs              # CLI interface and main logic
│   ├── types.rs             # Data structures and enums
│   ├── detector.rs          # Strategy pattern traits and manager
│   ├── process.rs           # Cross-platform process detection
│   └── detectors/
│       ├── mod.rs           # Detector module exports
│       ├── jetbrains.rs     # JetBrains IDEs detector
│       └── terminal.rs      # Terminal editors detector
├── Cargo.toml
├── test_terminal_editors.sh # Testing script
└── README.md
```

## Platform-Specific Notes

### Windows
- Uses `winapi` crate for process and window enumeration
- Supports both 32-bit and 64-bit IDE processes

### macOS
- Uses `core-foundation` and `cocoa` crates
- Requires accessibility permissions for some IDEs

### Linux
- Uses X11 for window information
- Reads `/proc` filesystem for process details

## MCP Integration

This tool is designed to work with Model Context Protocol (MCP) to provide context about currently open files to AI assistants and other tools.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-ide-support`)
3. Make your changes and add tests
4. Commit your changes (`git commit -am 'Add support for New IDE'`)
5. Push to the branch (`git push origin feature/new-ide-support`)
6. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Troubleshooting

### Common Issues

- **No processes found**: Make sure the IDE is actually running
- **Permission denied**: On macOS, you may need to grant accessibility permissions
- **Build errors**: Ensure you have the correct platform dependencies installed

### Getting Help

- Check the verbose output: `--verbose`
- List all processes: `--debug-processes`
- Verify supported IDEs: `--list-ides`

For bugs and feature requests, please open an issue on GitHub.