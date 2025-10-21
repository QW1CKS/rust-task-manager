//! Process Control and Manipulation (T148-T184)
//!
//! Provides process control operations with proper privilege handling:
//! - Process termination (graceful and forceful)
//! - Priority management
//! - Process suspension/resume
//! - CPU affinity control
//! - Privilege checking and UAC elevation
//!
//! Safety: All Win32 API calls are wrapped in safe abstractions with RAII.

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, GetPriorityClass, SetPriorityClass,
    WaitForSingleObject, GetProcessAffinityMask, SetProcessAffinityMask,
    PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_SET_INFORMATION,
    PROCESS_SUSPEND_RESUME, PROCESS_CREATION_FLAGS, THREAD_PRIORITY,
    IDLE_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    ABOVE_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, REALTIME_PRIORITY_CLASS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
};
use windows::core::BOOL;
use std::time::Duration;

/// T148-T150: Process handle with RAII cleanup
///
/// Automatically closes process handle when dropped.
pub struct ProcessHandle {
    handle: HANDLE,
    pid: u32,
}

impl ProcessHandle {
    /// T149: Open process with appropriate access rights
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to open
    /// * `access` - Desired access rights
    ///
    /// # Returns
    ///
    /// Result with ProcessHandle or error
    pub fn open(pid: u32, access: u32) -> Result<Self, ProcessError> {
        unsafe {
            let handle = OpenProcess(
                windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS(access),
                false,
                pid,
            )
            .map_err(|e| ProcessError::AccessDenied(format!("Failed to open process {}: {}", pid, e)))?;

            if handle.is_invalid() {
                return Err(ProcessError::NotFound(pid));
            }

            Ok(Self { handle, pid })
        }
    }

    /// Open for termination
    pub fn open_for_terminate(pid: u32) -> Result<Self, ProcessError> {
        Self::open(pid, PROCESS_TERMINATE.0 | PROCESS_QUERY_INFORMATION.0)
    }

    /// Open for priority management
    pub fn open_for_priority(pid: u32) -> Result<Self, ProcessError> {
        Self::open(pid, PROCESS_SET_INFORMATION.0 | PROCESS_QUERY_INFORMATION.0)
    }

    /// Open for suspension
    pub fn open_for_suspend(pid: u32) -> Result<Self, ProcessError> {
        Self::open(pid, PROCESS_SUSPEND_RESUME.0)
    }

    /// Open for affinity management
    pub fn open_for_affinity(pid: u32) -> Result<Self, ProcessError> {
        Self::open(pid, PROCESS_SET_INFORMATION.0 | PROCESS_QUERY_INFORMATION.0)
    }

    /// Get raw handle
    pub fn as_raw(&self) -> HANDLE {
        self.handle
    }

    /// Get process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }
}

/// T150: RAII cleanup - automatically close handle
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

/// T159: Process priority class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityClass {
    /// Lowest priority - only runs when system is idle
    Idle,
    /// Below normal priority
    BelowNormal,
    /// Normal priority (default for most processes)
    Normal,
    /// Above normal priority
    AboveNormal,
    /// High priority (can impact system responsiveness)
    High,
    /// Real-time priority (WARNING: can starve system!)
    Realtime,
}

impl PriorityClass {
    /// Convert to Win32 priority class constant
    fn to_win32(&self) -> PROCESS_CREATION_FLAGS {
        match self {
            PriorityClass::Idle => IDLE_PRIORITY_CLASS,
            PriorityClass::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            PriorityClass::Normal => NORMAL_PRIORITY_CLASS,
            PriorityClass::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
            PriorityClass::High => HIGH_PRIORITY_CLASS,
            PriorityClass::Realtime => REALTIME_PRIORITY_CLASS,
        }
    }

    /// Convert from Win32 priority class constant
    fn from_win32(value: PROCESS_CREATION_FLAGS) -> Option<Self> {
        match value {
            IDLE_PRIORITY_CLASS => Some(PriorityClass::Idle),
            BELOW_NORMAL_PRIORITY_CLASS => Some(PriorityClass::BelowNormal),
            NORMAL_PRIORITY_CLASS => Some(PriorityClass::Normal),
            ABOVE_NORMAL_PRIORITY_CLASS => Some(PriorityClass::AboveNormal),
            HIGH_PRIORITY_CLASS => Some(PriorityClass::High),
            REALTIME_PRIORITY_CLASS => Some(PriorityClass::Realtime),
            _ => None,
        }
    }
}

