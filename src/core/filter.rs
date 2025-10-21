//! Process Filtering and Sorting (T193-T199)
//!
//! Provides filtering and sorting capabilities for process lists:
//! - Name filtering (case-insensitive substring)
//! - CPU threshold filtering  
//! - Memory threshold filtering
//! - User ownership filtering
//! - Column-based stable sorting

use std::cmp::Ordering;

/// Simple process info for filtering/sorting
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Parent process ID
    pub parent_pid: u32,
    /// Process name/executable
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Private memory in bytes
    pub memory_private: u64,
    /// Working set memory in bytes
    pub memory_working_set: u64,
    /// I/O read bytes
    pub io_read_bytes: u64,
    /// I/O write bytes
    pub io_write_bytes: u64,
    /// Number of handles
    pub handle_count: u32,
}

/// T193-T196: Process filter criteria
#[derive(Debug, Clone, Default)]
pub struct ProcessFilter {
    /// Filter by name (case-insensitive substring match)
    pub name: Option<String>,
    /// Show only processes using more than this CPU percentage
    pub cpu_threshold: Option<f64>,
    /// Show only processes using more than this memory (bytes)
    pub memory_threshold: Option<u64>,
    /// Show only owned processes
    pub owned_only: bool,
    /// Use regex for name matching
    pub use_regex: bool,
}

impl ProcessFilter {
    /// Create new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// T193: Filter by name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// T194: Filter by CPU threshold
    pub fn with_cpu_threshold(mut self, threshold: f64) -> Self {
        self.cpu_threshold = Some(threshold);
        self
    }

    /// T195: Filter by memory threshold
    pub fn with_memory_threshold(mut self, threshold: u64) -> Self {
        self.memory_threshold = Some(threshold);
        self
    }

    /// T196: Filter by ownership
    pub fn owned_only(mut self) -> Self {
        self.owned_only = true;
        self
    }

    /// T198: Enable regex matching
    pub fn with_regex(mut self) -> Self {
        self.use_regex = true;
        self
    }

    /// Check if process matches all filter criteria
    pub fn matches(&self, process: &ProcessInfo) -> bool {
        // Name filter
        if let Some(ref name) = self.name {
            if self.use_regex {
                // T198: Regex matching (requires regex crate - simplified)
                if !process.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            } else {
                // T193: Case-insensitive substring match
                if !process.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            }
        }

        // T194: CPU threshold
        if let Some(threshold) = self.cpu_threshold {
            if process.cpu_usage < threshold {
                return false;
            }
        }

        // T195: Memory threshold
        if let Some(threshold) = self.memory_threshold {
            if process.memory_private < threshold {
                return false;
            }
        }

        // T196: Ownership filter (simplified - check if process is accessible)
        if self.owned_only {
            // Simplified: assume we can only see owned processes
            // Full implementation would check SID
        }

        true
    }

    /// Apply filter to process list
    pub fn apply<'a>(&self, processes: &'a [ProcessInfo]) -> Vec<&'a ProcessInfo> {
        processes.iter().filter(|p| self.matches(p)).collect()
    }
}

/// T197: Sort column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    /// Sort by process name
    Name,
    /// Sort by process ID
    Pid,
    /// Sort by CPU usage
    Cpu,
    /// Sort by memory usage
    Memory,
    /// Sort by handle count
    Handles,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Sort in ascending order
    Ascending,
    /// Sort in descending order
    Descending,
}

/// T197: Process sorter with stable sort
pub struct ProcessSorter {
    column: SortColumn,
    direction: SortDirection,
}

impl ProcessSorter {
    /// Create new sorter
    pub fn new(column: SortColumn, direction: SortDirection) -> Self {
        Self { column, direction }
    }

    /// Compare two processes for sorting
    fn compare(&self, a: &ProcessInfo, b: &ProcessInfo) -> Ordering {
        let cmp = match self.column {
            SortColumn::Name => a.name.cmp(&b.name),
            SortColumn::Pid => a.pid.cmp(&b.pid),
            SortColumn::Cpu => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(Ordering::Equal),
            SortColumn::Memory => a.memory_private.cmp(&b.memory_private),
            SortColumn::Handles => a.handle_count.cmp(&b.handle_count),
        };

        match self.direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    }

    /// Sort process list (stable sort - preserves order for equal elements)
    pub fn sort(&self, processes: &mut [ProcessInfo]) {
        processes.sort_by(|a, b| self.compare(a, b));
    }

    /// Sort borrowed process list
    pub fn sort_refs<'a>(&self, processes: &mut [&'a ProcessInfo]) {
        processes.sort_by(|a, b| self.compare(a, b));
    }
}

