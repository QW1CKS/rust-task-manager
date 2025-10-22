//! Application configuration persistence with Windows Registry
//!
//! Implements T423-T428:
//! - Registry storage at HKCU\Software\TaskManager
//! - Window position and size persistence
//! - Theme preference storage
//! - Refresh rate and history length
//! - Column widths and visibility
//! - Import/export to JSON

use windows::Win32::Foundation::RECT;
use windows::Win32::System::Registry::*;
use windows::core::PCWSTR;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

use crate::app::theme::Theme;

/// Registry key path
#[allow(dead_code)]
const REGISTRY_KEY: &str = "Software\\TaskManager";

/// Configuration data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Window position and size
    pub window: WindowConfig,
    /// Theme preference
    pub theme: ThemeConfig,
    /// Monitoring settings
    pub monitoring: MonitoringConfig,
    /// Table column settings
    pub columns: ColumnConfig,
    /// Startup options
    pub startup: StartupConfig,
}

/// Window position and size (T424)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
}

/// Theme configuration (T425)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub preference: Theme,
}

/// Monitoring settings (T425)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Refresh rate in milliseconds (100, 500, 1000, 2000, 5000, 10000)
    pub refresh_rate_ms: u32,
    /// History length in seconds (60, 300, 3600, 86400)
    pub history_length_sec: u32,
    /// Graph type: 0=Line, 1=Area, 2=Both
    pub graph_type: u32,
}

/// Column configuration (T426)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnConfig {
    /// Column widths (name -> width in pixels)
    pub widths: std::collections::HashMap<String, i32>,
    /// Column visibility (name -> visible)
    pub visibility: std::collections::HashMap<String, bool>,
}

/// Startup options (T420)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Run at login
    pub run_at_login: bool,
    /// Start minimized
    pub start_minimized: bool,
    /// Performance mode (disable animations)
    pub performance_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                x: 100,
                y: 100,
                width: 1200,
                height: 800,
                maximized: false,
            },
            theme: ThemeConfig {
                preference: Theme::System,
            },
            monitoring: MonitoringConfig {
                refresh_rate_ms: 1000,
                history_length_sec: 60,
                graph_type: 0, // Line
            },
            columns: ColumnConfig {
                widths: Default::default(),
                visibility: Default::default(),
            },
            startup: StartupConfig {
                run_at_login: false,
                start_minimized: false,
                performance_mode: false,
            },
        }
    }
}

