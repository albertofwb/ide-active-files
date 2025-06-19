# TODO List

## Phase 1: Foundation ✅ COMPLETED
- [x] Project setup and basic structure
- [x] Strategy pattern implementation
- [x] Core data types and interfaces
- [x] Linux process detection via /proc
- [x] Terminal editor detection (Vim, Nano)
- [x] JetBrains IDE detection framework
- [x] CLI interface with multiple output formats
- [x] Build system (Makefile) with comprehensive targets
- [x] Documentation (README.md, SETUP.md)
- [x] **NEW: Fixed all build warnings and enhanced debugging**
- [x] **NEW: Visual Studio Code detection support**
- [x] **NEW: Enhanced JetBrains detection with .idea/ workspace parsing**
- [x] **NEW: System installation support (/usr/local/bin/)**
- [x] **NEW: Shell auto-completion for bash and zsh**

## Phase 2: Windows & macOS Support 🚧 IN PROGRESS
- [ ] Complete Windows process detection implementation
  - [ ] Enhance window title enumeration
  - [ ] Improve process command line extraction
  - [ ] Test with various JetBrains IDEs on Windows
- [ ] Complete macOS process detection implementation
  - [ ] Implement Cocoa/Core Foundation APIs
  - [ ] Test process detection and window title parsing
  - [ ] Handle macOS security permissions
- [ ] Cross-platform testing and validation

## Phase 3: Enhanced IDE Support 📋 PLANNED
- [ ] **Visual Studio Code**
  - [ ] Process detection for code.exe/Code.exe
  - [ ] Workspace file parsing (.vscode/settings.json)
  - [ ] Open editors detection via extension API or file system
  - [ ] Multi-workspace support
- [ ] **Visual Studio (Windows)**
  - [ ] Process detection for devenv.exe
  - [ ] Solution/project file parsing (.sln, .csproj)
  - [ ] COM interface integration if available
- [ ] **Sublime Text**
  - [ ] Process detection
  - [ ] Session file parsing
- [ ] **Atom** (if still needed)
  - [ ] Process detection
  - [ ] Project state detection
- [ ] **Emacs**
  - [ ] Process detection
  - [ ] Buffer list extraction via emacsclient

## Phase 4: Advanced Features 📋 PLANNED
- [ ] **Real-time monitoring**
  - [ ] File system watchers for IDE state changes
  - [ ] Polling mode with configurable intervals
  - [ ] Event-based updates
- [ ] **Enhanced file information**
  - [ ] File modification detection (unsaved changes)
  - [ ] Git status integration (modified, staged, etc.)
  - [ ] File type detection and categorization
  - [ ] Project root detection improvements
- [ ] **Configuration system**
  - [ ] User configuration file (~/.ide-files.toml)
  - [ ] Custom IDE detection rules
  - [ ] Output format templates
  - [ ] Process exclusion/inclusion filters

## Phase 5: Integration & Ecosystem 📋 PLANNED
- [ ] **MCP Integration**
  - [ ] MCP server implementation
  - [ ] Real-time context updates
  - [ ] WebSocket support for live updates
- [ ] **Plugin/Extension development**
  - [ ] VSCode extension for enhanced detection
  - [ ] JetBrains plugin for direct API access
  - [ ] Browser extension for web-based IDEs
- [ ] **API & Libraries**
  - [ ] REST API server mode
  - [ ] Language bindings (Python, Node.js)
  - [ ] WebAssembly build for browser usage

## Code Quality & Maintenance 🔧 ONGOING
- [ ] **Testing**
  - [ ] Unit tests for all detectors
  - [ ] Integration tests with real IDEs
  - [ ] Cross-platform test automation
  - [ ] Performance benchmarks
- [ ] **Code improvements**
  - [ ] Remove remaining dead code warnings
  - [ ] Add comprehensive error handling
  - [ ] Improve regex patterns for window title parsing
  - [ ] Add logging framework for debugging