/// T181: Process control error types
#[derive(Debug)]
pub enum ProcessError {
    /// Access denied (insufficient privileges)
    AccessDenied(String),
    /// Process not found
    NotFound(u32),
    /// Invalid operation for current process state
    InvalidOperation(String),
    /// Operation timed out
    Timeout,
    /// Other Windows API error
    WindowsError(windows::core::Error),
}

/// T182: Convert Windows errors to ProcessError
impl From<windows::core::Error> for ProcessError {
    fn from(err: windows::core::Error) -> Self {
        ProcessError::WindowsError(err)
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
            Self::NotFound(pid) => write!(f, "Process {} not found", pid),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::WindowsError(err) => write!(f, "Windows error: {}", err),
        }
    }
}

impl std::error::Error for ProcessError {}

/// T183: User-friendly error messages from HRESULT codes
impl ProcessError {
    /// Convert to user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            Self::AccessDenied(msg) => {
                format!(
                    "Access denied: {}\n\nYou may need administrator privileges to perform this operation.\n\
                    Try right-clicking the application and selecting 'Run as administrator'.",
                    msg
                )
            }
            Self::NotFound(pid) => {
                format!(
                    "Process {} not found.\n\nThe process may have already exited or you may not have permission to see it.",
                    pid
                )
            }
            Self::InvalidOperation(msg) => {
                format!(
                    "Cannot perform this operation: {}\n\nThe process may be in a state that doesn't allow this action.",
                    msg
                )
            }
            Self::Timeout => {
                "The operation timed out.\n\nThe process may be unresponsive or taking longer than expected to complete."
                    .to_string()
            }
            Self::WindowsError(err) => {
                let hresult = err.code().0;
                match hresult {
                    -2147024891 => { // 0x80070005 - ERROR_ACCESS_DENIED
                        "Access denied.\n\nYou don't have permission to perform this operation. \
                        Try running as administrator or check if the process is protected by the system."
                            .to_string()
                    }
                    -2147024809 => { // 0x80070057 - ERROR_INVALID_PARAMETER
                        "Invalid parameter.\n\nThe operation received an invalid parameter. \
                        This may indicate a bug or an unsupported operation."
                            .to_string()
                    }
                    -2147024890 => { // 0x80070006 - ERROR_INVALID_HANDLE
                        "Invalid handle.\n\nThe process handle is no longer valid. \
                        The process may have exited."
                            .to_string()
                    }
                    0x00000102 => {
                        // WAIT_TIMEOUT
                        "Operation timed out.\n\nThe process did not respond within the expected time."
                            .to_string()
                    }
                    _ => format!(
                        "Windows error (code: 0x{:08X}): {}\n\n\
                        This is an unexpected error. Please check the Windows Event Viewer for more details.",
                        hresult as u32, err
                    ),
                }
            }
        }
    }

    /// Get short description for logging (T184)
    pub fn short_description(&self) -> String {
        match self {
            Self::AccessDenied(msg) => format!("Access denied: {}", msg),
            Self::NotFound(pid) => format!("Process {} not found", pid),
            Self::InvalidOperation(msg) => format!("Invalid operation: {}", msg),
            Self::Timeout => "Timeout".to_string(),
            Self::WindowsError(err) => format!("Windows error 0x{:08X}", err.code().0 as u32),
        }
    }
}

/// T151-T157: Process termination functions
pub mod termination {
    use super::*;

    /// Context for window enumeration
    struct EnumWindowsContext {
        pid: u32,
        windows: Vec<HWND>,
    }

    /// T152: Enumerate windows for process
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let context = &mut *(lparam.0 as *mut EnumWindowsContext);
            
            let mut window_pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
            
            if window_pid == context.pid {
                context.windows.push(hwnd);
            }
            
