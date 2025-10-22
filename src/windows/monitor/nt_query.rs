//! NtQuerySystemInformation FFI wrapper for process enumeration
//!
//! This module provides unsafe wrappers around ntdll.dll's NtQuerySystemInformation
//! for efficient process enumeration. Uses pre-allocated buffers to meet zero-allocation
//! requirements in hot paths.

use std::mem;
use windows::Win32::Foundation::UNICODE_STRING;

/// System process information class
const SYSTEM_PROCESS_INFORMATION: u32 = 5;

/// Maximum buffer size for process enumeration (1MB)
/// This is pre-allocated once to avoid allocations in monitoring loop
const MAX_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// NTSTATUS type from Windows (i32)
#[allow(clippy::upper_case_acronyms)]
type NTSTATUS = i32;

// Direct FFI binding to NtQuerySystemInformation from ntdll.dll
//
// # Safety
//
// This is a raw FFI binding. Caller must ensure:
// - SystemInformationClass is valid
// - SystemInformation buffer is large enough
// - ReturnLength is a valid pointer
#[link(name = "ntdll")]
extern "system" {
    fn NtQuerySystemInformation(
        system_information_class: u32,
        system_information: *mut std::ffi::c_void,
        system_information_length: u32,
        return_length: *mut u32,
    ) -> NTSTATUS;
}

/// Process information extracted from SYSTEM_PROCESS_INFORMATION
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub parent_pid: u32,
    /// Process name (extracted from UNICODE_STRING)
    pub name: String,
    /// Thread count
    pub thread_count: u32,
    /// Handle count
    pub handle_count: u32,
    /// User-mode CPU time (100ns units)
    pub cpu_time_user: u64,
    /// Kernel-mode CPU time (100ns units)
    pub cpu_time_kernel: u64,
    /// Working set size (bytes)
    pub memory_working_set: u64,
    /// Page file usage (bytes)
    pub memory_pagefile: u64,
    /// Private page count
    pub memory_private: u64,
}

/// SYSTEM_PROCESS_INFORMATION structure from ntdll.dll
/// This matches the ABI of the Windows kernel structure
#[repr(C)]
#[allow(non_camel_case_types)]
struct SYSTEM_PROCESS_INFORMATION {
    next_entry_offset: u32,
    number_of_threads: u32,
    working_set_private_size: i64,
    hard_fault_count: u32,
    number_of_threads_high_watermark: u32,
    cycle_time: u64,
    create_time: i64,
    user_time: i64,
    kernel_time: i64,
    image_name: UNICODE_STRING,
    base_priority: i32, // KPRIORITY
    unique_process_id: usize,
    inherited_from_unique_process_id: usize,
    handle_count: u32,
    session_id: u32,
    unique_process_key: usize,
    peak_virtual_size: usize,
    virtual_size: usize,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
    private_page_count: usize,
    // Additional fields omitted for brevity - we extract what we need above
}

/// Process enumerator with pre-allocated buffer
pub struct ProcessEnumerator {
    /// Pre-allocated 1MB buffer for NtQuerySystemInformation
    buffer: Vec<u8>,
}

impl ProcessEnumerator {
    /// Create a new process enumerator with pre-allocated buffer
    ///
    /// # Performance
    ///
    /// Allocates 1MB once during initialization. After this, enumerate_processes()
    /// performs zero allocations (except for String conversions from process names).
    pub fn new() -> Self {
        Self {
            buffer: vec![0u8; MAX_BUFFER_SIZE],
        }
    }

