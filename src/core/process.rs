//! Process data structures and store (Structure of Arrays layout)

use static_assertions::const_assert;
use crate::windows::monitor::nt_query::ProcessInfo;

/// Maximum number of processes supported (constitutional requirement)
pub const MAX_PROCESSES: usize = 2048;

// Compile-time assertion to prevent accidental capacity reduction
const_assert!(MAX_PROCESSES == 2048);

/// Process store using Structure of Arrays (SoA) for cache efficiency
///
/// # Memory Layout
///
/// 2048 processes × ~200 bytes/process = ~410KB for SoA storage
/// (well within <15MB idle budget per constitution)
///
/// # Performance
///
/// SoA layout provides:
/// - Better CPU cache utilization when iterating over single metrics
/// - SIMD-friendly data access patterns
/// - Zero allocations after initialization
pub struct ProcessStore {
    /// Current number of processes
    count: usize,

    /// Process IDs (fixed capacity, sorted for binary search)
    pids: Box<[u32; MAX_PROCESSES]>,

    /// Parent process IDs
    parent_pids: Box<[u32; MAX_PROCESSES]>,

    /// Process names
    names: Box<[String; MAX_PROCESSES]>,

    /// Thread counts
    thread_counts: Box<[u32; MAX_PROCESSES]>,

    /// Handle counts
    handle_counts: Box<[u32; MAX_PROCESSES]>,

    /// CPU usage percentages (0.0-100.0)
    cpu_usage: Box<[f32; MAX_PROCESSES]>,

    /// User-mode CPU time (100ns units)
    cpu_time_user: Box<[u64; MAX_PROCESSES]>,

    /// Kernel-mode CPU time (100ns units)
    cpu_time_kernel: Box<[u64; MAX_PROCESSES]>,

    /// Working set size (bytes)
    memory_working_set: Box<[u64; MAX_PROCESSES]>,

    /// Private memory (bytes)
    memory_private: Box<[u64; MAX_PROCESSES]>,

    /// Committed memory (bytes)
    memory_committed: Box<[u64; MAX_PROCESSES]>,

    /// I/O read bytes
    io_read_bytes: Box<[u64; MAX_PROCESSES]>,

    /// I/O write bytes
    io_write_bytes: Box<[u64; MAX_PROCESSES]>,

    /// I/O read operations
    io_read_ops: Box<[u64; MAX_PROCESSES]>,

    /// I/O write operations
    io_write_ops: Box<[u64; MAX_PROCESSES]>,

    /// GDI objects
    gdi_objects: Box<[u32; MAX_PROCESSES]>,

    /// USER objects
    user_objects: Box<[u32; MAX_PROCESSES]>,

    /// Previous CPU times for delta calculation (user)
    prev_cpu_time_user: Box<[u64; MAX_PROCESSES]>,

    /// Previous CPU times for delta calculation (kernel)
    prev_cpu_time_kernel: Box<[u64; MAX_PROCESSES]>,
}

impl ProcessStore {
    /// Create a new empty process store
    ///
    /// # Memory Allocation
    ///
    /// This allocates ~410KB of memory for the SoA arrays. This is a one-time
    /// allocation; after this, update() performs zero allocations.
    pub fn new() -> Self {
        Self {
            count: 0,
            pids: Box::new([0; MAX_PROCESSES]),
            parent_pids: Box::new([0; MAX_PROCESSES]),
            names: Box::new(std::array::from_fn(|_| String::new())),
            thread_counts: Box::new([0; MAX_PROCESSES]),
            handle_counts: Box::new([0; MAX_PROCESSES]),
            cpu_usage: Box::new([0.0; MAX_PROCESSES]),
            cpu_time_user: Box::new([0; MAX_PROCESSES]),
            cpu_time_kernel: Box::new([0; MAX_PROCESSES]),
            memory_working_set: Box::new([0; MAX_PROCESSES]),
            memory_private: Box::new([0; MAX_PROCESSES]),
            memory_committed: Box::new([0; MAX_PROCESSES]),
            io_read_bytes: Box::new([0; MAX_PROCESSES]),
            io_write_bytes: Box::new([0; MAX_PROCESSES]),
            io_read_ops: Box::new([0; MAX_PROCESSES]),
            io_write_ops: Box::new([0; MAX_PROCESSES]),
            gdi_objects: Box::new([0; MAX_PROCESSES]),
            user_objects: Box::new([0; MAX_PROCESSES]),
            prev_cpu_time_user: Box::new([0; MAX_PROCESSES]),
            prev_cpu_time_kernel: Box::new([0; MAX_PROCESSES]),
        }
    }