- [ ] **Documentation**
  - [ ] API documentation generation
  - [ ] Video tutorials for setup
  - [ ] Architecture decision records (ADRs)
  - [ ] Contributing guidelines

## Bug Fixes 🐛 ONGOING
- [ ] Fix unused code warnings
  - [ ] Remove or use `list_supported_ides` method
  - [ ] Remove or use `find_processes_by_name` function
  - [ ] Remove or use `executable_path` field
- [ ] Windows-specific issues
  - [ ] Handle Unicode in window titles
  - [ ] Process elevation permissions
  - [ ] Long path support
- [ ] macOS-specific issues
  - [ ] Handle sandboxed applications
  - [ ] Notarization requirements
  - [ ] Apple Silicon compatibility

## Platform-Specific Tasks

### Linux 🐧
- [ ] Support for Wayland sessions
- [ ] Snap/Flatpak application detection
- [ ] AppImage application detection
- [ ] SELinux compatibility testing

### macOS 🍎
- [ ] Handle System Integrity Protection (SIP)
- [ ] Code signing and notarization
- [ ] Universal binary support (x86_64 + ARM64)
- [ ] Privacy permission handling

### Windows 🪟
- [ ] Windows Store app detection
- [ ] UAC permission handling
- [ ] Windows Terminal integration
- [ ] PowerShell module

## Performance Optimizations ⚡
- [ ] **Process scanning optimization**
  - [ ] Cache process list between calls
  - [ ] Incremental updates instead of full scans
  - [ ] Parallel process analysis
- [ ] **Memory usage optimization**
  - [ ] Reduce allocations in hot paths
  - [ ] Streaming JSON output for large results
  - [ ] Configurable buffer sizes
- [ ] **Startup time optimization**
  - [ ] Lazy detector initialization
  - [ ] Binary size reduction
  - [ ] Static linking optimizations

## Security Considerations 🔒
- [ ] **Process access security**
  - [ ] Minimal permission requirements
  - [ ] Safe process enumeration
  - [ ] Input validation for all user data
- [ ] **Data privacy**
  - [ ] Option to exclude sensitive file paths
  - [ ] No network communication without consent
  - [ ] Local-only operation by default

## Distribution & Packaging 📦
- [ ] **Package managers**
  - [ ] Homebrew formula (macOS/Linux)
  - [ ] Chocolatey package (Windows)
  - [ ] APT repository (Debian/Ubuntu)
  - [ ] AUR package (Arch Linux)
  - [ ] Cargo crates.io publication
- [ ] **Release automation**
  - [ ] GitHub Actions CI/CD
  - [ ] Automated testing on all platforms
  - [ ] Automated release generation
  - [ ] Binary signing and verification

## Future Ideas 💡
- [ ] **Web dashboard**
  - [ ] Real-time IDE monitoring dashboard
  - [ ] Team development insights
  - [ ] Project activity visualization
- [ ] **AI integration**
  - [ ] Smart project detection
  - [ ] Code context analysis
  - [ ] Development pattern recognition
- [ ] **Cloud integration**
  - [ ] Remote development environment support
  - [ ] Cloud IDE detection (GitPod, CodeSpaces)
  - [ ] Synchronized context across devices

## Community & Ecosystem 👥
- [ ] **Community building**
  - [ ] Discord/Slack community
  - [ ] Regular contributor meetings
  - [ ] Hackathon organization
- [ ] **Education & Outreach**
  - [ ] Blog posts about architecture
  - [ ] Conference talks and presentations
  - [ ] Open source mentorship program

---

## Priority Levels
- 🔥 **Critical**: Core functionality, security issues
- 🚀 **High**: Major features, performance improvements  
- 📋 **Medium**: Nice-to-have features, quality improvements
- 💡 **Low**: Future ideas, experimental features

## Contributing
See individual issues for specific tasks. Each TODO item should eventually become a GitHub issue with detailed requirements and acceptance criteria.

Last updated: 2025-06-19