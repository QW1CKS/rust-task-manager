/// Critical Windows system processes that should never be terminated
///
/// These processes are essential for Windows operation. Attempting to
/// terminate them will result in system instability or immediate BSOD.
pub const CRITICAL_PROCESSES: &[&str] = &[
  "csrss.exe",    // Client/Server Runtime Subsystem
  "wininit.exe",  // Windows Initialization Process
  "services.exe", // Service Control Manager
  "smss.exe",     // Session Manager Subsystem
  "lsass.exe",    // Local Security Authority Subsystem Service
];

/// Check if a process name matches a critical system process
///
/// Case-insensitive comparison.
///
/// # Examples
/// ```
/// assert!(is_critical_process("csrss.exe"));
/// assert!(is_critical_process("LSASS.EXE"));
/// assert!(!is_critical_process("notepad.exe"));
/// ```
pub fn is_critical_process(process_name: &str) -> bool {
  let name_lower = process_name.to_lowercase();
  CRITICAL_PROCESSES
    .iter()
    .any(|critical| name_lower == critical.to_lowercase())
}

/// Additional Windows-specific utility functions
#[cfg(target_os = "windows")]
pub mod windows_specific {
  /// Check if the current process has administrator privileges
  ///
  /// # Note
  /// This is a placeholder for future implementation using Windows API
  pub fn is_elevated() -> bool {
    // TODO: Implement using Windows API in future phases
    false
  }

  /// Request administrator privileges via UAC
  ///
  /// # Note
  /// This is a placeholder for future implementation
  pub fn request_elevation() -> crate::error::Result<()> {
    // TODO: Implement UAC elevation in future phases
    Err(crate::error::AppError::AccessDenied(
      "UAC elevation not yet implemented".to_string(),
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_critical_process() {
    // Test critical processes
    assert!(is_critical_process("csrss.exe"));
    assert!(is_critical_process("CSRSS.EXE"));
    assert!(is_critical_process("lsass.exe"));
    assert!(is_critical_process("LSASS.EXE"));
    assert!(is_critical_process("services.exe"));
    assert!(is_critical_process("wininit.exe"));
    assert!(is_critical_process("smss.exe"));

    // Test non-critical processes
    assert!(!is_critical_process("notepad.exe"));
    assert!(!is_critical_process("explorer.exe"));
    assert!(!is_critical_process("chrome.exe"));
    assert!(!is_critical_process(""));
  }

  #[test]
  fn test_critical_processes_list() {
    assert_eq!(CRITICAL_PROCESSES.len(), 5);
    assert!(CRITICAL_PROCESSES.contains(&"csrss.exe"));
  }
}