    /// Enumerate all processes on the system
    ///
    /// # Safety
    ///
    /// This function contains unsafe blocks for:
    /// - FFI call to NtQuerySystemInformation
    /// - Pointer arithmetic through linked list of SYSTEM_PROCESS_INFORMATION
    /// - Reading UNICODE_STRING from kernel memory
    ///
    /// # Pre-conditions
    ///
    /// - buffer.len() >= MAX_BUFFER_SIZE
    /// - buffer is properly aligned for SYSTEM_PROCESS_INFORMATION
    ///
    /// # Post-conditions
    ///
    /// - Returns Ok(Vec<ProcessInfo>) with all running processes
    /// - Returns Err if NtQuerySystemInformation fails or buffer too small
    ///
    /// # Performance
    ///
    /// Target: <5ms for 1000 processes
    /// Actual measurements needed via benchmarks/monitoring.rs
    pub fn enumerate_processes(&mut self) -> Result<Vec<ProcessInfo>, String> {
        let mut return_length: u32 = 0;

        // T359: Validate buffer preconditions
        debug_assert_eq!(self.buffer.len(), MAX_BUFFER_SIZE, 
            "buffer size {} != MAX_BUFFER_SIZE {}", self.buffer.len(), MAX_BUFFER_SIZE);
        debug_assert!(!self.buffer.is_empty(), "buffer must not be empty");

        // SAFETY (T358): NtQuerySystemInformation call is safe because:
        // - SYSTEM_PROCESS_INFORMATION (5) is a valid information class
        // - self.buffer is properly allocated with MAX_BUFFER_SIZE capacity
        // - buffer pointer is valid for the entire buffer length
        // - return_length is a valid mutable reference
        // - Function is a stable Windows API exported by ntdll.dll
        let status = unsafe {
            NtQuerySystemInformation(
                SYSTEM_PROCESS_INFORMATION,
                self.buffer.as_mut_ptr() as *mut std::ffi::c_void,
                self.buffer.len() as u32,
                &mut return_length,
            )
        };

        // Check if call succeeded (NTSTATUS >= 0 means success)
        if status < 0 {
            return Err(format!(
                "NtQuerySystemInformation failed: NTSTATUS 0x{:08X}",
                status as u32
            ));
        }

        // T359: Validate return length is reasonable
        debug_assert!(return_length as usize <= self.buffer.len(),
            "return_length {} exceeds buffer size {}", return_length, self.buffer.len());

        // Parse the linked list of SYSTEM_PROCESS_INFORMATION structures
        self.parse_process_list()
    }

    /// Parse linked list of SYSTEM_PROCESS_INFORMATION structures
    ///
    /// # Safety
    ///
    /// Unsafe pointer arithmetic walking linked list via NextEntryOffset.
    /// Each structure may be at variable offset depending on thread info size.
    /// 
    /// # Performance (T313)
    /// 
    /// Uses rayon for parallel extraction when process count > 100.
    /// Parallel processing can reduce enumeration time by 2-3x on multi-core systems.
    fn parse_process_list(&self) -> Result<Vec<ProcessInfo>, String> {
        // First pass: collect all valid offsets
        let mut offsets = Vec::with_capacity(256);
        let mut offset: usize = 0;

        loop {
            // T359: Validate offset in bounds before pointer arithmetic
            debug_assert!(offset <= self.buffer.len(), 
                "offset {} exceeds buffer length {}", offset, self.buffer.len());
            
            if offset + mem::size_of::<SYSTEM_PROCESS_INFORMATION>() > self.buffer.len() {
                break;
            }

            offsets.push(offset);

            // SAFETY (T358): Pointer arithmetic is safe because:
            // - offset is validated to be within buffer bounds above
            // - buffer.len() >= offset + size_of(SYSTEM_PROCESS_INFORMATION)
            // - buffer is properly allocated and aligned
            let ptr = unsafe {
                self.buffer.as_ptr().add(offset) as *const SYSTEM_PROCESS_INFORMATION
            };
            
            // SAFETY (T358): Dereferencing is safe because:
            // - ptr points to valid memory within buffer bounds
            // - SYSTEM_PROCESS_INFORMATION layout matches Windows kernel ABI
            // - buffer was filled by NtQuerySystemInformation which guarantees valid data
            let info = unsafe { &*ptr };

            if info.next_entry_offset == 0 {
                break;
            }
            offset += info.next_entry_offset as usize;
        }

        // T313: Use parallel processing for large process lists
        if offsets.len() > 100 {
            use rayon::prelude::*;
            Ok(offsets
                .par_iter()
                .filter_map(|&off| {
                    // T359: Validate offset in bounds
                    debug_assert!(off + mem::size_of::<SYSTEM_PROCESS_INFORMATION>() <= self.buffer.len(),
                        "parallel offset {} invalid for buffer size {}", off, self.buffer.len());
                    
                    // SAFETY (T358): Same guarantees as above - offset validated, buffer valid
                    let ptr = unsafe {
                        self.buffer.as_ptr().add(off) as *const SYSTEM_PROCESS_INFORMATION
                    };
                    let info = unsafe { &*ptr };
                    self.extract_process_info(info)
                })
                .collect())
        } else {
            // Sequential processing for small lists (lower overhead)
            Ok(offsets
                .iter()
                .filter_map(|&off| {
                    // T359: Validate offset in bounds
                    debug_assert!(off + mem::size_of::<SYSTEM_PROCESS_INFORMATION>() <= self.buffer.len(),
                        "sequential offset {} invalid for buffer size {}", off, self.buffer.len());
                    
                    // SAFETY (T358): Same guarantees as above - offset validated, buffer valid
                    let ptr = unsafe {
                        self.buffer.as_ptr().add(off) as *const SYSTEM_PROCESS_INFORMATION
                    };
                    let info = unsafe { &*ptr };
                    self.extract_process_info(info)
                })
                .collect())
        }
    }

