use serde::{Deserialize, Serialize};

/// System information structure containing static hardware and software configuration
///
/// This data is gathered once at application startup and cached for the entire session.
/// Represents the computer's hardware specifications and operating system details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
  /// Operating system name (e.g., "Windows 11")
  pub os_name: String,

  /// Operating system version (e.g., "10.0.22631")
  pub os_version: String,

  /// Kernel version
  pub kernel_version: String,

  /// CPU model name (e.g., "Intel(R) Core(TM) i7-9700K")
  pub cpu_model: String,

  /// CPU architecture (e.g., "x86_64")
  pub cpu_architecture: String,

  /// Number of physical CPU cores
  pub cpu_cores: u32,

  /// Total RAM in bytes
  pub total_memory: u64,

  /// Computer hostname
  pub hostname: String,
}

impl SystemInfo {
  /// Gather system information using sysinfo crate
  ///
  /// # Errors
  /// Returns `AppError::SystemInfoError` if system information cannot be retrieved
  pub fn gather() -> crate::error::Result<Self> {
    use sysinfo::System;

    let sys = System::new_all();

    Ok(Self {
      os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
      os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
      kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
      cpu_model: sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string()),
      cpu_architecture: std::env::consts::ARCH.to_string(),
      cpu_cores: sys.cpus().len() as u32,
      total_memory: sys.total_memory(),
      hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_gather_system_info() {
    let info = SystemInfo::gather().expect("Failed to gather system info");

    // Validate non-empty strings
    assert!(!info.os_name.is_empty());
    assert!(!info.hostname.is_empty());

    // Validate positive values
    assert!(info.cpu_cores > 0);
    assert!(info.total_memory > 0);
  }
}
