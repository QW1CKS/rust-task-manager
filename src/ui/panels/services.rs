//! Services tab for Windows service management (T446-T450)
//!
//! Displays Windows services with:
//! - Service name, status, startup type, description
//! - Start/Stop/Restart buttons
//! - Dependency tree view
//! - Filter for running/stopped services

use windows::Win32::Foundation::HWND;

// Note: Full Service API may not be available in current windows crate version
// Define minimal types and constants we need
#[allow(non_camel_case_types)]
#[allow(dead_code)]
type SC_HANDLE = isize;

#[allow(dead_code)]
const SC_MANAGER_ENUMERATE_SERVICE: u32 = 0x0004;

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
    Unknown,
}

impl ServiceStatus {
    /// Convert from Windows SERVICE_STATUS dwCurrentState
    pub fn from_win32(state: u32) -> Self {
        match state {
            1 => ServiceStatus::Stopped,         // SERVICE_STOPPED
            2 => ServiceStatus::StartPending,    // SERVICE_START_PENDING
            3 => ServiceStatus::StopPending,     // SERVICE_STOP_PENDING
            4 => ServiceStatus::Running,         // SERVICE_RUNNING
            5 => ServiceStatus::ContinuePending, // SERVICE_CONTINUE_PENDING
            6 => ServiceStatus::PausePending,    // SERVICE_PAUSE_PENDING
            7 => ServiceStatus::Paused,          // SERVICE_PAUSED
            _ => ServiceStatus::Unknown,
        }
    }

    /// Get color for status display
    pub fn color(&self) -> u32 {
        match self {
            ServiceStatus::Running => 0xFF00AA00,          // Green
            ServiceStatus::Stopped => 0xFF808080,          // Gray
            ServiceStatus::StartPending => 0xFF0078D4,     // Blue
            ServiceStatus::StopPending => 0xFFFFAA00,      // Orange
            ServiceStatus::Paused => 0xFFFFAA00,           // Orange
            _ => 0xFF808080,                               // Gray
        }
    }

    /// Get display text
    pub fn label(&self) -> &'static str {
        match self {
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::StartPending => "Starting...",
            ServiceStatus::StopPending => "Stopping...",
            ServiceStatus::Running => "Running",
            ServiceStatus::ContinuePending => "Resuming...",
            ServiceStatus::PausePending => "Pausing...",
            ServiceStatus::Paused => "Paused",
            ServiceStatus::Unknown => "Unknown",
        }
    }
}

/// Service startup type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupType {
    Boot,
    System,
    Automatic,
    Manual,
    Disabled,
}

impl StartupType {
    /// Convert from Windows SERVICE_CONFIG dwStartType
    pub fn from_win32(start_type: u32) -> Self {
        match start_type {
            0 => StartupType::Boot,      // SERVICE_BOOT_START
            1 => StartupType::System,    // SERVICE_SYSTEM_START
            2 => StartupType::Automatic, // SERVICE_AUTO_START
            3 => StartupType::Manual,    // SERVICE_DEMAND_START
            4 => StartupType::Disabled,  // SERVICE_DISABLED
            _ => StartupType::Manual,
        }
    }

    /// Get display text
    pub fn label(&self) -> &'static str {
        match self {
            StartupType::Boot => "Boot",
            StartupType::System => "System",
            StartupType::Automatic => "Automatic",
            StartupType::Manual => "Manual",
            StartupType::Disabled => "Disabled",
        }
    }
}

/// Windows service information (T447)
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name (internal name)
    pub name: String,
    /// Display name (user-friendly)
    pub display_name: String,
    /// Current status
    pub status: ServiceStatus,
    /// Startup type
    pub startup_type: StartupType,
    /// Description
    pub description: String,
    /// Dependencies (service names)
    pub dependencies: Vec<String>,
    /// Process ID (if running)
    pub pid: Option<u32>,
}

impl ServiceInfo {
    /// Create new service info
    pub fn new(name: String, display_name: String) -> Self {
        Self {
            name,
            display_name,
            status: ServiceStatus::Unknown,
            startup_type: StartupType::Manual,
            description: String::new(),
            dependencies: Vec::new(),
            pid: None,
        }
    }
}

/// Service manager for querying and controlling services
pub struct ServiceManager {
    services: Vec<ServiceInfo>,
    filter: ServiceFilter,
}

/// Service filter type (T450)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceFilter {
    All,
    Running,
    Stopped,
}