    /// Extract ProcessInfo from SYSTEM_PROCESS_INFORMATION
    ///
    /// # Arguments
    ///
    /// * `info` - Pointer to SYSTEM_PROCESS_INFORMATION structure
    ///
    /// # Returns
    ///
    /// Some(ProcessInfo) if extraction successful, None if process is System Idle (PID 0)
    fn extract_process_info(&self, info: &SYSTEM_PROCESS_INFORMATION) -> Option<ProcessInfo> {
        // Extract PID (unique_process_id is a usize pointer in kernel, truncate to u32)
        let pid = info.unique_process_id as u32;

        // Skip System Idle Process (PID 0)
        if pid == 0 {
            return None;
        }

        // Extract parent PID
        let parent_pid = info.inherited_from_unique_process_id as u32;

        // Extract process name from UNICODE_STRING
        let name = self.extract_process_name(&info.image_name);

        // Extract CPU times (100ns units)
        let cpu_time_user = info.user_time as u64;
        let cpu_time_kernel = info.kernel_time as u64;

        // Extract memory metrics
        let memory_working_set = info.working_set_size as u64;
        let memory_pagefile = info.pagefile_usage as u64;
        let memory_private = info.private_page_count as u64;

        Some(ProcessInfo {
            pid,
            parent_pid,
            name,
            thread_count: info.number_of_threads,
            handle_count: info.handle_count,
            cpu_time_user,
            cpu_time_kernel,
            memory_working_set,
            memory_pagefile,
            memory_private,
        })
    }

    /// Extract process name from UNICODE_STRING
    ///
    /// # Safety
    ///
    /// Reads UTF-16 string from kernel memory pointer in UNICODE_STRING.
    ///
    /// # Arguments
    ///
    /// * `unicode_str` - UNICODE_STRING containing process name
    ///
    /// # Returns
    ///
    /// Process name as UTF-8 String, or "<unknown>" if extraction fails
    fn extract_process_name(&self, unicode_str: &UNICODE_STRING) -> String {
        // T359: Validate UNICODE_STRING preconditions
        debug_assert!(!unicode_str.Buffer.is_null() || unicode_str.Length == 0,
            "non-null Buffer must have Length > 0");
        debug_assert!(unicode_str.Length % 2 == 0, 
            "UNICODE_STRING Length {} must be even (UTF-16 pairs)", unicode_str.Length);
        debug_assert!(unicode_str.Length <= unicode_str.MaximumLength,
            "Length {} exceeds MaximumLength {}", unicode_str.Length, unicode_str.MaximumLength);
        
        if unicode_str.Buffer.is_null() || unicode_str.Length == 0 {
            return String::from("<unknown>");
        }

        // SAFETY (T358): Reading UTF-16 buffer is safe because:
        // - Buffer pointer is non-null (checked above)
        // - Length is validated to be non-zero and even (UTF-16 requirement)
        // - Length is in bytes, divided by 2 for u16 count
        // - Buffer is populated by Windows kernel with valid UTF-16 data
        // - UNICODE_STRING is part of stable Windows ABI, guaranteed valid layout
        let slice = unsafe {
            std::slice::from_raw_parts(
                unicode_str.Buffer.as_ptr(),
                (unicode_str.Length / 2) as usize, // Length is in bytes, convert to u16 count
            )
        };

        // Convert UTF-16 to UTF-8
        String::from_utf16_lossy(slice)
    }
}

impl Default for ProcessEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_enumerator_creates() {
        let enumerator = ProcessEnumerator::new();
        assert_eq!(enumerator.buffer.len(), MAX_BUFFER_SIZE);
    }

    #[test]
    fn test_enumerate_processes() {
        let mut enumerator = ProcessEnumerator::new();
        let result = enumerator.enumerate_processes();
        assert!(result.is_ok(), "enumerate_processes should succeed");

        let processes = result.unwrap();
        assert!(!processes.is_empty(), "Should find at least one process");

        // Verify we can find our own process
        let current_pid = std::process::id();
        let found = processes.iter().any(|p| p.pid == current_pid);
        assert!(found, "Should find current process in list");
    }

    #[test]
    fn test_process_info_has_valid_data() {
        let mut enumerator = ProcessEnumerator::new();
        let processes = enumerator.enumerate_processes().unwrap();

        for proc in &processes {
            // All processes should have valid PID
            assert!(proc.pid > 0, "PID should be positive");

            // Process names should not be empty (except System Idle which we skip)
            assert!(!proc.name.is_empty(), "Process name should not be empty");

            // Thread count should be positive
            assert!(
                proc.thread_count > 0,
                "Process {} should have at least one thread",
                proc.name
            );
        }
    }
}