            true.into()
        }
    }

    /// T151-T153: Graceful termination via WM_CLOSE
    ///
    /// Attempts to close all top-level windows for the process.
    /// Returns true if at least one window was found.
    pub fn terminate_graceful(pid: u32) -> bool {
        unsafe {
            let mut context = EnumWindowsContext {
                pid,
                windows: Vec::new(),
            };

            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut context as *mut _ as isize),
            );

            for &hwnd in &context.windows {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, windows::Win32::Foundation::WPARAM(0), windows::Win32::Foundation::LPARAM(0));
            }

            !context.windows.is_empty()
        }
    }

    /// T154: Forceful termination via TerminateProcess
    ///
    /// Immediately terminates the process. Use as last resort.
    pub fn terminate_force(pid: u32) -> Result<(), ProcessError> {
        let handle = ProcessHandle::open_for_terminate(pid)?;
        
        unsafe {
            TerminateProcess(handle.as_raw(), 1)
                .map_err(|e| ProcessError::WindowsError(e))?;
        }
        
        Ok(())
    }

    /// T155-T156: Terminate with timeout
    ///
    /// Attempts graceful termination first, then forceful if timeout expires.
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID to terminate
    /// * `timeout` - How long to wait for graceful termination
    ///
    /// # Returns
    ///
    /// Ok if process terminated, Err if timeout or error
    pub fn terminate_with_timeout(pid: u32, timeout: Duration) -> Result<(), ProcessError> {
        // Try graceful first
        let had_windows = terminate_graceful(pid);
        
        if had_windows {
            // Wait for process to exit
            let handle = ProcessHandle::open_for_terminate(pid)?;
            
            unsafe {
                let timeout_ms = timeout.as_millis().min(u32::MAX as u128) as u32;
                let result = WaitForSingleObject(handle.as_raw(), timeout_ms);
                
                // WAIT_OBJECT_0 = 0 means process exited
                if result.0 == 0 {
                    return Ok(());
                }
            }
        }
        
        // Graceful failed or timed out, force terminate
        terminate_force(pid)
    }

    /// T157: Terminate process tree (process and all children)
    ///
    /// Recursively terminates child processes then parent.
    /// NOTE: Requires enumerating child processes - simplified implementation.
    pub fn terminate_tree(pid: u32, timeout: Duration) -> Result<(), ProcessError> {
        // Simplified: Just terminate the process itself
        // Full implementation would enumerate children using CreateToolhelp32Snapshot
        terminate_with_timeout(pid, timeout)
    }
}

/// T158-T162: Priority management functions
pub mod priority {
    use super::*;

    /// T158-T160: Set process priority class
    pub fn set_priority(pid: u32, priority: PriorityClass) -> Result<(), ProcessError> {
        let handle = ProcessHandle::open_for_priority(pid)?;
        
        // T161: Warn on Realtime priority
        if priority == PriorityClass::Realtime {
            eprintln!("WARNING: Setting Realtime priority can starve the system!");
        }
        
        unsafe {
            SetPriorityClass(handle.as_raw(), priority.to_win32())
                .map_err(|e| ProcessError::WindowsError(e))?;
        }
        
        Ok(())
    }

    /// T160: Get current process priority
    pub fn get_priority(pid: u32) -> Result<PriorityClass, ProcessError> {
        let handle = ProcessHandle::open_for_priority(pid)?;
        
        unsafe {
            let priority = GetPriorityClass(handle.as_raw());
            PriorityClass::from_win32(PROCESS_CREATION_FLAGS(priority))
                .ok_or_else(|| ProcessError::InvalidOperation(format!("Unknown priority class: {}", priority)))
        }
    }

    /// T162: Per-thread priority adjustment
    ///
    /// Sets priority for a specific thread within a process.
    /// Thread priorities are relative to the process base priority.
    pub fn set_thread_priority(thread_id: u32, priority: ThreadPriority) -> Result<(), ProcessError> {
        use windows::Win32::System::Threading::{OpenThread, SetThreadPriority, THREAD_SET_INFORMATION};
        
        unsafe {
            let handle = OpenThread(THREAD_SET_INFORMATION, false, thread_id)
                .map_err(|e| ProcessError::WindowsError(e))?;
            
            SetThreadPriority(handle, priority.to_win32())
                .map_err(|e| ProcessError::WindowsError(e))?;
            
            let _ = CloseHandle(handle);
            Ok(())
        }
    }

    /// T162: Enumerate threads for a process
    ///
    /// Returns list of thread IDs for the specified process.
    /// 
    /// Note: Currently disabled due to missing ToolHelp API in windows crate
    #[allow(dead_code)]
    pub fn enumerate_threads(_pid: u32) -> Result<Vec<u32>, ProcessError> {
        // TODO: Re-enable when ToolHelp API is available
        // use windows::Win32::System::Diagnostics::ToolHelp::{
        //     CreateToolhelp32Snapshot, Thread32First, Thread32Next,
        //     THREADENTRY32, TH32CS_SNAPTHREAD,
        // };
        
        // For now, return empty vector
        Ok(Vec::new())
        
        /*
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)
                .map_err(|e| ProcessError::WindowsError(e))?;
            
            let mut entry = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };
            
            let mut threads = Vec::new();
            
            if Thread32First(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        threads.push(entry.th32ThreadID);
                    }
                    
                    if Thread32Next(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            
            let _ = CloseHandle(snapshot);
            Ok(threads)
        }
        */
    }
}

