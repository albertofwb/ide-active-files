# CLAUDE.md - Development Context for IDE Files Detection Tool

## Project Overview

This is a **Rust CLI tool** that detects currently open files in various IDEs and editors, designed for integration with Model Context Protocol (MCP) systems. The project uses the **Strategy Pattern** for extensible IDE detection.

## Quick Commands

```bash
# Essential development commands
make build              # Build debug version
make install           # Install with idf alias + auto-completion  
make test              # Run tests with test files
make uninstall         # Remove from system
./scripts/test-installation.sh  # Comprehensive installation test

# Quick usage after installation
idf --auto             # Auto-detect any IDE (short alias)
idf --list-ides        # Show all supported IDEs
idf --ide=vim          # Detect specific IDE
```

## Project Structure

```
ide-files/
├── src/
│   ├── main.rs              # CLI interface with clap
│   ├── types.rs             # Core data structures (FileInfo, DetectionResult)
│   ├── detector.rs          # Strategy pattern interface (IDEDetector trait)
│   ├── process.rs           # Cross-platform process detection (/proc on Linux)
│   └── detectors/
│       ├── mod.rs           # Detector module exports
│       ├── jetbrains.rs     # JetBrains IDEs (GoLand, PyCharm, IntelliJ, etc.)
│       ├── terminal.rs      # Terminal editors (Vim, Nano)
│       └── vscode.rs        # Visual Studio Code detection
├── scripts/
│   ├── setup-completion.sh  # Auto-completion setup for bash/zsh
│   └── test-installation.sh # Comprehensive Linux installation test
├── Cargo.toml               # Dependencies and project config
├── Makefile                 # Build automation (comprehensive targets)
├── README.md                # User documentation
├── SETUP.md                 # Environment setup instructions
├── TODO.md                  # Development roadmap
└── CLAUDE.md                # This file - development context
```

## Architecture

### Strategy Pattern Implementation

```rust
// Core trait in src/detector.rs
pub trait IDEDetector {
    fn ide_type(&self) -> SupportedIDE;
    fn is_target_process(&self, process: &ProcessInfo) -> bool;
    fn extract_files(&self, processes: &[ProcessInfo]) -> DetectionResult<DetectionResult>;
}

// Manager coordinates all detectors
pub struct IDEDetectorManager {
    detectors: Vec<Box<dyn IDEDetector>>,
}
```

### Key Data Structures

```rust
// src/types.rs
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_active: bool,
    pub is_modified: bool,
    pub tab_index: Option<usize>,
    pub project_name: Option<String>,
}

pub struct DetectionResult {
    pub timestamp: DateTime<Utc>,
    pub ide_name: String,
    pub ide_version: Option<String>,
    pub active_file: Option<String>,
    pub open_files: Vec<FileInfo>,
    pub project_path: Option<String>,
}
```

## Current Implementation Status

### ✅ Completed Features
- **Linux process detection** via `/proc` filesystem
- **JetBrains IDEs**: GoLand, PyCharm, IntelliJ IDEA, WebStorm, etc.
  - Window title parsing with regex patterns
  - Command line argument extraction
  - `.idea/workspace.xml` integration for recent files
- **Terminal Editors**: Vim, Nano detection
- **VSCode**: Basic process detection and workspace parsing
- **CLI Interface**: Multiple output formats (JSON, plain, paths)
- **Installation**: System-wide with `idf` alias and auto-completion
- **Build System**: Comprehensive Makefile with 20+ targets

### 🚧 In Progress / Next Steps
- **Enhanced Terminal Detection**: Neovim server/client, Emacs daemon
- **Container Support**: Docker IDE detection, VS Code remote server
- **Real-time Monitoring**: File system watchers, polling mode
- **Cross-platform**: Windows/macOS process APIs

## Environment Context

### Linux Command Line Environment
- **No desktop environment** - terminal-only development
- **Focus on terminal editors** and remote IDEs
- **SSH-based development** workflows
- **Container/Docker** integration priorities

### Development Workflow
```bash
# Daily development cycle
make build && make test     # Build and test
./target/debug/ide-files --auto --verbose  # Manual testing
make install               # System installation
idf --auto                 # Test installed version
git add . && git commit    # Commit when feature complete
```

## Key Implementation Details

### Process Detection (Linux)
```rust
// src/process.rs - Linux implementation
pub fn get_running_processes() -> Vec<ProcessInfo> {
    // Reads /proc/*/cmdline and /proc/*/stat
    // Extracts process name, command line, window titles
}
```

### JetBrains Detection Strategy
```rust
// src/detectors/jetbrains.rs
// 1. Process name matching (idea, goland, pycharm, etc.)
// 2. Window title parsing with regex patterns
// 3. Command line argument extraction for project paths
// 4. .idea/workspace.xml parsing for recent files
```

### File System Integration
- **Project root detection**: `.git`, `.idea`, `Cargo.toml`, `package.json`
- **Recent files**: IDE workspace files, session management
- **File modification detection**: Timestamp comparison, git status

## Testing Strategy

### Automated Testing
```bash
./scripts/test-installation.sh  # Comprehensive installation test
make test                       # Unit tests + test file creation
make ci                        # Full CI checks (check, test, clippy)
```

### Manual Testing Scenarios
```bash
# Test with actual editors running
vim /tmp/test.go               # Terminal: vim detection
code /tmp/project              # VSCode: process and workspace detection
# JetBrains IDEs require manual testing with actual installations
```