    /// Update process store with new snapshot
    ///
    /// # Zero Allocations
    ///
    /// This method reuses existing arrays, performing zero allocations after
    /// the initial ProcessStore::new() call. String updates may reallocate
    /// if process names change length, but this is rare.
    ///
    /// # Performance
    ///
    /// Target: <2ms for 1000 processes (including CPU % calculation)
    ///
    /// # Arguments
    ///
    /// * `processes` - Vec of ProcessInfo from NtQuerySystemInformation
    pub fn update(&mut self, processes: Vec<ProcessInfo>) {
        self.count = processes.len().min(MAX_PROCESSES);

        // Copy data from Vec into SoA arrays
        for (i, proc) in processes.into_iter().enumerate().take(MAX_PROCESSES) {
            self.pids[i] = proc.pid;
            self.parent_pids[i] = proc.parent_pid;
            
            // Reuse String allocation if possible
            self.names[i].clear();
            self.names[i].push_str(&proc.name);

            self.thread_counts[i] = proc.thread_count;
            self.handle_counts[i] = proc.handle_count;

            // Calculate CPU usage percentage from time deltas
            let prev_user = self.prev_cpu_time_user[i];
            let prev_kernel = self.prev_cpu_time_kernel[i];
            let delta_user = proc.cpu_time_user.saturating_sub(prev_user);
            let delta_kernel = proc.cpu_time_kernel.saturating_sub(prev_kernel);
            
            // Store new times for next delta
            self.cpu_time_user[i] = proc.cpu_time_user;
            self.cpu_time_kernel[i] = proc.cpu_time_kernel;
            self.prev_cpu_time_user[i] = proc.cpu_time_user;
            self.prev_cpu_time_kernel[i] = proc.cpu_time_kernel;

            // CPU % = (delta_time / elapsed_time) * 100 * num_cpus
            // For now, store raw delta (will be calculated in metrics layer)
            let total_delta = delta_user + delta_kernel;
            self.cpu_usage[i] = total_delta as f32 / 100_000.0; // Rough approximation

            self.memory_working_set[i] = proc.memory_working_set;
            self.memory_private[i] = proc.memory_private;
            self.memory_committed[i] = proc.memory_pagefile;

            // I/O metrics not available from NtQuery - will be populated by detailed query
            // GDI/USER objects require additional queries
        }

        // Sort by PID for binary search
        self.sort_by_pid();
    }

    /// Sort arrays by PID for O(log n) lookup
    ///
    /// This is an in-place sort that maintains SoA structure by swapping
    /// elements across all arrays simultaneously.
    fn sort_by_pid(&mut self) {
        // Simple insertion sort (sufficient for mostly-sorted data)
        // Process list usually changes slowly between updates
        for i in 1..self.count {
            let mut j = i;
            while j > 0 && self.pids[j - 1] > self.pids[j] {
                // Swap all arrays at positions j-1 and j
                self.pids.swap(j - 1, j);
                self.parent_pids.swap(j - 1, j);
                self.names.swap(j - 1, j);
                self.thread_counts.swap(j - 1, j);
                self.handle_counts.swap(j - 1, j);
                self.cpu_usage.swap(j - 1, j);
                self.cpu_time_user.swap(j - 1, j);
                self.cpu_time_kernel.swap(j - 1, j);
                self.memory_working_set.swap(j - 1, j);
                self.memory_private.swap(j - 1, j);
                self.memory_committed.swap(j - 1, j);
                self.io_read_bytes.swap(j - 1, j);
                self.io_write_bytes.swap(j - 1, j);
                self.io_read_ops.swap(j - 1, j);
                self.io_write_ops.swap(j - 1, j);
                self.gdi_objects.swap(j - 1, j);
                self.user_objects.swap(j - 1, j);
                self.prev_cpu_time_user.swap(j - 1, j);
                self.prev_cpu_time_kernel.swap(j - 1, j);
                j -= 1;
            }
        }
    }

    /// Get current process count
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get process info by PID (binary search)
    ///
    /// # Performance
    ///
    /// O(log n) lookup via binary search on sorted PID array
    ///
    /// # Returns
    ///
    /// Some(index) if found, None if not found
    pub fn get_by_pid(&self, pid: u32) -> Option<usize> {
        self.pids[..self.count].binary_search(&pid).ok()
    }