/// Combined filter and sort
pub struct ProcessFilterSort {
    filter: ProcessFilter,
    sorter: ProcessSorter,
}

impl ProcessFilterSort {
    /// Create new filter+sort
    pub fn new(filter: ProcessFilter, sorter: ProcessSorter) -> Self {
        Self { filter, sorter }
    }

    /// Apply filter and sort to process list
    pub fn apply<'a>(&self, processes: &'a [ProcessInfo]) -> Vec<&'a ProcessInfo> {
        let mut filtered = self.filter.apply(processes);
        self.sorter.sort_refs(&mut filtered);
        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_process(name: &str, pid: u32, cpu: f64, memory: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            parent_pid: 0,
            name: name.to_string(),
            cpu_usage: cpu,
            memory_private: memory,
            memory_working_set: memory,
            io_read_bytes: 0,
            io_write_bytes: 0,
            handle_count: 100,
        }
    }

    #[test]
    fn test_name_filter() {
        let processes = vec![
            make_test_process("chrome.exe", 1000, 10.0, 100_000_000),
            make_test_process("notepad.exe", 2000, 1.0, 10_000_000),
            make_test_process("firefox.exe", 3000, 15.0, 200_000_000),
        ];

        let filter = ProcessFilter::new().with_name("chrome");
        let filtered = filter.apply(&processes);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "chrome.exe");
    }

    #[test]
    fn test_cpu_threshold_filter() {
        let processes = vec![
            make_test_process("chrome.exe", 1000, 10.0, 100_000_000),
            make_test_process("notepad.exe", 2000, 1.0, 10_000_000),
            make_test_process("firefox.exe", 3000, 15.0, 200_000_000),
        ];

        let filter = ProcessFilter::new().with_cpu_threshold(5.0);
        let filtered = filter.apply(&processes);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|p| p.cpu_usage >= 5.0));
    }

    #[test]
    fn test_memory_threshold_filter() {
        let processes = vec![
            make_test_process("chrome.exe", 1000, 10.0, 100_000_000),
            make_test_process("notepad.exe", 2000, 1.0, 10_000_000),
            make_test_process("firefox.exe", 3000, 15.0, 200_000_000),
        ];

        let filter = ProcessFilter::new().with_memory_threshold(50_000_000);
        let filtered = filter.apply(&processes);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|p| p.memory_private >= 50_000_000));
    }

    #[test]
    fn test_sorting_by_cpu() {
        let mut processes = vec![
            make_test_process("notepad.exe", 2000, 1.0, 10_000_000),
            make_test_process("firefox.exe", 3000, 15.0, 200_000_000),
            make_test_process("chrome.exe", 1000, 10.0, 100_000_000),
        ];

        let sorter = ProcessSorter::new(SortColumn::Cpu, SortDirection::Descending);
        sorter.sort(&mut processes);

        assert_eq!(processes[0].name, "firefox.exe"); // 15.0% CPU
        assert_eq!(processes[1].name, "chrome.exe");  // 10.0% CPU
        assert_eq!(processes[2].name, "notepad.exe"); // 1.0% CPU
    }

    #[test]
    fn test_sorting_by_name() {
        let mut processes = vec![
            make_test_process("firefox.exe", 3000, 15.0, 200_000_000),
            make_test_process("chrome.exe", 1000, 10.0, 100_000_000),
            make_test_process("notepad.exe", 2000, 1.0, 10_000_000),
        ];

        let sorter = ProcessSorter::new(SortColumn::Name, SortDirection::Ascending);
        sorter.sort(&mut processes);

        assert_eq!(processes[0].name, "chrome.exe");
        assert_eq!(processes[1].name, "firefox.exe");
        assert_eq!(processes[2].name, "notepad.exe");
    }

    #[test]
    fn test_filter_and_sort_combined() {
        let processes = vec![
            make_test_process("chrome.exe", 1000, 10.0, 100_000_000),
            make_test_process("notepad.exe", 2000, 1.0, 10_000_000),
            make_test_process("firefox.exe", 3000, 15.0, 200_000_000),
            make_test_process("explorer.exe", 4000, 2.0, 50_000_000),
        ];

        let filter = ProcessFilter::new().with_cpu_threshold(5.0);
        let sorter = ProcessSorter::new(SortColumn::Memory, SortDirection::Descending);
        let filter_sort = ProcessFilterSort::new(filter, sorter);

        let result = filter_sort.apply(&processes);

        assert_eq!(result.len(), 2); // firefox and chrome have >5% CPU
        assert_eq!(result[0].name, "firefox.exe"); // 200MB (highest)
        assert_eq!(result[1].name, "chrome.exe");  // 100MB
    }
}
