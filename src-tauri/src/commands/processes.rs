/// Tauri command to retrieve list of all running processes
///
/// This command is called repeatedly (every 1-2 seconds) to get
/// the current process list with CPU, memory, and status information.
///
/// # Returns
/// - `Ok(Vec<ProcessInfo>)` with all running processes
/// - `Err(String)` if process list cannot be retrieved
#[tauri::command]
pub async fn get_processes() -> Result<Vec<crate::models::ProcessInfo>, String> {
  crate::services::system_monitor::get_process_list().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_get_processes() {
    let result = get_processes().await;
    assert!(result.is_ok());

    let processes = result.unwrap();
    assert!(!processes.is_empty());
    assert!(processes.iter().any(|p| p.pid > 0));
    assert!(processes.iter().any(|p| !p.name.is_empty()));
  }

  #[tokio::test]
  async fn test_get_processes_returns_process_info() {
    let result = get_processes().await;
    assert!(result.is_ok());

    let processes = result.unwrap();
    // Verify at least one process has meaningful data
    let valid_process = processes.iter().find(|p| p.memory_bytes > 0);
    assert!(valid_process.is_some());
  }
}