## Common Development Tasks

### Adding New IDE Support
1. **Create detector**: `src/detectors/new_ide.rs`
2. **Implement trait**: `IDEDetector` with process matching and file extraction
3. **Register detector**: Add to `src/detectors/mod.rs` and `src/main.rs`
4. **Add enum variant**: `SupportedIDE` in `src/types.rs`
5. **Update completion**: Add to shell completion lists
6. **Test thoroughly**: Manual testing with actual IDE

### Debugging Process Detection
```bash
# Debug commands
idf --debug-processes                    # List all processes
idf --ide=vim --verbose                 # Verbose detection output
ps aux | grep -E "(vim|code|idea)"      # Manual process verification
```

### Cross-Platform Development
```rust
// src/process.rs structure
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]  
mod windows;
#[cfg(target_os = "macos")]
mod macos;

// Platform-specific implementations
pub use linux::*;  // etc.
```

## Performance Considerations

### Current Optimizations
- **Efficient process scanning**: Single `/proc` traversal
- **Regex compilation**: Compiled once, reused
- **Memory usage**: Minimal allocations in hot paths

### Planned Optimizations
- **Process list caching**: Avoid repeated scans
- **Incremental updates**: Only scan changed processes
- **Parallel processing**: Multi-threaded process analysis

## Security & Privacy

### Current Security Model
- **Local-only operation**: No network communication
- **Minimal permissions**: Standard user process access
- **Safe process enumeration**: Error handling for permission denied

### Privacy Considerations
- **File path exposure**: Users should be aware of what's detected
- **Process information**: Only IDE-related processes analyzed
- **No data transmission**: All processing stays local

## Integration Points

### MCP Integration
```bash
# Designed for MCP context updates
idf --auto --format=json     # Structured output for MCP
idf --auto --active          # Only current active file
idf --format=paths           # Simple file path list
```

### Shell Integration
```bash
# Auto-completion working for both names
ide-files --ide <TAB>        # Full command name
idf --format <TAB>           # Short alias (recommended)
```

## Troubleshooting Common Issues

### Build Issues
```bash
# Missing dependencies (Linux)
sudo apt install libx11-dev pkg-config
make install-deps           # Automated dependency installation

# Rust not installed
make setup-rust             # Install Rust toolchain
```

### Detection Issues
```bash
# No IDEs detected
idf --debug-processes       # Check if processes are visible
ps aux | grep <ide-name>    # Verify IDE is actually running

# Permission issues
# Most issues are due to IDE not running or process name mismatch
```

### Installation Issues
```bash
# Completion not working
exec zsh                    # Restart shell to load completion
rm ~/.zcompdump*           # Clear zsh completion cache

# Binary not found after install
echo $PATH                 # Verify /usr/local/bin in PATH
which idf                  # Verify installation location
```

## Future Development Priorities

### High Priority (Linux Command Line Focus)
1. **Enhanced Neovim detection**: LSP server integration, session files
2. **Remote development**: SSH session detection, VS Code server
3. **Container support**: Docker IDE detection, dev containers
4. **Real-time monitoring**: File system watchers for live updates

### Medium Priority
1. **Configuration system**: User preferences, custom detection rules
2. **Performance optimization**: Caching, incremental updates
3. **Advanced filtering**: Project-based filtering, file type filters

### Low Priority
1. **GUI IDEs**: Desktop environment integration (when needed)
2. **Cross-platform**: Windows/macOS support
3. **Web dashboard**: Development insights, team monitoring

## Git Workflow

### Branch Strategy
- `master`: Stable releases
- Feature branches: `feature/new-ide-detector`
- Commit messages: Descriptive with Claude Code footer

### Commit Message Format
```
Add feature description

- Bullet points for changes
- Focus on what and why
- Include breaking changes

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Contributing Guidelines

### Code Style
- **Rust conventions**: `cargo fmt`, `cargo clippy`
- **Error handling**: Comprehensive error types, no unwrap() in production
- **Documentation**: Inline docs for all public APIs
- **Testing**: Unit tests for new detectors, integration tests for workflows

### Pull Request Process
1. **Feature branch**: Create from master
2. **Implementation**: Follow existing patterns
3. **Testing**: Comprehensive testing on target platform
4. **Documentation**: Update README, TODO, this file
5. **Review**: Submit PR with detailed description

---

## Quick Reference

### Most Used Commands
```bash
make build && idf --auto    # Build and test
make install               # Install system-wide
idf --list-ides           # Show supported IDEs
idf --ide=vim --verbose   # Debug specific IDE
./scripts/test-installation.sh  # Full installation test
```

### Key Files to Modify
- **New IDE detector**: `src/detectors/new_ide.rs`
- **Add CLI option**: `src/main.rs` (clap configuration)
- **Data structures**: `src/types.rs`
- **Build system**: `Makefile`
- **Documentation**: `README.md`, `TODO.md`, this file

### Development Environment
- **Platform**: Linux command line (no desktop)
- **Focus**: Terminal editors, remote development, containers
- **Testing**: Automated scripts + manual IDE testing
- **Installation**: System-wide with auto-completion

---

*Last updated: 2025-06-19*
*Environment: Linux terminal, no desktop GUI*
*Status: Phase 1 complete, Linux command-line optimizations in progress*