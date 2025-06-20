# VSCode Detection Improvements

## Fixed Issues

### 1. Extension Files Detection Bug
**Problem**: VSCode detector was returning cached extension files (`.js` files from `.vscode/extensions/`) instead of actual open files.

**Root Cause**: 
- VSCode spawns extension processes with file paths in command line arguments
- The detector was incorrectly treating these as user-opened files
- Loop index bug caused infinite loop when skipping files with `continue`

**Solution**:
- Added file path filtering to exclude:
  - `/.vscode/extensions/` paths
  - `/resources/app/extensions/` paths
  - `/CachedExtension` paths
  - `node_modules` paths
  - Server and bundle JS files
- Fixed loop index increment when using `continue`

### 2. Project Path Detection
**Problem**: `project_path` was always null even when VSCode opened a folder/workspace.

**Root Cause**: VSCode doesn't keep workspace path in command line after startup.

**Solution**:
- Extract workspace path from `workspace.json` in VSCode's storage directory
- Enhanced `get_vscode_session_files()` to return both files and workspace path
- Properly detect workspace mode vs direct file mode

## VSCode Detection Modes

### 1. Folder/Workspace Mode
When VSCode opens a folder: `code /path/to/project`
- Command line shows only `/usr/share/code/code` after startup
- Files are retrieved from SQLite database (`state.vscdb`)
- Project path is extracted from `workspace.json`

### 2. Direct File Mode
When VSCode opens files directly: `code file1.py file2.js`
- Files are extracted from command line arguments
- No workspace/project path is set
- Each file is treated independently

## Technical Implementation

### Key Changes:
1. **File Filtering**: Skip extension and internal files in `extract_vscode_info()`
2. **Workspace Detection**: Added `extract_workspace_from_json()` method
3. **Return Type Update**: `get_vscode_recent_files()` now returns `(Vec<FileInfo>, Option<String>)`
4. **Mode Detection**: Use `found_cmdline_files` flag to determine detection strategy

### Example Output:
```json
{
  "timestamp": "2025-06-20T23:27:37.612238366+00:00",
  "ide_name": "Visual Studio Code",
  "active_file": "/home/albert/codes/Claude-Code-Usage-Monitor/ccusage_monitor.py",
  "open_files": [
    {
      "path": "/home/albert/codes/Claude-Code-Usage-Monitor/ccusage_monitor.py",
      "name": "ccusage_monitor.py",
      "is_active": true
    },
    {
      "path": "/home/albert/codes/Claude-Code-Usage-Monitor/CONTRIBUTING.md",
      "name": "CONTRIBUTING.md",
      "is_active": true
    }
  ],
  "project_path": "/home/albert/codes/Claude-Code-Usage-Monitor"
}
```

## Testing

Use the test script to verify VSCode detection:
```bash
./test-vscode.sh
```

This script:
1. Shows current VSCode state (folder vs files mode)
2. Displays VSCode main process info
3. Shows command line arguments
4. Lists workspace storage directories
5. Runs detailed detection with verbose output