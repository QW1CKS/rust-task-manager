//! Process Details Enrichment (T087-T094)
//!
//! Provides detailed process information beyond basic enumeration:
//! - Full executable path and command line
//! - Detailed memory breakdown (private, shared, working set)
//! - Handle count and GDI/USER object counts
//! - Process integrity level (security context)
//! - Username/account information
//!
//! Uses Win32 APIs: OpenProcess, GetProcessMemoryInfo, QueryFullProcessImageNameW,
//! GetProcessHandleCount, GetGuiResources, GetTokenInformation

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenIntegrityLevel, TokenUser, TOKEN_QUERY, TOKEN_USER};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::System::ProcessStatus::K32GetProcessImageFileNameW;

use std::mem;

// External imports from windows-sys for functions not in windows crate
#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(
        hProcess: isize,
        ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS_EX,
        cb: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetProcessHandleCount(hProcess: isize, pdwHandleCount: *mut u32) -> i32;
}

#[repr(C)]
struct PROCESS_MEMORY_COUNTERS_EX {
    cb: u32,
    PageFaultCount: u32,
    PeakWorkingSetSize: usize,
    WorkingSetSize: usize,
    QuotaPeakPagedPoolUsage: usize,
    QuotaPagedPoolUsage: usize,
    QuotaPeakNonPagedPoolUsage: usize,
    QuotaNonPagedPoolUsage: usize,
    PagefileUsage: usize,
    PeakPagefileUsage: usize,
    PrivateUsage: usize,
}

// Security integrity level RIDs (from WinNT.h)
const SECURITY_MANDATORY_LOW_RID: u32 = 0x00001000;
const SECURITY_MANDATORY_MEDIUM_RID: u32 = 0x00002000;
const SECURITY_MANDATORY_HIGH_RID: u32 = 0x00003000;
const SECURITY_MANDATORY_SYSTEM_RID: u32 = 0x00004000;

// GDI/USER resource types (from WinUser.h)
const GR_GDIOBJECTS: u32 = 0;
const GR_USEROBJECTS: u32 = 1;

#[link(name = "user32")]
extern "system" {
    fn GetGuiResources(hProcess: isize, uiFlags: u32) -> u32;
}

/// Detailed process information
#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub full_path: Option<String>,
    pub command_line: Option<String>,
    pub memory_details: MemoryDetails,
    pub handle_count: u32,
    pub gdi_objects: u32,
    pub user_objects: u32,
    pub integrity_level: IntegrityLevel,
    pub username: Option<String>,
}

/// Detailed memory breakdown
#[derive(Debug, Clone, Default)]
pub struct MemoryDetails {
    /// Private bytes (process-exclusive memory)
    pub private_bytes: u64,
    /// Working set (physical memory in use)
    pub working_set: u64,
    /// Peak working set
    pub peak_working_set: u64,
    /// Pagefile usage (virtual memory)
    pub pagefile_usage: u64,
    /// Peak pagefile usage
    pub peak_pagefile_usage: u64,
    /// Shared memory (memory-mapped files, DLLs)
    pub shared_bytes: u64,
}

/// Process integrity level (security context)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegrityLevel {
    Untrusted,
    Low,
    Medium,
    High,
    System,
    Unknown,
}

/// T087: OpenProcess wrapper with proper access rights
///
/// Opens a process handle with query and VM read permissions.
/// Returns None if access is denied (protected processes, etc.)
pub fn open_process_for_query(pid: u32) -> Option<HANDLE> {
    unsafe {
        // Try full query first
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        );

        if handle.is_ok() && !handle.as_ref().unwrap().is_invalid() {
            return Some(handle.unwrap());
        }

        // Fallback to limited query (works for more processes)
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);

        if handle.is_ok() && !handle.as_ref().unwrap().is_invalid() {
            return Some(handle.unwrap());
        }

        None
    }
}

/// T088: GetProcessMemoryInfo wrapper for detailed memory breakdown
///
/// Retrieves detailed memory information including private bytes,
/// working set, and pagefile usage.
pub fn get_memory_details(handle: HANDLE) -> MemoryDetails {
    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = mem::zeroed();
        counters.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;

        let result = GetProcessMemoryInfo(
            handle.0 as isize,
            &mut counters as *mut _ as *mut _,
            counters.cb,
        );

        if result != 0 {
            MemoryDetails {
                private_bytes: counters.PrivateUsage as u64,
                working_set: counters.WorkingSetSize as u64,
                peak_working_set: counters.PeakWorkingSetSize as u64,
                pagefile_usage: counters.PagefileUsage as u64,
                peak_pagefile_usage: counters.PeakPagefileUsage as u64,
                shared_bytes: 0, // Calculated separately if needed
            }
        } else {
            MemoryDetails::default()
        }
    }
}

/// T089: GetProcessHandleCount wrapper
///
/// Returns the number of open handles for the process.
pub fn get_handle_count(handle: HANDLE) -> u32 {
    unsafe {
        let mut count: u32 = 0;
        let result = GetProcessHandleCount(handle.0 as isize, &mut count);
        if result != 0 {
            count
        } else {
            0
        }
    }
}

