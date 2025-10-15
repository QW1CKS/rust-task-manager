use thiserror::Error;

/// Application error types for the Rust Task Manager
///
/// All errors implement std::error::Error via thiserror derive macro
/// and can be serialized to JSON for IPC communication with frontend.
#[derive(Error, Debug)]
#[allow(dead_code)] // Temporary: will be used in Phase 3 Tauri commands
pub enum AppError {
  #[error("System information unavailable: {0}")]
  SystemInfoError(String),

  #[error("Performance metrics collection failed: {0}")]
  PerformanceError(String),

  #[error("Process not found: {0}")]
  ProcessNotFound(String),

  #[error("Permission denied: {0}")]
  PermissionDenied(String),

  #[error("Access denied: {0}")]
  AccessDenied(String),

  #[error("Cannot terminate critical system process: {0}")]
  CriticalProcessProtection(String),

  #[error("Process termination failed: {0}")]
  ProcessTerminationFailed(String),

  #[error("Process termination failed: {0}")]
  TerminationFailed(String),

  #[error("I/O error: {0}")]
  IoError(#[from] std::io::Error),

  #[error("Serialization error: {0}")]
  SerializationError(#[from] serde_json::Error),
}

// Implement Serialize for AppError so it can be sent to frontend
impl serde::Serialize for AppError {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;
