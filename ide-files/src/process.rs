use crate::detector::DetectionResult;
use crate::types::ProcessInfo;

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
    use crate::detector::DetectionError;
    use std::ffi::CStr;
    use std::mem;
    use winapi::shared::minwindef::*;
    use winapi::um::handleapi::*;
    use winapi::um::tlhelp32::*;

    let mut processes = Vec::new();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(DetectionError::SystemError {
                message: "Failed to create process snapshot".to_string(),
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
    use std::mem;
    use std::ptr;
    use winapi::um::winuser::*;

    unsafe {
        let mut window_title = String::new();

        EnumWindows(
            Some(enum_windows_proc),
            &mut window_title as *mut String as isize,
        );

        window_title
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_proc(
    hwnd: winapi::shared::windef::HWND,
    lparam: isize,
) -> i32 {
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
    use crate::detector::DetectionError;
    use std::fs;
    use std::path::Path;

    let mut processes = Vec::new();
    
    // Get window titles from X11
    let window_titles = get_x11_window_titles();

    // Read all entries in /proc
    let proc_dir = Path::new("/proc");
    let entries = fs::read_dir(proc_dir).map_err(|e| DetectionError::SystemError {
        message: format!("Failed to read /proc: {}", e),
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

                        // Get window title from X11 if available, otherwise use cmdline
                        let window_title = window_titles.get(&pid)
                            .cloned()
                            .unwrap_or_else(|| {
                                fs::read_to_string(&cmdline_path)
                                    .unwrap_or_default()
                                    .replace('\0', " ")
                                    .trim()
                                    .to_string()
                            });

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

#[cfg(target_os = "linux")]
fn get_x11_window_titles() -> std::collections::HashMap<u32, String> {
    use std::collections::HashMap;
    use x11::xlib::*;
    use std::ffi::CString;
    use std::ffi::CStr;
    use std::ptr;
    
    let mut window_titles = HashMap::new();
    
    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return window_titles;
        }
        
        let root = XDefaultRootWindow(display);
        let mut root_return = 0;
        let mut parent = 0;
        let mut children: *mut Window = ptr::null_mut();
        let mut nchildren = 0;
        
        if XQueryTree(display, root, &mut root_return, &mut parent, &mut children, &mut nchildren) != 0 {
            for i in 0..nchildren {
                let window = *children.offset(i as isize);
                
                // Get window PID
                let atom_name = CString::new("_NET_WM_PID").unwrap();
                let atom = XInternAtom(display, atom_name.as_ptr(), 0);
                
                let mut actual_type = 0;
                let mut actual_format = 0;
                let mut nitems = 0;
                let mut bytes_after = 0;
                let mut prop: *mut u8 = ptr::null_mut();
                
                if XGetWindowProperty(
                    display,
                    window,
                    atom,
                    0,
                    1,
                    0,
                    AnyPropertyType as u64,
                    &mut actual_type,
                    &mut actual_format,
                    &mut nitems,
                    &mut bytes_after,
                    &mut prop
                ) == Success as i32 && !prop.is_null() {
                    let pid = *(prop as *const u32);
                    
                    // Get window name - try both WM_NAME and _NET_WM_NAME
                    let mut window_name: *mut i8 = ptr::null_mut();
                    let mut got_title = false;
                    
                    // First try _NET_WM_NAME (UTF-8)
                    let net_wm_name = CString::new("_NET_WM_NAME").unwrap();
                    let net_wm_name_atom = XInternAtom(display, net_wm_name.as_ptr(), 0);
                    let utf8_string = CString::new("UTF8_STRING").unwrap();
                    let utf8_atom = XInternAtom(display, utf8_string.as_ptr(), 0);
                    
                    let mut prop_name: *mut u8 = ptr::null_mut();
                    let mut actual_type_name = 0;
                    let mut actual_format_name = 0;
                    let mut nitems_name = 0;
                    let mut bytes_after_name = 0;
                    
                    if XGetWindowProperty(
                        display,
                        window,
                        net_wm_name_atom,
                        0,
                        8192,
                        0,
                        utf8_atom,
                        &mut actual_type_name,
                        &mut actual_format_name,
                        &mut nitems_name,
                        &mut bytes_after_name,
                        &mut prop_name
                    ) == Success as i32 && !prop_name.is_null() && nitems_name > 0 {
                        let title = String::from_utf8_lossy(std::slice::from_raw_parts(prop_name, nitems_name as usize)).to_string();
                        if !title.is_empty() {
                            window_titles.insert(pid, title);
                            got_title = true;
                        }
                        XFree(prop_name as *mut _);
                    }
                    
                    // Fall back to WM_NAME if needed
                    if !got_title && XFetchName(display, window, &mut window_name) != 0 && !window_name.is_null() {
                        let title = CStr::from_ptr(window_name).to_string_lossy().to_string();
                        if !title.is_empty() {
                            window_titles.insert(pid, title);
                        }
                        XFree(window_name as *mut _);
                    }
                    
                    XFree(prop as *mut _);
                }
            }
            
            if !children.is_null() {
                XFree(children as *mut _);
            }
        }
        
        XCloseDisplay(display);
    }
    
    window_titles
}

pub fn find_processes_by_name(name: &str) -> DetectionResult<Vec<ProcessInfo>> {
    let all_processes = find_all_processes()?;

    Ok(all_processes
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
        .collect())
}

pub fn list_all_processes() -> DetectionResult<()> {
    let processes = find_all_processes()?;

    println!("All running processes:");
    for process in processes {
        if !process.name.is_empty() {
            println!(
                "PID: {}, Name: {}, Title: {}",
                process.pid,
                process.name,
                if process.window_title.is_empty() {
                    "<no title>"
                } else {
                    &process.window_title
                }
            );
        }
    }

    Ok(())
}
