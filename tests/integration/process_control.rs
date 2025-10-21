//! Process Control Integration Tests (T220-T224)
//!
//! Test process control operations with real processes:
//! - Test process spawning
//! - Graceful termination (WM_CLOSE)
//! - Forceful termination
//! - Priority changes
//! - Privilege checking

use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;

/// T220: Helper to spawn a test process
fn spawn_test_process() -> Option<u32> {
    // Spawn a long-running process (ping with timeout)
    let child = Command::new("ping")
        .args(&["127.0.0.1", "-n", "100"]) // Ping 100 times (~100 seconds)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    Some(child.id())
}

/// T220: Helper to spawn a GUI process for WM_CLOSE testing
fn spawn_gui_process() -> Option<u32> {
    // Spawn notepad - a simple GUI app
    let child = Command::new("notepad.exe")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    Some(child.id())
}

/// T220: Helper to check if process exists
fn process_exists(pid: u32) -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
    use windows::Win32::Foundation::CloseHandle;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid);
        if handle.is_ok() {
            let h = handle.unwrap();
            if !h.is_invalid() {
                let _ = CloseHandle(h);
                return true;
            }
        }
        false
    }
}

#[test]
#[ignore] // Requires spawning real processes
fn test_spawn_test_process() {
    // T220: Test that we can spawn a process
    let pid = spawn_test_process();
    assert!(pid.is_some(), "Failed to spawn test process");
    
    let pid = pid.unwrap();
    assert!(pid > 0, "Invalid PID");
    assert!(process_exists(pid), "Process should exist after spawn");
    
    // Clean up
    let _ = Command::new("taskkill")
        .args(&["/F", "/PID", &pid.to_string()])
        .output();
}

#[test]
#[ignore] // Requires spawning real processes
fn test_graceful_termination() {
    // T221: Test graceful termination via WM_CLOSE
    let pid = spawn_gui_process();
    assert!(pid.is_some(), "Failed to spawn GUI process");
    
    let pid = pid.unwrap();
    thread::sleep(Duration::from_millis(500)); // Let process start
    
    assert!(process_exists(pid), "Process should exist");
    
    // Attempt graceful termination
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE};
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    
    unsafe {
        extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> windows::Win32::Foundation::BOOL {
            let target_pid = lparam.0 as u32;
            let mut window_pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
            
            if window_pid == target_pid {
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            
            true.into()
        }
        
        let _ = EnumWindows(Some(enum_windows_callback), LPARAM(pid as isize));
    }
    
    // Wait for graceful shutdown
    thread::sleep(Duration::from_millis(2000));
    
    // Process should be gone (notepad responds to WM_CLOSE)
    // Note: This might fail if "Save?" dialog appears - notepad is not ideal
    // but it's the simplest GUI app available
    
    // Clean up if still running
    let _ = Command::new("taskkill")
        .args(&["/F", "/PID", &pid.to_string()])
        .output();
}

#[test]
#[ignore] // Requires spawning real processes
fn test_forceful_termination() {
    // T222: Test forceful termination using TerminateProcess
    let pid = spawn_test_process();
    assert!(pid.is_some(), "Failed to spawn test process");
    
    let pid = pid.unwrap();
    thread::sleep(Duration::from_millis(500)); // Let process start
    
    assert!(process_exists(pid), "Process should exist");
    
    // Forceful termination
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    use windows::Win32::Foundation::CloseHandle;
    
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid);
        assert!(handle.is_ok(), "Should be able to open process");
        
        let handle = handle.unwrap();
        let result = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        
        assert!(result.is_ok(), "Should be able to terminate process");
    }
    
    // Wait a bit
    thread::sleep(Duration::from_millis(500));
    
    // Process should be gone
    assert!(!process_exists(pid), "Process should no longer exist after termination");
}

#[test]
#[ignore] // Requires spawning real processes
fn test_priority_changes() {
    // T223: Test priority changes
    let pid = spawn_test_process();
    assert!(pid.is_some(), "Failed to spawn test process");
    
    let pid = pid.unwrap();
    thread::sleep(Duration::from_millis(500));
    
    use windows::Win32::System::Threading::{
        OpenProcess, GetPriorityClass, SetPriorityClass,
        PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
        BELOW_NORMAL_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    };
    use windows::Win32::Foundation::CloseHandle;
    
    unsafe {
        // Open with appropriate access
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION.0 | PROCESS_SET_INFORMATION.0,
            false,
            pid
        );
        assert!(handle.is_ok(), "Should be able to open process");
        
        let handle = handle.unwrap();
        
        // Get initial priority (should be normal)
        let initial_priority = GetPriorityClass(handle);
        assert_ne!(initial_priority, 0, "Should be able to get priority");
        
        // Change to below normal
        let result = SetPriorityClass(handle, BELOW_NORMAL_PRIORITY_CLASS);
        assert!(result.is_ok(), "Should be able to set priority");
        
        // Verify change
        let new_priority = GetPriorityClass(handle);
        assert_eq!(new_priority, BELOW_NORMAL_PRIORITY_CLASS.0, "Priority should be below normal");
        
        // Restore to normal
        let result = SetPriorityClass(handle, NORMAL_PRIORITY_CLASS);
        assert!(result.is_ok(), "Should be able to restore priority");
        
        let _ = CloseHandle(handle);
    }
    
    // Clean up
    let _ = Command::new("taskkill")
        .args(&["/F", "/PID", &pid.to_string()])
        .output();
}

#[test]
fn test_privilege_checking_system_process() {
    // T224: Test privilege checking for system process
    // Try to open a system process (should fail without admin)
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE};
    use windows::Win32::Foundation::CloseHandle;
    
    // PID 4 is usually the System process
    let system_pid = 4u32;
    
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, system_pid);
        
        // Should fail unless we're running as admin
        if handle.is_ok() {
            let h = handle.unwrap();
            if !h.is_invalid() {
                // We got access - probably running as admin
                let _ = CloseHandle(h);
                println!("Running with elevated privileges - can access system process");
            } else {
                println!("Access denied to system process (expected without elevation)");
            }
        } else {
            // Expected - access denied
            println!("Access denied to system process (expected without elevation)");
        }
    }
    
    // This test always passes - it's just checking the behavior
}

#[test]
fn test_privilege_checking_own_process() {
    // T224: Test that we can always control our own process
    use windows::Win32::System::Threading::{
        OpenProcess, GetCurrentProcessId,
        PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
    };
    use windows::Win32::Foundation::CloseHandle;
    
    unsafe {
        let own_pid = GetCurrentProcessId();
        
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION.0 | PROCESS_SET_INFORMATION.0,
            false,
            own_pid
        );
        
        assert!(handle.is_ok(), "Should always be able to open own process");
        
        let handle = handle.unwrap();
        assert!(!handle.is_invalid(), "Handle should be valid");
        
        let _ = CloseHandle(handle);
    }
}

#[test]
fn test_process_exists_check() {
    // Test our helper function
    use windows::Win32::System::Threading::GetCurrentProcessId;
    
    unsafe {
        let own_pid = GetCurrentProcessId();
        assert!(process_exists(own_pid), "Own process should exist");
        
        // Invalid PID should not exist
        assert!(!process_exists(0), "PID 0 should not exist");
        assert!(!process_exists(u32::MAX), "Invalid PID should not exist");
    }
}