/// Configuration manager with async loading and registry backend
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    dirty: Arc<RwLock<bool>>,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            dirty: Arc::new(RwLock::new(false)),
        }
    }

    /// Load configuration from registry (T427)
    ///
    /// Target: <50ms load time
    pub fn load(&self) -> Result<(), Box<dyn std::error::Error>> {
        let loaded = Self::load_from_registry()?;
        
        let mut config = self.config.write().unwrap();
        *config = loaded;
        
        Ok(())
    }

    /// Save configuration to registry
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        Self::save_to_registry(&config)?;
        
        let mut dirty = self.dirty.write().unwrap();
        *dirty = false;
        
        Ok(())
    }

    /// Get current configuration (read-only)
    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    /// Update window configuration (T424)
    pub fn set_window_bounds(&self, bounds: RECT, maximized: bool) {
        let mut config = self.config.write().unwrap();
        config.window.x = bounds.left;
        config.window.y = bounds.top;
        config.window.width = bounds.right - bounds.left;
        config.window.height = bounds.bottom - bounds.top;
        config.window.maximized = maximized;
        
        *self.dirty.write().unwrap() = true;
    }

    /// Update theme preference (T425)
    pub fn set_theme(&self, theme: Theme) {
        let mut config = self.config.write().unwrap();
        config.theme.preference = theme;
        
        *self.dirty.write().unwrap() = true;
    }

    /// Update monitoring settings (T425)
    pub fn set_monitoring(&self, refresh_rate_ms: u32, history_length_sec: u32, graph_type: u32) {
        let mut config = self.config.write().unwrap();
        config.monitoring.refresh_rate_ms = refresh_rate_ms;
        config.monitoring.history_length_sec = history_length_sec;
        config.monitoring.graph_type = graph_type;
        
        *self.dirty.write().unwrap() = true;
    }

    /// Update column width (T426)
    pub fn set_column_width(&self, column_name: &str, width: i32) {
        let mut config = self.config.write().unwrap();
        config.columns.widths.insert(column_name.to_string(), width);
        
        *self.dirty.write().unwrap() = true;
    }

    /// Update column visibility (T426)
    pub fn set_column_visibility(&self, column_name: &str, visible: bool) {
        let mut config = self.config.write().unwrap();
        config.columns.visibility.insert(column_name.to_string(), visible);
        
        *self.dirty.write().unwrap() = true;
    }

    /// Update startup options
    pub fn set_startup_options(&self, run_at_login: bool, start_minimized: bool, performance_mode: bool) {
        let mut config = self.config.write().unwrap();
        config.startup.run_at_login = run_at_login;
        config.startup.start_minimized = start_minimized;
        config.startup.performance_mode = performance_mode;
        
        *self.dirty.write().unwrap() = true;
    }

    /// Export configuration to JSON file (T428)
    pub fn export_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        let json = serde_json::to_string_pretty(&*config)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Import configuration from JSON file (T428)
    pub fn import_from_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let loaded: AppConfig = serde_json::from_str(&json)?;
        
        let mut config = self.config.write().unwrap();
        *config = loaded;
        
        *self.dirty.write().unwrap() = true;
        Ok(())
    }

    /// Check if configuration has unsaved changes
    pub fn is_dirty(&self) -> bool {
        *self.dirty.read().unwrap()
    }

    /// Load from Windows Registry
    fn load_from_registry() -> Result<AppConfig, Box<dyn std::error::Error>> {
        let mut config = AppConfig::default();

        unsafe {
            let mut hkey = Default::default();
            let subkey = windows::core::w!("Software\\TaskManager");
            
            // Open registry key (read-only)
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey,
                Some(0),
                KEY_READ,
                &mut hkey,
            ).is_err() {
                // Key doesn't exist yet, use defaults
                return Ok(config);
            }

            // Load window position (T424)
            if let Ok(x) = Self::read_dword(hkey, "WindowX") {
                config.window.x = x as i32;
            }
            if let Ok(y) = Self::read_dword(hkey, "WindowY") {
                config.window.y = y as i32;
            }
            if let Ok(width) = Self::read_dword(hkey, "WindowWidth") {
                config.window.width = width as i32;
            }
            if let Ok(height) = Self::read_dword(hkey, "WindowHeight") {
                config.window.height = height as i32;
            }
            if let Ok(maximized) = Self::read_dword(hkey, "WindowMaximized") {
                config.window.maximized = maximized != 0;
            }

            // Load theme preference (T425)
            if let Ok(theme) = Self::read_dword(hkey, "ThemePreference") {
                config.theme.preference = match theme {
                    0 => Theme::System,
                    1 => Theme::Light,
                    2 => Theme::Dark,
                    _ => Theme::System,
                };
            }

            // Load monitoring settings (T425)
            if let Ok(refresh) = Self::read_dword(hkey, "RefreshRateMs") {
                config.monitoring.refresh_rate_ms = refresh;
            }
            if let Ok(history) = Self::read_dword(hkey, "HistoryLengthSec") {
                config.monitoring.history_length_sec = history;
            }
            if let Ok(graph_type) = Self::read_dword(hkey, "GraphType") {
                config.monitoring.graph_type = graph_type;
            }

            // Load startup options
            if let Ok(run_at_login) = Self::read_dword(hkey, "RunAtLogin") {
                config.startup.run_at_login = run_at_login != 0;
            }
            if let Ok(start_minimized) = Self::read_dword(hkey, "StartMinimized") {
                config.startup.start_minimized = start_minimized != 0;
            }
            if let Ok(perf_mode) = Self::read_dword(hkey, "PerformanceMode") {
                config.startup.performance_mode = perf_mode != 0;
            }

            let _ = RegCloseKey(hkey);
        }

        Ok(config)
    }

    /// Save to Windows Registry
    fn save_to_registry(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let mut hkey = Default::default();
            let subkey = windows::core::w!("Software\\TaskManager");
            
            // Create/open registry key
            let result = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                subkey,
                Some(0),
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                Some(std::ptr::null()),
                &mut hkey,
                Some(std::ptr::null_mut()),
            );
            
            if result.is_err() {
                return Err("Failed to create registry key".into());
            }

            // Save window position (T424)
            Self::write_dword(hkey, "WindowX", config.window.x as u32)?;
            Self::write_dword(hkey, "WindowY", config.window.y as u32)?;
            Self::write_dword(hkey, "WindowWidth", config.window.width as u32)?;
            Self::write_dword(hkey, "WindowHeight", config.window.height as u32)?;
            Self::write_dword(hkey, "WindowMaximized", if config.window.maximized { 1 } else { 0 })?;

            // Save theme preference (T425)
            let theme_val = match config.theme.preference {
                Theme::System => 0,
                Theme::Light => 1,
                Theme::Dark => 2,
            };
            Self::write_dword(hkey, "ThemePreference", theme_val)?;

            // Save monitoring settings (T425)
            Self::write_dword(hkey, "RefreshRateMs", config.monitoring.refresh_rate_ms)?;
            Self::write_dword(hkey, "HistoryLengthSec", config.monitoring.history_length_sec)?;
            Self::write_dword(hkey, "GraphType", config.monitoring.graph_type)?;

            // Save startup options
            Self::write_dword(hkey, "RunAtLogin", if config.startup.run_at_login { 1 } else { 0 })?;
            Self::write_dword(hkey, "StartMinimized", if config.startup.start_minimized { 1 } else { 0 })?;
            Self::write_dword(hkey, "PerformanceMode", if config.startup.performance_mode { 1 } else { 0 })?;

            let _ = RegCloseKey(hkey);
        }

        Ok(())
    }

    /// Read DWORD from registry
    unsafe fn read_dword(hkey: windows::Win32::System::Registry::HKEY, name: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let value_name = crate::util::strings::to_wide_string(name);
        let mut data: u32 = 0;
        let mut data_size = std::mem::size_of::<u32>() as u32;

        // SAFETY: Called within unsafe context, all parameters valid
        let result = unsafe {
            RegQueryValueExW(
                hkey,
                PCWSTR::from_raw(value_name.as_ptr()),
                Some(std::ptr::null_mut()),
                Some(std::ptr::null_mut()),
                Some(&mut data as *mut u32 as *mut u8),
                Some(&mut data_size),
            )
        };
        
        if result.is_err() {
            return Err("Failed to read registry value".into());
        }

        Ok(data)
    }

    /// Write DWORD to registry
    unsafe fn write_dword(hkey: windows::Win32::System::Registry::HKEY, name: &str, value: u32) -> Result<(), Box<dyn std::error::Error>> {
        let value_name = crate::util::strings::to_wide_string(name);
        
        // SAFETY: Creating slice from valid u32 reference
        let data_bytes = unsafe {
            std::slice::from_raw_parts(
                &value as *const u32 as *const u8,
                std::mem::size_of::<u32>()
            )
        };

        // SAFETY: Called within unsafe context, all parameters valid
        let result = unsafe {
            RegSetValueExW(
                hkey,
                PCWSTR::from_raw(value_name.as_ptr()),
                Some(0),
                REG_DWORD,
                Some(data_bytes),
            )
        };
        
        if result.is_err() {
            return Err("Failed to write registry value".into());
        }

        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = AppConfig::default();
        assert_eq!(config.window.width, 1200);
        assert_eq!(config.window.height, 800);
        assert_eq!(config.monitoring.refresh_rate_ms, 1000);
    }

    #[test]
    fn test_config_manager() {
        let manager = ConfigManager::new();
        let config = manager.get();
        assert_eq!(config.window.width, 1200);
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_window_bounds_update() {
        let manager = ConfigManager::new();
        
        let bounds = RECT {
            left: 200,
            top: 150,
            right: 1400,
            bottom: 950,
        };
        manager.set_window_bounds(bounds, false);
        
        assert!(manager.is_dirty());
        let config = manager.get();
        assert_eq!(config.window.x, 200);
        assert_eq!(config.window.y, 150);
        assert_eq!(config.window.width, 1200);
        assert_eq!(config.window.height, 800);
    }

    #[test]
    fn test_theme_update() {
        let manager = ConfigManager::new();
        manager.set_theme(Theme::Dark);
        
        assert!(manager.is_dirty());
        let config = manager.get();
        assert_eq!(config.theme.preference, Theme::Dark);
    }

    #[test]
    fn test_monitoring_update() {
        let manager = ConfigManager::new();
        manager.set_monitoring(2000, 300, 1);
        
        let config = manager.get();
        assert_eq!(config.monitoring.refresh_rate_ms, 2000);
        assert_eq!(config.monitoring.history_length_sec, 300);
        assert_eq!(config.monitoring.graph_type, 1);
    }

    #[test]
    fn test_column_width() {
        let manager = ConfigManager::new();
        manager.set_column_width("Name", 250);
        manager.set_column_width("CPU", 80);
        
        let config = manager.get();
        assert_eq!(config.columns.widths.get("Name"), Some(&250));
        assert_eq!(config.columns.widths.get("CPU"), Some(&80));
    }

    #[test]
    fn test_column_visibility() {
        let manager = ConfigManager::new();
        manager.set_column_visibility("PID", true);
        manager.set_column_visibility("Threads", false);
        
        let config = manager.get();
        assert_eq!(config.columns.visibility.get("PID"), Some(&true));
        assert_eq!(config.columns.visibility.get("Threads"), Some(&false));
    }

    #[test]
    fn test_startup_options() {
        let manager = ConfigManager::new();
        manager.set_startup_options(true, false, true);
        
        let config = manager.get();
        assert!(config.startup.run_at_login);
        assert!(!config.startup.start_minimized);
        assert!(config.startup.performance_mode);
    }

    #[test]
    fn test_json_export_import() {
        let manager = ConfigManager::new();
        manager.set_theme(Theme::Dark);
        manager.set_monitoring(500, 3600, 2);
        
        let temp_path = "test_config.json";
        manager.export_to_file(temp_path).unwrap();
        
        let manager2 = ConfigManager::new();
        manager2.import_from_file(temp_path).unwrap();
        
        let config = manager2.get();
        assert_eq!(config.theme.preference, Theme::Dark);
        assert_eq!(config.monitoring.refresh_rate_ms, 500);
        assert_eq!(config.monitoring.history_length_sec, 3600);
        
        let _ = std::fs::remove_file(temp_path);
    }
}
