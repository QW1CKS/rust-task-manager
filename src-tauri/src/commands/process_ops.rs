use crate::services::process_manager::ProcessManager;
use std::sync::Mutex;
use tauri::State;

/// Tauri command to terminate a process by PID
///
/// # Arguments
/// * `pid` - Process ID to terminate
/// * `manager` - Managed state containing ProcessManager
///
/// # Returns
/// * `Ok(())` - Process successfully terminated
/// * `Err(String)` - Error message explaining why termination failed
///
/// # Errors
/// - "Process with PID {pid} not found" - Process doesn't exist
/// - "Cannot terminate critical system process: {name}" - Attempting to kill critical process (FR-023)
/// - "Failed to terminate process {name} ({pid}). Administrator privileges may be required." - Permission denied (FR-022)
/// - "Process {name} ({pid}) did not terminate after kill signal" - Kill signal sent but process still running
#[tauri::command]
pub async fn kill_process(
  pid: u32,
  manager: State<'_, Mutex<ProcessManager>>,
) -> Result<(), String> {
  let mut manager = manager
    .lock()
    .map_err(|e| format!("Failed to acquire process manager lock: {}", e))?;

  manager.terminate_process(pid).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_kill_nonexistent_process() {
    let mut manager = ProcessManager::new();
    let result = manager.terminate_process(999999);
    assert!(result.is_err());
  }

  #[test]
  fn test_current_process_exists() {
    let mut manager = ProcessManager::new();
    let current_pid = std::process::id();
    assert!(manager.process_exists(current_pid));
  }
}