    /// Get process name by index
    pub fn name(&self, index: usize) -> Option<&str> {
        if index < self.count {
            Some(&self.names[index])
        } else {
            None
        }
    }

    /// Get CPU usage by index
    pub fn cpu_usage(&self, index: usize) -> Option<f32> {
        if index < self.count {
            Some(self.cpu_usage[index])
        } else {
            None
        }
    }

    /// Get memory working set by index
    pub fn memory_working_set(&self, index: usize) -> Option<u64> {
        if index < self.count {
            Some(self.memory_working_set[index])
        } else {
            None
        }
    }

    /// Filter processes by name (returns iterator over indices)
    ///
    /// # Zero Allocations
    ///
    /// Returns iterator that yields indices, no Vec allocation.
    ///
    /// # Arguments
    ///
    /// * `predicate` - Function to test process name
    ///
    /// # Example
    ///
    /// ```ignore
    /// let chrome_indices: Vec<usize> = store
    ///     .filter(|name| name.contains("chrome"))
    ///     .collect();
    /// ```
    pub fn filter<'a, F>(&'a self, mut predicate: F) -> impl Iterator<Item = usize> + 'a
    where
        F: FnMut(&str) -> bool + 'a,
    {
        (0..self.count).filter(move |&i| predicate(&self.names[i]))
    }

    /// Get all PIDs as a slice
    pub fn pids(&self) -> &[u32] {
        &self.pids[..self.count]
    }
}

impl Default for ProcessStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_store_creates() {
        let store = ProcessStore::new();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_process_store_update() {
        let mut store = ProcessStore::new();
        
        let processes = vec![
            ProcessInfo {
                pid: 100,
                parent_pid: 0,
                name: "test.exe".to_string(),
                thread_count: 1,
                handle_count: 10,
                cpu_time_user: 1000,
                cpu_time_kernel: 500,
                memory_working_set: 1024 * 1024,
                memory_pagefile: 512 * 1024,
                memory_private: 256 * 1024,
            },
            ProcessInfo {
                pid: 200,
                parent_pid: 100,
                name: "child.exe".to_string(),
                thread_count: 2,
                handle_count: 20,
                cpu_time_user: 2000,
                cpu_time_kernel: 1000,
                memory_working_set: 2 * 1024 * 1024,
                memory_pagefile: 1024 * 1024,
                memory_private: 512 * 1024,
            },
        ];

        store.update(processes);
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn test_get_by_pid() {
        let mut store = ProcessStore::new();
        
        let processes = vec![
            ProcessInfo {
                pid: 100,
                parent_pid: 0,
                name: "test.exe".to_string(),
                thread_count: 1,
                handle_count: 10,
                cpu_time_user: 1000,
                cpu_time_kernel: 500,
                memory_working_set: 1024 * 1024,
                memory_pagefile: 512 * 1024,
                memory_private: 256 * 1024,
            },
        ];

        store.update(processes);
        
        let index = store.get_by_pid(100);
        assert!(index.is_some());
        assert_eq!(store.name(index.unwrap()), Some("test.exe"));
    }

    #[test]
    fn test_filter() {
        let mut store = ProcessStore::new();
        
        let processes = vec![
            ProcessInfo {
                pid: 100,
                parent_pid: 0,
                name: "chrome.exe".to_string(),
                thread_count: 1,
                handle_count: 10,
                cpu_time_user: 1000,
                cpu_time_kernel: 500,
                memory_working_set: 1024 * 1024,
                memory_pagefile: 512 * 1024,
                memory_private: 256 * 1024,
            },
            ProcessInfo {
                pid: 200,
                parent_pid: 0,
                name: "firefox.exe".to_string(),
                thread_count: 2,
                handle_count: 20,
                cpu_time_user: 2000,
                cpu_time_kernel: 1000,
                memory_working_set: 2 * 1024 * 1024,
                memory_pagefile: 1024 * 1024,
                memory_private: 512 * 1024,
            },
        ];

        store.update(processes);
        
        let chrome_indices: Vec<usize> = store.filter(|name| name.contains("chrome")).collect();
        assert_eq!(chrome_indices.len(), 1);
        assert_eq!(store.name(chrome_indices[0]), Some("chrome.exe"));
    }
}
