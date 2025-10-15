/// Tauri command to retrieve static system information
///
/// This command is called once at application startup to gather
/// hardware and software specifications that don't change during runtime.
///
/// # Returns
/// - `Ok(SystemInfo)` with system details
/// - `Err(String)` if system information cannot be retrieved
#[tauri::command]
pub async fn get_system_info() -> Result<crate::models::SystemInfo, String> {
  crate::models::SystemInfo::gather().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_get_system_info() {
    let result = get_system_info().await;
    assert!(result.is_ok());

    let info = result.unwrap();
    assert!(!info.os_name.is_empty());
    assert!(!info.hostname.is_empty());
    assert!(info.cpu_cores > 0);
    assert!(info.total_memory > 0);
  }
}
