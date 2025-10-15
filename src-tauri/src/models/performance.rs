use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Performance metrics representing a real-time resource utilization snapshot
///
/// Collected every 1-2 seconds and maintained in a 60-point rolling buffer on the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMetrics {
  /// Timestamp when metrics were collected (Unix epoch milliseconds)
  pub timestamp: u64,

  /// Overall CPU usage percentage (0.0 - 100.0)
  pub cpu_usage_percent: f32,

  /// Per-core CPU usage percentages
  pub cpu_per_core: Vec<f32>,

  /// Used memory in bytes
  pub memory_used: u64,

  /// Total memory in bytes
  pub memory_total: u64,

  /// Memory usage percentage (0.0 - 100.0)
  pub memory_percent: f32,

  /// Disk read speed in bytes/second
  pub disk_read_bps: u64,

  /// Disk write speed in bytes/second
  pub disk_write_bps: u64,

  /// Network upload speed in bytes/second
  pub network_upload_bps: u64,

  /// Network download speed in bytes/second
  pub network_download_bps: u64,
}

impl PerformanceMetrics {
  /// Collect current performance metrics
  ///
  /// # Errors
  /// Returns `AppError::PerformanceError` if metrics cannot be collected
  pub fn collect(system: &mut sysinfo::System) -> crate::error::Result<Self> {
    // Refresh system data
    system.refresh_cpu_all();
    system.refresh_memory();

    // Get timestamp
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_err(|e| {
        crate::error::AppError::PerformanceError(format!("Failed to get timestamp: {}", e))
      })?
      .as_millis() as u64;

    // Calculate CPU usage
    let cpu_usage_percent = system.global_cpu_usage();
    let cpu_per_core = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    // Get memory info
    let memory_used = system.used_memory();
    let memory_total = system.total_memory();
    let memory_percent = Self::calculate_memory_percent(memory_used, memory_total);

    // TODO: Implement disk and network metrics in future phases
    // For now, return placeholder values
    Ok(Self {
      timestamp,
      cpu_usage_percent,
      cpu_per_core,
      memory_used,
      memory_total,
      memory_percent,
      disk_read_bps: 0,
      disk_write_bps: 0,
      network_upload_bps: 0,
      network_download_bps: 0,
    })
  }

  /// Calculate memory percentage from used/total
  pub fn calculate_memory_percent(used: u64, total: u64) -> f32 {
    if total == 0 {
      0.0
    } else {
      (used as f64 / total as f64 * 100.0) as f32
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_calculate_memory_percent() {
    assert_eq!(PerformanceMetrics::calculate_memory_percent(0, 100), 0.0);
    assert_eq!(PerformanceMetrics::calculate_memory_percent(50, 100), 50.0);
    assert_eq!(
      PerformanceMetrics::calculate_memory_percent(100, 100),
      100.0
    );
    assert_eq!(PerformanceMetrics::calculate_memory_percent(100, 0), 0.0);
  }

  #[test]
  fn test_collect_performance_metrics() {
    let mut sys = sysinfo::System::new_all();
    let metrics = PerformanceMetrics::collect(&mut sys).expect("Failed to collect metrics");

    // Validate timestamp
    assert!(metrics.timestamp > 0);

    // Validate CPU metrics
    assert!(metrics.cpu_usage_percent >= 0.0 && metrics.cpu_usage_percent <= 100.0);
    assert!(!metrics.cpu_per_core.is_empty());

    // Validate memory metrics
    assert!(metrics.memory_total > 0);
    assert!(metrics.memory_used <= metrics.memory_total);
    assert!(metrics.memory_percent >= 0.0 && metrics.memory_percent <= 100.0);
  }
}
