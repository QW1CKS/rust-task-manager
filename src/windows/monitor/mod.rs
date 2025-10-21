//! System monitoring using Windows APIs

pub mod dxgi;
pub mod memory;
pub mod nt_query;
pub mod pdh;

use crate::core::metrics::SystemMetrics;
use std::time::Instant;

/// Process snapshot from system monitoring
///
/// # Ownership Model
///
/// SystemMonitor::collect_all() returns owned ProcessSnapshot.
/// Caller takes ownership and can pass to ProcessStore::update().
/// SystemMonitor retains no references to collected data.
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
    /// List of processes
    pub processes: Vec<nt_query::ProcessInfo>,
    /// System-wide metrics at time of snapshot
    pub system_metrics: SystemMetrics,
}

/// System monitor coordinating all data collectors
///
/// # Performance Target
///
/// collect_all() must complete in <50ms for constitutional compliance
///
/// # Error Handling
///
/// If any collector fails, returns partial data with error.
/// UI can display last-known-good data with staleness indicator.
pub struct SystemMonitor {
    /// Process enumerator (reuses 1MB buffer)
    process_enum: nt_query::ProcessEnumerator,
}

impl SystemMonitor {
    /// Create a new system monitor
    ///
    /// # Memory Allocation
    ///
    /// Allocates 1MB buffer for process enumeration.
    pub fn new() -> Self {
        Self {
            process_enum: nt_query::ProcessEnumerator::new(),
        }
    }

    /// Collect all system data
    ///
    /// # Performance
    ///
    /// Target: <50ms total
    /// - Process enumeration: <5ms
    /// - Memory metrics: <1ms
    /// - Total: <10ms typical
    ///
    /// # Returns
    ///
    /// Ok(ProcessSnapshot) with complete system state, or Err if collection fails
    pub fn collect_all(&mut self) -> Result<ProcessSnapshot, String> {
        let start = Instant::now();

        // Collect process list
        let processes = self.process_enum.enumerate_processes()?;

        // Collect memory metrics
        let memory = memory::get_memory_metrics()?;

        // Build system metrics
        let mut system_metrics = SystemMetrics::new();
        system_metrics.memory_total = memory.total_physical;
        system_metrics.memory_available = memory.available_physical;
        system_metrics.memory_load_percent = memory.load_percent;

        let elapsed = start.elapsed();
        
        // Log if we exceed performance budget
        if elapsed.as_millis() > 50 {
            eprintln!("Warning: collect_all() took {}ms (target <50ms)", elapsed.as_millis());
        }

        Ok(ProcessSnapshot {
            timestamp: Instant::now(),
            processes,
            system_metrics,
        })
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creates() {
        let _monitor = SystemMonitor::new();
    }

    #[test]
    fn test_collect_all() {
        let mut monitor = SystemMonitor::new();
        let result = monitor.collect_all();
        
        assert!(result.is_ok(), "collect_all should succeed");
        
        let snapshot = result.unwrap();
        assert!(!snapshot.processes.is_empty(), "Should find processes");
        assert!(snapshot.system_metrics.memory_total > 0, "Should have memory info");
    }

    #[test]
    fn test_collect_all_performance() {
        let mut monitor = SystemMonitor::new();
        let start = Instant::now();
        
        let _ = monitor.collect_all();
        
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 50,
            "collect_all should complete in <50ms, took {}ms",
            elapsed.as_millis()
        );
    }
}
