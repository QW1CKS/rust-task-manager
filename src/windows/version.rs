//! Windows version detection for feature degradation

use windows::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOEXW};

/// Cached Windows version information
static mut WINDOWS_VERSION: Option<WindowsVersion> = None;

#[derive(Debug, Clone, Copy)]
pub struct WindowsVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl WindowsVersion {
    /// Returns true if running on Windows 11 or later
    /// Windows 11 = NT 10.0, build >= 22000
    pub fn is_windows_11(&self) -> bool {
        self.major >= 10 && self.build >= 22000
    }

    /// Returns true if running on Windows 11 22H2 or later (build >= 22621)
    /// Required for DesktopAcrylicBackdrop
    pub fn is_windows_11_22h2(&self) -> bool {
        self.major >= 10 && self.build >= 22621
    }

    /// Returns true if running on Windows 10 or later
    pub fn is_windows_10_plus(&self) -> bool {
        self.major >= 10
    }
}

/// Get the current Windows version (cached after first call)
/// 
/// # Safety
/// 
/// Uses unsafe to access static mutable cache, but only writes once
/// and subsequent reads are safe (immutable after initialization).
pub fn get_windows_version() -> WindowsVersion {
    unsafe {
        if let Some(version) = WINDOWS_VERSION {
            return version;
        }

        // Use GetVersionExW (deprecated but still works reliably)
        let mut version_info = OSVERSIONINFOEXW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOEXW>() as u32,
            ..Default::default()
        };

        let success = GetVersionExW(&mut version_info as *mut _ as *mut _).is_ok();
        
        let version = if success {
            WindowsVersion {
                major: version_info.dwMajorVersion,
                minor: version_info.dwMinorVersion,
                build: version_info.dwBuildNumber,
            }
        } else {
            // Fallback: assume Windows 10 if we can't detect
            WindowsVersion {
                major: 10,
                minor: 0,
                build: 19041, // Windows 10 2004 baseline
            }
        };

        WINDOWS_VERSION = Some(version);
        version
    }
}

/// Returns true if Windows 11 features (Mica/Acrylic) are available
pub fn supports_fluent_design() -> bool {
    get_windows_version().is_windows_11()
}

/// Returns true if Windows 11 22H2 features (DesktopAcrylicBackdrop) are available
pub fn supports_desktop_acrylic() -> bool {
    get_windows_version().is_windows_11_22h2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_detection() {
        let version = get_windows_version();
        // Should always be at least Windows 10 (our minimum requirement)
        assert!(version.is_windows_10_plus());
        println!("Running on Windows {}.{} (build {})", version.major, version.minor, version.build);
    }

    #[test]
    fn test_windows_11_detection() {
        let version = get_windows_version();
        println!("Windows 11: {}", version.is_windows_11());
        println!("Windows 11 22H2: {}", version.is_windows_11_22h2());
    }
}
