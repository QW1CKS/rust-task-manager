use serde::{Deserialize, Serialize};

/// Process execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
  Running,
  Sleeping,
  Stopped,
  Other,
}

/// Process information structure representing a running process or service
///
/// Dynamic - collected every refresh cycle, can appear/disappear between polls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
  /// Process ID (unique identifier)
  pub pid: u32,

  /// Process name (executable name)
  pub name: String,

  /// Full executable path (may be None if inaccessible)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub exe_path: Option<String>,

  /// Command-line arguments (may be None if inaccessible)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cmd_args: Option<Vec<String>>,

  /// CPU usage percentage (0.0 - 100.0)
  pub cpu_percent: f32,

  /// Memory usage in bytes
  pub memory_bytes: u64,

  /// Current process status
  pub status: ProcessStatus,

  /// Parent process ID (0 if no parent or inaccessible)
  pub parent_pid: u32,

  /// Process start time (Unix epoch seconds)
  pub start_time: u64,

  /// User account running the process (may be None if inaccessible)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,
}

impl ProcessInfo {
  /// Convert from sysinfo::Process
  pub fn from_sysinfo(pid: sysinfo::Pid, process: &sysinfo::Process) -> Self {
    // Convert process status
    let status = match process.status() {
      sysinfo::ProcessStatus::Run => ProcessStatus::Running,
      sysinfo::ProcessStatus::Sleep => ProcessStatus::Sleeping,
      sysinfo::ProcessStatus::Stop => ProcessStatus::Stopped,
      _ => ProcessStatus::Other,
    };

    // Get executable path
    let exe_path = process
      .exe()
      .and_then(|p| p.to_str())
      .map(|s| s.to_string());

    // Get command-line arguments
    let cmd_args = {
      let args = process.cmd();
      if args.is_empty() {
        None
      } else {
        Some(
          args
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect(),
        )
      }
    };

    // Get parent PID
    let parent_pid = process.parent().map(|p| p.as_u32()).unwrap_or(0);

    // Get start time (Unix epoch seconds)
    let start_time = process.start_time();

    // Get user (not available on all platforms)
    #[cfg(target_os = "windows")]
    let user = process.user_id().and(None); // TODO: Implement Windows user lookup

    #[cfg(not(target_os = "windows"))]
    let user = process.user_id().map(|uid| uid.to_string());

    Self {
      pid: pid.as_u32(),
      name: process.name().to_string_lossy().to_string(),
      exe_path,
      cmd_args,
      cpu_percent: process.cpu_usage(),
      memory_bytes: process.memory(),
      status,
      parent_pid,
      start_time,
      user,
    }
  }

  /// Check if this is a critical Windows system process
  pub fn is_critical(&self) -> bool {
    const CRITICAL: &[&str] = &[
      "csrss.exe",
      "wininit.exe",
      "services.exe",
      "lsass.exe",
      "smss.exe",
    ];

    CRITICAL.contains(&self.name.to_lowercase().as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_critical() {
    let mut process = ProcessInfo {
      pid: 1,
      name: "csrss.exe".to_string(),
      exe_path: None,
      cmd_args: None,
      cpu_percent: 0.0,
      memory_bytes: 0,
      status: ProcessStatus::Running,
      parent_pid: 0,
      start_time: 0,
      user: None,
    };

    assert!(process.is_critical());

    process.name = "notepad.exe".to_string();
    assert!(!process.is_critical());

    process.name = "LSASS.EXE".to_string();
    assert!(process.is_critical());
  }
}
