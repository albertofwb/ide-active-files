use crate::types::ProcessInfo;
use crate::detector::DetectionResult;

pub fn find_all_processes() -> DetectionResult<Vec<ProcessInfo>> {
    #[cfg(target_os = "windows")]
    return find_processes_windows();
    
    #[cfg(target_os = "macos")]
    return find_processes_macos();
    
    #[cfg(target_os = "linux")]
    return find_processes_linux();
}

#[cfg(target_os = "windows")]
fn find_processes_windows() -> DetectionResult<Vec<ProcessInfo>> {
    use winapi::um::tlhelp32::*;
    use winapi::um::handleapi::*;
    use winapi::shared::minwindef::*;
    use std::ffi::CStr;
    use std::mem;
    use crate::detector::DetectionError;

    let mut processes = Vec::new();
    
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(DetectionError::SystemError {
                message: "Failed to create process snapshot".to_string()
            });
        }

        let mut entry: PROCESSENTRY32 = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut entry) == TRUE {
            loop {
                let process_name = CStr::from_ptr(entry.szExeFile.as_ptr())
                    .to_string_lossy()
                    .to_string();
                
                let window_title = get_window_title_by_pid(entry.th32ProcessID);
                
                processes.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    name: process_name,
                    window_title,
                    executable_path: String::new(), // TODO: Get full path
                });

                if Process32Next(snapshot, &mut entry) != TRUE {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }
    
    Ok(processes)
}

#[cfg(target_os = "windows")]
fn get_window_title_by_pid(pid: u32) -> String {
    use winapi::um::winuser::*;
    use std::mem;
    use std::ptr;

    unsafe {
        let mut window_title = String::new();
        
        EnumWindows(Some(enum_windows_proc), &mut window_title as *mut String as isize);
        
        window_title
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_proc(hwnd: winapi::shared::windef::HWND, lparam: isize) -> i32 {
    // Implement window enumeration logic
    1
}

#[cfg(target_os = "macos")]
fn find_processes_macos() -> DetectionResult<Vec<ProcessInfo>> {
    // TODO: Implement macOS version
    Ok(vec![])
}

#[cfg(target_os = "linux")]
fn find_processes_linux() -> DetectionResult<Vec<ProcessInfo>> {
    use std::fs;
    use std::path::Path;
    use crate::detector::DetectionError;
    
    let mut processes = Vec::new();
    
    // Read all entries in /proc
    let proc_dir = Path::new("/proc");
    let entries = fs::read_dir(proc_dir)
        .map_err(|e| DetectionError::SystemError {
            message: format!("Failed to read /proc: {}", e)
        })?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            // Check if entry is a PID directory
            if let Some(pid_str) = path.file_name().and_then(|n| n.to_str()) {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // Read process info
                    let cmdline_path = path.join("cmdline");
                    let comm_path = path.join("comm");
                    
                    if let Ok(comm) = fs::read_to_string(&comm_path) {
                        let name = comm.trim().to_string();
                        
                        // Get command line for window title (simplified)
                        let window_title = fs::read_to_string(&cmdline_path)
                            .unwrap_or_default()
                            .replace('\0', " ")
                            .trim()
                            .to_string();
                        
                        let executable_path = fs::read_link(path.join("exe"))
                            .ok()
                            .and_then(|p| p.to_str().map(|s| s.to_string()))
                            .unwrap_or_default();
                        
                        processes.push(ProcessInfo {
                            pid,
                            name,
                            window_title,
                            executable_path,
                        });
                    }
                }
            }
        }
    }
    
    Ok(processes)
}

pub fn find_processes_by_name(name: &str) -> DetectionResult<Vec<ProcessInfo>> {
    let all_processes = find_all_processes()?;
    
    Ok(all_processes.into_iter()
        .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
        .collect())
}

pub fn list_all_processes() -> DetectionResult<()> {
    let processes = find_all_processes()?;
    
    println!("All running processes:");
    for process in processes {
        if !process.name.is_empty() {
            println!("PID: {}, Name: {}, Title: {}", 
                process.pid, process.name, 
                if process.window_title.is_empty() { "<no title>" } else { &process.window_title });
        }
    }
    
    Ok(())
}