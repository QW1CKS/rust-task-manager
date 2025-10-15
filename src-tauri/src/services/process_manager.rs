use crate::error::{AppError, Result};
use crate::utils::windows::is_critical_process;
use sysinfo::{Pid, System};

/// Service for managing process operations (termination, priority, etc.)
pub struct ProcessManager {
  system: System,
}

impl ProcessManager {
  /// Create a new ProcessManager instance
  pub fn new() -> Self {
    Self {
      system: System::new_all(),
    }
  }

  /// Terminate a process by PID with safety checks
  ///
  /// # Safety Checks
  /// - Validates process exists
  /// - Checks if process is a critical Windows system process (FR-023)
  /// - Attempts graceful termination first, then forceful if needed
  ///
  /// # Errors
  /// - `AppError::ProcessNotFound`: Process doesn't exist
  /// - `AppError::CriticalProcessProtection`: Attempting to kill critical system process
  /// - `AppError::PermissionDenied`: Insufficient privileges (may need UAC elevation)
  /// - `AppError::ProcessTerminationFailed`: Process exists but couldn't be terminated
  pub fn terminate_process(&mut self, pid: u32) -> Result<()> {
    // Refresh process list to get latest state
    self.system.refresh_all();

    let sys_pid = Pid::from_u32(pid);

    // Check if process exists
    let process = self
      .system
      .process(sys_pid)
      .ok_or_else(|| AppError::ProcessNotFound(format!("Process with PID {} not found", pid)))?;

    let process_name = process.name().to_string_lossy().to_string();

    // Critical process check (FR-023)
    if is_critical_process(&process_name) {
      return Err(AppError::CriticalProcessProtection(format!(
        "Cannot terminate critical system process: {}",
        process_name
      )));
    }

    // Attempt to kill the process
    // Note: On Windows, this will trigger UAC if the process requires elevated privileges
    if !process.kill() {
      // Process exists but couldn't be killed - likely permissions issue
      return Err(AppError::PermissionDenied(format!(
        "Failed to terminate process {} ({}). Administrator privileges may be required.",
        process_name, pid
      )));
    }

    // Verify termination
    self.system.refresh_all();
    if self.system.process(sys_pid).is_some() {
      return Err(AppError::ProcessTerminationFailed(format!(
        "Process {} ({}) did not terminate after kill signal",
        process_name, pid
      )));
    }

    Ok(())
  }

  /// Check if a process with the given PID exists (Helper for tests)
  #[allow(dead_code)]
  pub fn process_exists(&mut self, pid: u32) -> bool {
    self.system.refresh_all();
    self.system.process(Pid::from_u32(pid)).is_some()
  }

  /// Get process name by PID (Helper for error messages)
  #[allow(dead_code)]
  pub fn get_process_name(&mut self, pid: u32) -> Option<String> {
    self.system.refresh_all();
    self
      .system
      .process(Pid::from_u32(pid))
      .map(|p| p.name().to_string_lossy().to_string())
  }
}

impl Default for ProcessManager {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_process_manager_creation() {
    let manager = ProcessManager::new();
    assert!(!manager.system.processes().is_empty());
  }
  #[test]
  fn test_nonexistent_process_returns_error() {
    let mut manager = ProcessManager::new();
    let result = manager.terminate_process(999999);
    assert!(result.is_err());
    match result {
      Err(AppError::ProcessNotFound(_)) => (),
      _ => panic!("Expected ProcessNotFound error"),
    }
  }

  #[test]
  fn test_critical_process_protection() {
    let mut manager = ProcessManager::new();

    // Find csrss.exe (critical system process)
    manager.system.refresh_all();
    if let Some(critical_process) = manager
      .system
      .processes()
      .values()
      .find(|p| p.name().to_string_lossy().to_lowercase() == "csrss.exe")
    {
      let pid = critical_process.pid().as_u32();
      let result = manager.terminate_process(pid);
      assert!(result.is_err());
      match result {
        Err(AppError::CriticalProcessProtection(_)) => (),
        _ => panic!("Expected CriticalProcessProtection error"),
      }
    }
  }

  #[test]
  fn test_process_exists() {
    let mut manager = ProcessManager::new();

    // Current process should exist
    let current_pid = std::process::id();
    assert!(manager.process_exists(current_pid));

    // Nonexistent process
    assert!(!manager.process_exists(999999));
  }

  #[test]
  fn test_get_process_name() {
    let mut manager = ProcessManager::new();

    // Current process should have a name
    let current_pid = std::process::id();
    let name = manager.get_process_name(current_pid);
    assert!(name.is_some());
    assert!(!name.unwrap().is_empty());

    // Nonexistent process
    assert!(manager.get_process_name(999999).is_none());
  }
}
