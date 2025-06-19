# Makefile for ide-files project

# Variables
PROJECT_NAME = ide-files
TARGET_DIR = target
DEBUG_BIN = $(TARGET_DIR)/debug/$(PROJECT_NAME)
RELEASE_BIN = $(TARGET_DIR)/release/$(PROJECT_NAME)
TEST_DIR = /tmp/ide-test

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Platform-specific variables
ifeq ($(UNAME_S),Linux)
    PLATFORM = linux
    ARCHIVE_EXT = tar.gz
endif
ifeq ($(UNAME_S),Darwin)
    PLATFORM = macos
    ARCHIVE_EXT = tar.gz
endif
ifeq ($(OS),Windows_NT)
    PLATFORM = windows
    ARCHIVE_EXT = zip
    PROJECT_NAME := $(PROJECT_NAME).exe
    DEBUG_BIN := $(DEBUG_BIN).exe
    RELEASE_BIN := $(RELEASE_BIN).exe
endif

# Ensure cargo is in PATH
export PATH := $(HOME)/.cargo/bin:$(PATH)

# Default target
.PHONY: all
all: build

# Build targets
.PHONY: build
build: ## Build debug version
	@echo "Building $(PROJECT_NAME) in debug mode..."
	cargo build

.PHONY: release
release: ## Build optimized release version
	@echo "Building $(PROJECT_NAME) in release mode..."
	cargo build --release

.PHONY: check
check: ## Check code without building
	@echo "Checking code..."
	cargo check

# Development targets
.PHONY: test
test: build ## Run tests and create test files
	@echo "Creating test environment..."
	@mkdir -p $(TEST_DIR)
	@echo "console.log('Hello from test file');" > $(TEST_DIR)/test.js
	@echo "print('Hello from Python test')" > $(TEST_DIR)/test.py
	@echo "package main\n\nfunc main() {\n    println(\"Hello from Go test\")\n}" > $(TEST_DIR)/test.go
	@echo "Test files created in $(TEST_DIR)/"
	@echo "Running unit tests..."
	cargo test

.PHONY: run
run: build ## Run the application
	./$(DEBUG_BIN)

.PHONY: demo
demo: build ## Run demo commands
	@echo "=== IDE Files Detection Demo ==="
	@echo "Listing supported IDEs:"
	./$(DEBUG_BIN) --list-ides
	@echo ""
	@echo "Auto-detecting running IDEs:"
	./$(DEBUG_BIN) --auto --verbose || echo "No IDEs detected"
	@echo ""
	@echo "Checking for vim specifically:"
	./$(DEBUG_BIN) --ide=vim --format=plain || echo "No vim processes found"

# Utility targets
.PHONY: clean
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf $(TEST_DIR)

.PHONY: fmt
fmt: ## Format code
	@echo "Formatting code..."
	cargo fmt

.PHONY: clippy
clippy: ## Run clippy linter
	@echo "Running clippy..."
	cargo clippy -- -D warnings

.PHONY: fix
fix: ## Auto-fix code issues
	@echo "Auto-fixing code issues..."
	cargo fix --allow-dirty --allow-staged
	cargo fmt

# Install targets
.PHONY: install
install: release ## Install to system
	@echo "Installing $(PROJECT_NAME)..."
	cargo install --path .

.PHONY: install-deps
install-deps: ## Install system dependencies
	@echo "Installing system dependencies..."
	@if [ -f /etc/debian_version ]; then \
		sudo apt update && sudo apt install -y libx11-dev pkg-config; \
	elif [ -f /etc/redhat-release ]; then \
		sudo dnf install -y libX11-devel pkgconf-pkg-config; \
	elif [ -f /etc/arch-release ]; then \
		sudo pacman -S libx11 pkgconf; \
	else \
		echo "Please install libx11-dev and pkg-config manually for your system"; \
	fi

# Development workflow
.PHONY: dev
dev: install-deps build test clippy ## Complete development setup and checks

.PHONY: ci
ci: check test clippy ## Continuous integration checks

# Documentation
.PHONY: doc
doc: ## Generate documentation
	@echo "Generating documentation..."
	cargo doc --open

# File size and info
.PHONY: size
size: release ## Show binary size
	@echo "Binary sizes:"
	@ls -lh $(DEBUG_BIN) $(RELEASE_BIN) 2>/dev/null || echo "Build binaries first"

# Usage examples
.PHONY: examples
examples: build ## Show usage examples
	@echo "=== Usage Examples ==="
	@echo "1. List all supported IDEs:"
	@echo "   ./$(DEBUG_BIN) --list-ides"
	@echo ""
	@echo "2. Auto-detect any running IDE:"
	@echo "   ./$(DEBUG_BIN) --auto --verbose"
	@echo ""
	@echo "3. Check specific IDE (vim):"
	@echo "   ./$(DEBUG_BIN) --ide=vim --format=json"
	@echo ""
	@echo "4. Get only file paths:"
	@echo "   ./$(DEBUG_BIN) --auto --format=paths"
	@echo ""
	@echo "5. Get only active file:"
	@echo "   ./$(DEBUG_BIN) --auto --active --format=paths"
	@echo ""
	@echo "6. Debug mode (list all processes):"
	@echo "   ./$(DEBUG_BIN) --debug-processes"

# Debug targets
.PHONY: debug-processes
debug-processes: build ## Show all running processes
	./$(DEBUG_BIN) --debug-processes

.PHONY: debug-vim
debug-vim: build ## Debug vim detection specifically
	@echo "Checking for vim processes in system:"
	@ps aux | grep -E "(vim|nvim)" | grep -v grep || echo "No vim processes found"
	@echo ""
	@echo "Running vim detection:"
	./$(DEBUG_BIN) --ide=vim --verbose || echo "Vim detection failed"

# Help target
.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

# Rust setup (for new environments)
.PHONY: setup-rust
setup-rust: ## Install Rust if not present
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "Please run: source ~/.cargo/env"; \
	else \
		echo "Rust is already installed"; \
		cargo --version; \
	fi

# Package targets
.PHONY: package
package: release ## Create distributable package
	@echo "Creating package for $(PLATFORM)-$(UNAME_M)..."
	@mkdir -p dist
	@cp $(RELEASE_BIN) dist/
	@cp README.md dist/ 2>/dev/null || echo "# $(PROJECT_NAME)" > dist/README.md
	@cp SETUP.md dist/ 2>/dev/null || echo "See repository for setup instructions" > dist/SETUP.md
ifeq ($(PLATFORM),windows)
	@cd dist && zip $(PROJECT_NAME)-$(PLATFORM)-$(UNAME_M).zip $(PROJECT_NAME) README.md SETUP.md
	@echo "Package created: dist/$(PROJECT_NAME)-$(PLATFORM)-$(UNAME_M).zip"
else
	@tar -czf dist/$(PROJECT_NAME)-$(PLATFORM)-$(UNAME_M).tar.gz -C dist $(PROJECT_NAME) README.md SETUP.md
	@echo "Package created: dist/$(PROJECT_NAME)-$(PLATFORM)-$(UNAME_M).tar.gz"
endif

# Watch for changes (requires cargo-watch)
.PHONY: watch
watch: ## Watch for changes and rebuild (requires cargo install cargo-watch)
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x build; \
	else \
		echo "Install cargo-watch first: cargo install cargo-watch"; \
	fi