impl ServiceManager {
    /// Create new service manager
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            filter: ServiceFilter::All,
        }
    }

    /// Enumerate all services
    ///
    /// # Safety
    ///
    /// Calls Windows Service Control Manager APIs which are unsafe.
    pub fn enumerate_services(&mut self) -> windows::core::Result<()> {
        self.services.clear();

        // TODO: Full implementation would use Windows Service Control Manager APIs
        // For now, add some dummy services for UI testing
        
        self.services.push(ServiceInfo {
            name: "wuauserv".to_string(),
            display_name: "Windows Update".to_string(),
            status: ServiceStatus::Running,
            startup_type: StartupType::Automatic,
            description: "Enables the detection, download, and installation of updates".to_string(),
            dependencies: vec!["rpcss".to_string()],
            pid: Some(1234),
        });

        self.services.push(ServiceInfo {
            name: "spooler".to_string(),
            display_name: "Print Spooler".to_string(),
            status: ServiceStatus::Running,
            startup_type: StartupType::Automatic,
            description: "Loads files to memory for later printing".to_string(),
            dependencies: vec!["RPCSS".to_string(), "http".to_string()],
            pid: Some(5678),
        });

        Ok(())
    }

    /// Get filtered service list
    pub fn get_services(&self) -> Vec<&ServiceInfo> {
        self.services.iter().filter(|service| {
            match self.filter {
                ServiceFilter::All => true,
                ServiceFilter::Running => service.status == ServiceStatus::Running,
                ServiceFilter::Stopped => service.status == ServiceStatus::Stopped,
            }
        }).collect()
    }

    /// Set filter (T450)
    pub fn set_filter(&mut self, filter: ServiceFilter) {
        self.filter = filter;
    }

    /// Start a service (T448)
    pub fn start_service(&mut self, name: &str) -> windows::core::Result<()> {
        // TODO: Implement using OpenServiceW + StartServiceW
        let _ = name;
        Ok(())
    }

    /// Stop a service (T448)
    pub fn stop_service(&mut self, name: &str) -> windows::core::Result<()> {
        // TODO: Implement using OpenServiceW + ControlService(SERVICE_CONTROL_STOP)
        let _ = name;
        Ok(())
    }

    /// Restart a service (T448)
    pub fn restart_service(&mut self, name: &str) -> windows::core::Result<()> {
        // Stop then start
        self.stop_service(name)?;
        // Wait for service to stop
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.start_service(name)?;
        Ok(())
    }

    /// Get service dependencies (T449)
    pub fn get_dependencies(&self, name: &str) -> Vec<String> {
        self.services
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.dependencies.clone())
            .unwrap_or_default()
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Services panel UI state
pub struct ServicesPanel {
    #[allow(dead_code)]
    hwnd: HWND,
    manager: ServiceManager,
    selected_service: Option<String>,
}

impl ServicesPanel {
    /// Create new services panel
    pub fn new(hwnd: HWND) -> Self {
        let mut manager = ServiceManager::new();
        let _ = manager.enumerate_services(); // Ignore errors for now

        Self {
            hwnd,
            manager,
            selected_service: None,
        }
    }

    /// Get service manager
    pub fn manager(&self) -> &ServiceManager {
        &self.manager
    }

    /// Get mutable service manager
    pub fn manager_mut(&mut self) -> &mut ServiceManager {
        &mut self.manager
    }

    /// Select a service
    pub fn select_service(&mut self, name: String) {
        self.selected_service = Some(name);
    }

    /// Get selected service
    pub fn selected_service(&self) -> Option<&str> {
        self.selected_service.as_deref()
    }

    /// Refresh services list
    pub fn refresh(&mut self) -> windows::core::Result<()> {
        self.manager.enumerate_services()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_conversion() {
        assert_eq!(ServiceStatus::from_win32(1), ServiceStatus::Stopped);
        assert_eq!(ServiceStatus::from_win32(4), ServiceStatus::Running);
        assert_eq!(ServiceStatus::from_win32(7), ServiceStatus::Paused);
    }

    #[test]
    fn test_startup_type_conversion() {
        assert_eq!(StartupType::from_win32(2), StartupType::Automatic);
        assert_eq!(StartupType::from_win32(3), StartupType::Manual);
        assert_eq!(StartupType::from_win32(4), StartupType::Disabled);
    }

    #[test]
    fn test_service_filter() {
        let mut manager = ServiceManager::new();
        manager.services.push(ServiceInfo {
            name: "test1".to_string(),
            display_name: "Test 1".to_string(),
            status: ServiceStatus::Running,
            startup_type: StartupType::Automatic,
            description: String::new(),
            dependencies: Vec::new(),
            pid: Some(100),
        });
        manager.services.push(ServiceInfo {
            name: "test2".to_string(),
            display_name: "Test 2".to_string(),
            status: ServiceStatus::Stopped,
            startup_type: StartupType::Manual,
            description: String::new(),
            dependencies: Vec::new(),
            pid: None,
        });

        manager.set_filter(ServiceFilter::Running);
        assert_eq!(manager.get_services().len(), 1);

        manager.set_filter(ServiceFilter::Stopped);
        assert_eq!(manager.get_services().len(), 1);

        manager.set_filter(ServiceFilter::All);
        assert_eq!(manager.get_services().len(), 2);
    }
}
