#!/bin/bash
# Test script for Linux installation and idf alias

set -e

echo "🧪 Testing IDE Files Installation and idf Alias"
echo "================================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "ℹ️  $1"
}

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    print_warning "This test is designed for Linux. Current OS: $OSTYPE"
fi

echo ""
echo "Step 1: Building the project..."
make build > /dev/null 2>&1
print_status $? "Project built successfully"

echo ""
echo "Step 2: Testing pre-installation functionality..."

# Test that the binary works before installation
./target/debug/ide-files --help > /dev/null 2>&1
print_status $? "Binary runs correctly"

./target/debug/ide-files --list-ides > /dev/null 2>&1
print_status $? "List IDEs command works"

echo ""
echo "Step 3: Testing installation (requires sudo)..."

# Check if we can sudo
if ! sudo -n true 2>/dev/null; then
    print_warning "This test requires sudo access for installation testing"
    echo "Please run: sudo -v"
    echo "Then re-run this test script"
    exit 1
fi

# Install
print_info "Installing to /usr/local/bin/..."
make install > /dev/null 2>&1
print_status $? "Installation completed"

echo ""
echo "Step 4: Testing installed binaries..."

# Test that both commands are available
if command -v ide-files > /dev/null 2>&1; then
    print_status 0 "ide-files command is available"
else
    print_status 1 "ide-files command not found"
fi

if command -v idf > /dev/null 2>&1; then
    print_status 0 "idf alias is available"
else
    print_status 1 "idf alias not found"
fi

# Test that both commands work
ide-files --help > /dev/null 2>&1
print_status $? "ide-files --help works"

idf --help > /dev/null 2>&1
print_status $? "idf --help works"

# Test that both commands are the same
IDE_VERSION=$(ide-files --version 2>&1 | head -1)
IDF_VERSION=$(idf --version 2>&1 | head -1)

if [ "$IDE_VERSION" = "$IDF_VERSION" ]; then
    print_status 0 "Both commands report same version"
else
    print_status 1 "Version mismatch between commands"
fi

echo ""
echo "Step 5: Testing functionality..."

# Test auto-detection
idf --auto > /dev/null 2>&1 || true  # May fail if no IDEs running
print_status 0 "idf --auto command executed (may not find IDEs)"

# Test list IDEs
IDE_COUNT=$(idf --list-ides | grep -c "(.*)") || 0
if [ $IDE_COUNT -gt 0 ]; then
    print_status 0 "idf --list-ides shows $IDE_COUNT IDEs"
else
    print_status 1 "idf --list-ides shows no IDEs"
fi

echo ""
echo "Step 6: Testing auto-completion setup..."

# Test completion script exists
if [ -f "scripts/setup-completion.sh" ]; then
    print_status 0 "Completion script exists"
else
    print_status 1 "Completion script missing"
fi

# Test completion script is executable
if [ -x "scripts/setup-completion.sh" ]; then
    print_status 0 "Completion script is executable"
else
    print_status 1 "Completion script is not executable"
fi

# Test dry-run of completion setup (if not in CI)
if [ -z "$CI" ]; then
    print_info "Testing completion setup (dry run)..."
    # We can't actually install completion in test, but we can check the script
    bash -n scripts/setup-completion.sh
    print_status $? "Completion script syntax is valid"
fi

echo ""
echo "Step 7: Testing file operations..."

# Create a test file and test detection
echo "test content" > /tmp/idf-test.txt
if vim --version > /dev/null 2>&1; then
    print_info "vim available for testing"
    
    # Start vim in background (will exit immediately)
    echo ":q" | vim /tmp/idf-test.txt > /dev/null 2>&1 || true
    
    # Test if our tool can detect files  
    idf --ide=vim --verbose > /dev/null 2>&1 || true
    print_status 0 "File detection test completed (may not find active vim)"
else
    print_warning "vim not available, skipping file detection test"
fi

# Clean up test file
rm -f /tmp/idf-test.txt

echo ""
echo "Step 8: Testing uninstallation..."

print_info "Uninstalling..."
make uninstall > /dev/null 2>&1
print_status $? "Uninstallation completed"

# Verify binaries are removed
if ! command -v ide-files > /dev/null 2>&1; then
    print_status 0 "ide-files command removed"
else
    print_status 1 "ide-files command still exists"
fi

if ! command -v idf > /dev/null 2>&1; then
    print_status 0 "idf alias removed"
else
    print_status 1 "idf alias still exists"
fi

echo ""
echo "🎉 Installation and idf alias test completed!"
echo ""
echo "Summary:"
echo "- Installation process: ✅"
echo "- idf alias creation: ✅" 
echo "- Both commands functional: ✅"
echo "- Auto-completion setup: ✅"
echo "- Uninstallation: ✅"
echo ""
echo "To install for real usage:"
echo "  make install"
echo "  make install-completion"
echo ""
echo "Then use either:"
echo "  idf --auto          # Short alias (recommended)"
echo "  ide-files --auto    # Full name"