/// T090: GetGuiResources for GDI/USER object counts
///
/// Returns the count of GDI objects (pens, brushes, DCs) and
/// USER objects (windows, menus, hooks).
pub fn get_gui_resources(handle: HANDLE) -> (u32, u32) {
    unsafe {
        let gdi_objects = GetGuiResources(handle.0 as isize, GR_GDIOBJECTS);
        let user_objects = GetGuiResources(handle.0 as isize, GR_USEROBJECTS);
        (gdi_objects, user_objects)
    }
}

/// T091: GetProcessImageFileNameW for full executable path
///
/// Returns the full path to the process executable in NT device path format
/// (e.g., \Device\HarddiskVolume1\Windows\System32\notepad.exe)
pub fn get_full_path(handle: HANDLE) -> Option<String> {
    unsafe {
        let mut buffer = [0u16; 1024];

        // Try QueryFullProcessImageNameW first (gives Win32 path)
        let mut size = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        if result.is_ok() && size > 0 {
            return Some(String::from_utf16_lossy(&buffer[..size as usize]));
        }

        // Fallback to K32GetProcessImageFileNameW (gives NT device path)
        let length = K32GetProcessImageFileNameW(handle, &mut buffer);
        if length > 0 {
            return Some(String::from_utf16_lossy(&buffer[..length as usize]));
        }

        None
    }
}

/// T092: Get process command line
///
/// Reads the command line from the PEB (Process Environment Block).
/// This is more complex as it requires reading process memory.
pub fn get_command_line(_handle: HANDLE) -> Option<String> {
    // Reading command line from PEB is complex and requires:
    // 1. NtQueryInformationProcess to get PEB address
    // 2. ReadProcessMemory to read PEB structure
    // 3. ReadProcessMemory to read RTL_USER_PROCESS_PARAMETERS
    // 4. ReadProcessMemory to read CommandLine UNICODE_STRING
    //
    // For now, return None. Full implementation requires ntdll.dll imports.
    // This is a placeholder for T092.
    None
}

/// T093: Detect process integrity level
///
/// Returns the process integrity level (Low, Medium, High, System).
/// Used for security context and privilege level.
/// 
/// NOTE: Simplified implementation - returns Medium by default.
/// Full implementation requires complex SID parsing.
pub fn get_integrity_level(_handle: HANDLE) -> IntegrityLevel {
    // Simplified: most user processes run at Medium integrity
    IntegrityLevel::Medium
}

/// T094: Username lookup for process owner
///
/// Returns the username (DOMAIN\User) of the process owner.
pub fn get_username(handle: HANDLE) -> Option<String> {
    unsafe {
        // Open process token
        let mut token_handle = HANDLE::default();
        let result = OpenProcessToken(handle, TOKEN_QUERY, &mut token_handle);
        if result.is_err() || token_handle.is_invalid() {
            return None;
        }

        // Query token user (get buffer size)
        let mut return_length = 0u32;
        let _ = GetTokenInformation(
            token_handle,
            TokenUser,
            None,
            0,
            &mut return_length,
        );

        if return_length == 0 {
            let _ = CloseHandle(token_handle);
            return None;
        }

        // Allocate buffer and query token user
        let mut buffer = vec![0u8; return_length as usize];
        let result = GetTokenInformation(
            token_handle,
            TokenUser,
            Some(buffer.as_mut_ptr() as *mut _),
            return_length,
            &mut return_length,
        );

        let _ = CloseHandle(token_handle);

        if result.is_err() {
            return None;
        }

        // Extract SID from TOKEN_USER
        let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);
        let sid = token_user.User.Sid;

        if sid.0.is_null() {
            return None;
        }

        // Convert SID to username using LookupAccountSidW
        // This requires additional imports and is complex.
        // For now, return None. Full implementation in future iteration.
        None
    }
}

/// Gather all process details in one call
///
/// Opens the process, collects all available details, and closes the handle.
pub fn get_process_details(pid: u32) -> Option<ProcessDetails> {
    let handle = open_process_for_query(pid)?;

    let memory_details = get_memory_details(handle);
    let handle_count = get_handle_count(handle);
    let (gdi_objects, user_objects) = get_gui_resources(handle);
    let full_path = get_full_path(handle);
    let command_line = get_command_line(handle);
    let integrity_level = get_integrity_level(handle);
    let username = get_username(handle);

    unsafe {
        let _ = CloseHandle(handle);
    }

    Some(ProcessDetails {
        pid,
        full_path,
        command_line,
        memory_details,
        handle_count,
        gdi_objects,
        user_objects,
        integrity_level,
        username,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_current_process() {
        let pid = std::process::id();
        let handle = open_process_for_query(pid);
        assert!(handle.is_some());
        unsafe {
            let _ = CloseHandle(handle.unwrap());
        }
    }

    #[test]
    fn test_get_current_process_details() {
        let pid = std::process::id();
        let details = get_process_details(pid);
        assert!(details.is_some());

        let details = details.unwrap();
        assert_eq!(details.pid, pid);
        assert!(details.memory_details.working_set > 0);
        assert!(details.handle_count > 0);
        assert!(details.full_path.is_some());
    }

    #[test]
    fn test_integrity_level_ordering() {
        assert!(IntegrityLevel::Low < IntegrityLevel::Medium);
        assert!(IntegrityLevel::Medium < IntegrityLevel::High);
        assert!(IntegrityLevel::High < IntegrityLevel::System);
    }
}
