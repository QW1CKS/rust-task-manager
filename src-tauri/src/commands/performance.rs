/// Tauri command to retrieve real-time performance metrics
///
/// This command is called repeatedly (every 1-2 seconds) to get
/// current CPU, memory, disk, and network usage statistics.
///
/// # Returns
/// - `Ok(PerformanceMetrics)` with current system metrics
/// - `Err(String)` if metrics cannot be collected
#[tauri::command]
pub async fn get_performance_data() -> Result<crate::models::PerformanceMetrics, String> {
  crate::services::system_monitor::collect_performance_metrics().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_get_performance_data() {
    let result = get_performance_data().await;
    assert!(result.is_ok());

    let metrics = result.unwrap();
    assert!(metrics.timestamp > 0);
    assert!(metrics.cpu_usage_percent >= 0.0);
    assert!(metrics.cpu_usage_percent <= 100.0);
    assert!(metrics.memory_total > 0);
    assert!(metrics.memory_percent >= 0.0);
    assert!(metrics.memory_percent <= 100.0);
  }
}
