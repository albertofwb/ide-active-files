#!/bin/bash
# Test script for VSCode detection scenarios

echo "=== Testing VSCode Detection Scenarios ==="
echo

# Test 1: Current VSCode state (likely folder mode)
echo "1. Current VSCode state:"
./target/debug/ide-files --ide=vscode --format=json | jq '{
  mode: (if .project_path then "folder" else "files" end),
  active_file: .active_file,
  open_files_count: .open_files | length,
  project_path: .project_path
}'
echo

# Show VSCode process info
echo "2. VSCode main process:"
ps aux | grep "/usr/share/code/code" | grep -v "type=" | grep -v "extensions" | head -1
echo

# Show command line
PID=$(ps aux | grep "/usr/share/code/code" | grep -v "type=" | grep -v "extensions" | head -1 | awk '{print $2}')
if [ ! -z "$PID" ]; then
    echo "3. VSCode command line (PID $PID):"
    cat /proc/$PID/cmdline 2>/dev/null | tr '\0' ' ' && echo
    echo
fi

# Show workspace storage
echo "4. VSCode workspace storage:"
ls -lt ~/.config/Code/User/workspaceStorage/ | head -3
echo

# Test detection with verbose output
echo "5. Detailed detection result:"
./target/debug/ide-files --ide=vscode --verbose