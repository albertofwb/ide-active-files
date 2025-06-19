# Environment Setup Guide

This guide will help you set up your development environment for the IDE Files Detection Tool on Windows, Linux, and macOS.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Linux Setup](#linux-setup)
- [macOS Setup](#macos-setup)
- [Windows Setup](#windows-setup)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)
- [IDE Integration](#ide-integration)

## Prerequisites

The IDE Files Detection Tool requires:

- **Rust** (1.70.0 or later)
- **System development libraries** (platform-specific)
- **Git** (for version control)
- **Make** (for build automation)

## Linux Setup

### Ubuntu/Debian

1. **Update package manager:**
   ```bash
   sudo apt update
   ```

2. **Install system dependencies:**
   ```bash
   sudo apt install -y \
     curl \
     build-essential \
     pkg-config \
     libx11-dev \
     git \
     make
   ```

3. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

4. **Verify installation:**
   ```bash
   rustc --version
   cargo --version
   ```

### Red Hat/CentOS/Fedora

1. **Install system dependencies:**
   ```bash
   # For RHEL/CentOS 8+
   sudo dnf install -y \
     curl \
     gcc \
     gcc-c++ \
     make \
     pkgconf-pkg-config \
     libX11-devel \
     git

   # For older versions with yum
   sudo yum groupinstall -y "Development Tools"
   sudo yum install -y \
     curl \
     pkgconfig \
     libX11-devel \
     git
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Arch Linux

1. **Install system dependencies:**
   ```bash
   sudo pacman -S \
     curl \
     base-devel \
     pkgconf \
     libx11 \
     git \
     make
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Alpine Linux

1. **Install system dependencies:**
   ```bash
   sudo apk add \
     curl \
     build-base \
     pkgconfig \
     libx11-dev \
     git \
     make
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

## macOS Setup

### Using Homebrew (Recommended)

1. **Install Homebrew** (if not already installed):
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

2. **Install system dependencies:**
   ```bash
   brew install \
     pkg-config \
     git \
     make
   ```

3. **Install Xcode Command Line Tools:**
   ```bash
   xcode-select --install
   ```

4. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Using MacPorts

1. **Install MacPorts dependencies:**
   ```bash
   sudo port install \
     pkgconfig \
     git \
     gmake
   ```

2. **Install Xcode Command Line Tools:**
   ```bash
   xcode-select --install
   ```

3. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Manual Installation

1. **Install Xcode Command Line Tools:**
   ```bash
   xcode-select --install
   ```

2. **Download and install Git:**
   - Download from: https://git-scm.com/download/mac
   - Or use the built-in git with Xcode tools

3. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

## Windows Setup

### Using Visual Studio (Recommended)

1. **Install Visual Studio Community** (free):
   - Download from: https://visualstudio.microsoft.com/downloads/
   - During installation, select "Desktop development with C++"
   - This includes the MSVC compiler and Windows SDK

2. **Install Git for Windows:**
   - Download from: https://git-scm.com/download/win
   - This includes Git Bash and make

3. **Install Rust:**
   - Download from: https://rustup.rs/
   - Run the installer and follow prompts
   - Choose "1) Proceed with installation (default)"

4. **Restart your command prompt** and verify:
   ```cmd
   rustc --version
   cargo --version
   ```

### Using MSYS2/MinGW

1. **Install MSYS2:**
   - Download from: https://www.msys2.org/
   - Follow installation instructions

2. **Open MSYS2 terminal and install dependencies:**
   ```bash
   pacman -S \
     mingw-w64-x86_64-gcc \
     mingw-w64-x86_64-pkg-config \
     mingw-w64-x86_64-make \
     git
   ```

3. **Add MSYS2 to PATH:**
   - Add `C:\msys64\mingw64\bin` to your system PATH
   - Add `C:\msys64\usr\bin` to your system PATH

4. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Using WSL (Windows Subsystem for Linux)

1. **Install WSL2:**
   ```powershell
   wsl --install
   ```

2. **Install Ubuntu (or preferred distro):**
   ```powershell
   wsl --install -d Ubuntu
   ```

3. **Follow Linux setup instructions** inside WSL

## Building the Project

### Using Make (All Platforms)

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd ide-files
   ```

2. **Auto-install dependencies** (if not done manually):
   ```bash
   make install-deps
   ```

3. **Build the project:**
   ```bash
   # Debug build
   make build

   # Release build
   make release
   ```

4. **Run tests:**
   ```bash
   make test
   ```

### Manual Build

If make is not available:

```bash
# Install dependencies and build
cargo build

# Run the application
./target/debug/ide-files --help

# Run tests
cargo test
```

## Verification

Test your installation with these commands:

```bash
# Check Rust installation
rustc --version
cargo --version

# Check git
git --version

# Check make (if applicable)
make --version

# Test building the project
cd ide-files
make build

# Test running the application
./target/debug/ide-files --list-ides

# Test IDE detection
./target/debug/ide-files --auto --verbose
```

Expected output for `--list-ides`:
```
Supported IDEs:
  GoLand (--ide=goland)
  PyCharm (--ide=pycharm)
  IntelliJ IDEA (--ide=idea)
  Visual Studio Code (--ide=vscode)
  Visual Studio (--ide=vs)
  WebStorm (--ide=webstorm)
  PhpStorm (--ide=phpstorm)
  RubyMine (--ide=rubymine)
  CLion (--ide=clion)
  Vim (--ide=vim)
  Nano (--ide=nano)
```

## Troubleshooting

### Common Issues

#### Rust Not Found
```bash
# Make sure Rust is in your PATH
source ~/.cargo/env

# Or add to your shell profile
echo 'source ~/.cargo/env' >> ~/.bashrc
```

#### X11 Libraries Missing (Linux)
```bash
# Ubuntu/Debian
sudo apt install libx11-dev

# Red Hat/Fedora
sudo dnf install libX11-devel

# Arch Linux
sudo pacman -S libx11
```

#### MSVC Not Found (Windows)
- Install Visual Studio with C++ development tools
- Or install Build Tools for Visual Studio 2022

#### Permission Denied (macOS)
```bash
# If you get permission errors, try:
sudo xcode-select --reset
```

### Build Failures

#### Linker Errors
```bash
# Make sure you have a C compiler installed
gcc --version  # Linux/macOS
cl            # Windows with MSVC
```

#### Package Config Errors
```bash
# Install pkg-config
# Linux: Already covered in setup
# macOS: brew install pkg-config
# Windows: Use MSYS2 or VCPKG
```

### Platform-Specific Notes

#### Linux
- Some distributions may require additional development packages
- Use your distribution's package manager for system dependencies
- SELinux may interfere with process detection in some configurations

#### macOS
- Xcode Command Line Tools are required even with Homebrew
- Apple Silicon (M1/M2) Macs should work without special configuration
- Older macOS versions may need manual library installations

#### Windows
- Windows Defender may flag the binary during development
- Use "Windows Terminal" or "Git Bash" for better command-line experience
- WSL2 is recommended for the most Unix-like experience

## IDE Integration

### Visual Studio Code

1. **Install Rust Extension:**
   - Install "rust-analyzer" extension
   - Configure: `"rust-analyzer.cargo.features": "all"`

2. **Add to workspace settings** (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

3. **Add build tasks** (`.vscode/tasks.json`):
   ```json
   {
     "version": "2.0.0",
     "tasks": [
       {
         "label": "cargo build",
         "type": "shell",
         "command": "cargo",
         "args": ["build"],
         "group": "build"
       },
       {
         "label": "make demo",
         "type": "shell", 
         "command": "make",
         "args": ["demo"],
         "group": "test"
       }
     ]
   }
   ```

### JetBrains IDEs (IntelliJ IDEA, CLion, etc.)

1. **Install Rust Plugin:**
   - Go to File → Settings → Plugins
   - Search for "Rust" and install

2. **Configure project:**
   - Open the `ide-files` directory as a project
   - The IDE should automatically detect the Cargo.toml

### Vim/Neovim

1. **Install rust.vim:**
   ```vim
   " Add to your .vimrc
   Plug 'rust-lang/rust.vim'
   ```

2. **Add language server support:**
   ```vim
   " For rust-analyzer with coc.nvim
   Plug 'neoclide/coc.nvim'
   ```

## Quick Start Summary

For impatient developers, here's the fastest path:

```bash
# Linux (Ubuntu/Debian)
sudo apt update && sudo apt install -y curl build-essential pkg-config libx11-dev git make
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source ~/.cargo/env

# macOS (with Homebrew)
brew install pkg-config git make && xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source ~/.cargo/env

# Windows (download and install manually)
# 1. Install Visual Studio Community with C++ tools
# 2. Install Git from git-scm.com
# 3. Install Rust from rustup.rs

# Build and test
git clone <repo-url> && cd ide-files
make dev  # Complete development setup
make demo # Test the application
```

## Getting Help

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section
2. Verify your environment with `make ci`
3. Open an issue with your platform details and error messages
4. Include output of:
   ```bash
   rustc --version
   cargo --version
   uname -a  # Linux/macOS
   # or
   systeminfo | findstr "OS"  # Windows
   ```

---

**Happy Coding!** 🦀