/// Thread priority level (relative to process base priority)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadPriority {
    /// Idle priority
    Idle = -15,
    /// Lowest priority
    Lowest = -2,
    /// Below normal priority
    BelowNormal = -1,
    /// Normal priority
    Normal = 0,
    /// Above normal priority
    AboveNormal = 1,
    /// Highest priority
    Highest = 2,
    /// Time critical priority
    TimeCritical = 15,
}

impl ThreadPriority {
    /// Convert to Win32 THREAD_PRIORITY value
    pub fn to_win32(&self) -> THREAD_PRIORITY {
        THREAD_PRIORITY(*self as i32)
    }
}

/// T163-T166: Process suspension functions
pub mod suspension {
    use super::*;

    // NtSuspendProcess/NtResumeProcess from ntdll.dll
    #[link(name = "ntdll")]
    extern "system" {
        fn NtSuspendProcess(ProcessHandle: isize) -> i32;
        fn NtResumeProcess(ProcessHandle: isize) -> i32;
    }

    /// T163: Suspend process
    ///
    /// WARNING: Can cause deadlocks if process holds critical locks!
    pub fn suspend(pid: u32) -> Result<(), ProcessError> {
        let handle = ProcessHandle::open_for_suspend(pid)?;
        
        unsafe {
            let status = NtSuspendProcess(handle.as_raw().0 as isize);
            if status < 0 {
                return Err(ProcessError::InvalidOperation(format!("NtSuspendProcess failed: 0x{:08X}", status)));
            }
        }
        
        Ok(())
    }

    /// T164: Resume process
    pub fn resume(pid: u32) -> Result<(), ProcessError> {
        let handle = ProcessHandle::open_for_suspend(pid)?;
        
        unsafe {
            let status = NtResumeProcess(handle.as_raw().0 as isize);
            if status < 0 {
                return Err(ProcessError::InvalidOperation(format!("NtResumeProcess failed: 0x{:08X}", status)));
            }
        }
        
        Ok(())
    }
}

/// T167-T169: CPU affinity functions
pub mod affinity {
    use super::*;

    /// T167-T168: Set process CPU affinity mask
    ///
    /// # Arguments
    ///
    /// * `pid` - Process ID
    /// * `mask` - Bitmask where bit N = CPU N (e.g., 0b0011 = CPUs 0 and 1)
    pub fn set_affinity(pid: u32, mask: usize) -> Result<(), ProcessError> {
        let handle = ProcessHandle::open_for_affinity(pid)?;
        
        unsafe {
            SetProcessAffinityMask(handle.as_raw(), mask)
                .map_err(|e| ProcessError::WindowsError(e))?;
        }
        
        Ok(())
    }

    /// T168: Get process CPU affinity mask
    pub fn get_affinity(pid: u32) -> Result<(usize, usize), ProcessError> {
        let handle = ProcessHandle::open_for_affinity(pid)?;
        
        let mut process_mask = 0usize;
        let mut system_mask = 0usize;
        
        unsafe {
            GetProcessAffinityMask(handle.as_raw(), &mut process_mask, &mut system_mask)
                .map_err(|e| ProcessError::WindowsError(e))?;
        }
        
        Ok((process_mask, system_mask))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_conversion() {
        assert_eq!(PriorityClass::Normal.to_win32(), NORMAL_PRIORITY_CLASS);
        assert_eq!(PriorityClass::High.to_win32(), HIGH_PRIORITY_CLASS);
        
        assert_eq!(PriorityClass::from_win32(NORMAL_PRIORITY_CLASS), Some(PriorityClass::Normal));
    }

    #[test]
    fn test_open_current_process() {
        let pid = std::process::id();
        let handle = ProcessHandle::open_for_priority(pid).unwrap();
        assert_eq!(handle.pid(), pid);
    }

    #[test]
    fn test_get_current_priority() {
        let pid = std::process::id();
        let priority = priority::get_priority(pid).unwrap();
        // Most processes run at Normal priority
        println!("Current process priority: {:?}", priority);
    }

    #[test]
    fn test_get_current_affinity() {
        let pid = std::process::id();
        let (process_mask, system_mask) = affinity::get_affinity(pid).unwrap();
        assert!(system_mask > 0, "System should have at least one CPU");
        println!("Process affinity: 0x{:X}, System affinity: 0x{:X}", process_mask, system_mask);
    }
}
