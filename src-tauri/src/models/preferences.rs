use serde::{Deserialize, Serialize};

/// Theme mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
  Dark,
  Light,
}

impl Default for ThemeMode {
  fn default() -> Self {
    Self::Dark // Constitution: dark mode default
  }
}

/// Window state (position and size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
  pub width: u32,
  pub height: u32,
  pub x: i32,
  pub y: i32,
  pub maximized: bool,
}

impl Default for WindowState {
  fn default() -> Self {
    Self {
      width: 1200,
      height: 800,
      x: 0, // Centered by Tauri
      y: 0,
      maximized: false,
    }
  }
}

/// User preferences structure for persisted application settings
///
/// Stored in %APPDATA%\rust-task-manager\config.json on Windows
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
  /// Theme mode (dark/light)
  pub theme: ThemeMode,

  /// Window position and size
  pub window: WindowState,

  /// Last selected sort column for process list
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sort_column: Option<String>,

  /// Last selected sort order
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sort_order: Option<String>,
}

impl UserPreferences {
  /// Load preferences from config file
  ///
  /// # Errors
  /// Returns default preferences if file doesn't exist or cannot be read
  pub fn load() -> crate::error::Result<Self> {
    // TODO: Implement file loading in future phases
    // For now, return default preferences
    Ok(Self::default())
  }

  /// Save preferences to config file
  ///
  /// # Errors
  /// Returns `AppError::IoError` if file cannot be written
  pub fn save(&self) -> crate::error::Result<()> {
    // TODO: Implement file saving in future phases
    // For now, do nothing
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_preferences() {
    let prefs = UserPreferences::default();

    assert_eq!(prefs.theme, ThemeMode::Dark);
    assert_eq!(prefs.window.width, 1200);
    assert_eq!(prefs.window.height, 800);
    assert!(!prefs.window.maximized);
  }

  #[test]
  fn test_serialization() {
    let prefs = UserPreferences::default();
    let json = serde_json::to_string(&prefs).expect("Failed to serialize");

    assert!(json.contains("\"theme\":\"dark\""));
    assert!(json.contains("\"width\":1200"));
  }
}
