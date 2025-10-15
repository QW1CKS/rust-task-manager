use once_cell::sync::Lazy;
use std::sync::Mutex;
use sysinfo::System;

/// Global system monitor instance
///
/// Uses once_cell::Lazy for thread-safe lazy initialization.
/// Wrapped in Mutex to allow mutable access for refreshing system data.
pub static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new_all()));

/// Refresh system data and collect performance metrics
///
/// # Errors
/// Returns `AppError::PerformanceError` if system data cannot be refreshed
pub fn collect_performance_metrics() -> crate::error::Result<crate::models::PerformanceMetrics> {
  let mut system = SYSTEM.lock().map_err(|e| {
    crate::error::AppError::PerformanceError(format!("Failed to lock system: {}", e))
  })?;

  crate::models::PerformanceMetrics::collect(&mut system)
}

/// Get list of all running processes
///
/// # Errors
/// Returns `AppError::PerformanceError` if process list cannot be retrieved
pub fn get_process_list() -> crate::error::Result<Vec<crate::models::ProcessInfo>> {
  use sysinfo::ProcessesToUpdate;

  let mut system = SYSTEM.lock().map_err(|e| {
    crate::error::AppError::PerformanceError(format!("Failed to lock system: {}", e))
  })?;

  system.refresh_processes(ProcessesToUpdate::All, true);

  let processes = system
    .processes()
    .iter()
    .map(|(pid, process)| crate::models::ProcessInfo::from_sysinfo(*pid, process))
    .collect();

  Ok(processes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_collect_performance_metrics() {
    let metrics = collect_performance_metrics().expect("Failed to collect metrics");

    assert!(metrics.timestamp > 0);
    assert!(metrics.cpu_usage_percent >= 0.0);
    assert!(metrics.memory_total > 0);
  }

  #[test]
  fn test_get_process_list() {
    let processes = get_process_list().expect("Failed to get process list");

    assert!(!processes.is_empty());
    assert!(processes.iter().any(|p| p.pid > 0));
  }
}
