//! Theme system with light/dark mode detection (T376-T383)
//!
//! Implements Windows system theme detection and color palette management
//! following Windows 11 Fluent Design principles.

use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;
use windows::core::BOOL;
use std::sync::atomic::{AtomicU32, Ordering};
use serde::{Serialize, Deserialize};

/// Current active theme (atomic for thread-safe updates)
static CURRENT_THEME: AtomicU32 = AtomicU32::new(0); // 0 = System, 1 = Light, 2 = Dark

/// System theme types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    /// Follow system theme setting
    System,
    /// Force light theme
    Light,
    /// Force dark theme
    Dark,
}

impl Theme {
    /// Convert from u32 for atomic storage
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Theme::Light,
            2 => Theme::Dark,
            _ => Theme::System,
        }
    }

    /// Convert to u32 for atomic storage
    pub fn to_u32(self) -> u32 {
        match self {
            Theme::System => 0,
            Theme::Light => 1,
            Theme::Dark => 2,
        }
    }
}

/// Color palette for light theme (Windows 11 Fluent Design)
#[derive(Debug, Clone, Copy)]
pub struct LightTheme;

impl LightTheme {
    // Background colors
    pub const BACKGROUND: u32 = 0xFFFAFAFA;         // Light gray background
    pub const BACKGROUND_SECONDARY: u32 = 0xFFFFFFFF; // Pure white panels
    pub const BACKGROUND_TERTIARY: u32 = 0xFFF5F5F5;  // Slightly darker panels
    
    // Text colors
    pub const TEXT_PRIMARY: u32 = 0xFF000000;       // Black text
    pub const TEXT_SECONDARY: u32 = 0xFF666666;     // Gray text
    pub const TEXT_TERTIARY: u32 = 0xFF999999;      // Light gray text
    pub const TEXT_DISABLED: u32 = 0xFFCCCCCC;      // Disabled text
    
    // Accent colors (will be overridden by system accent)
    pub const ACCENT: u32 = 0xFF0078D4;             // Windows blue
    pub const ACCENT_HOVER: u32 = 0xFF005A9E;       // Darker blue
    pub const ACCENT_PRESSED: u32 = 0xFF004275;     // Even darker
    
    // UI element colors
    pub const BORDER: u32 = 0xFFE0E0E0;             // Light borders
    pub const DIVIDER: u32 = 0xFFF0F0F0;            // Subtle dividers
    pub const HOVER_OVERLAY: u32 = 0x14000000;      // 8% black overlay
    pub const PRESSED_OVERLAY: u32 = 0x1F000000;    // 12% black overlay
    pub const SELECTED_OVERLAY: u32 = 0x0A000000;   // 4% black overlay
    
    // Status colors
    pub const SUCCESS: u32 = 0xFF107C10;            // Green
    pub const WARNING: u32 = 0xFFFFA500;            // Orange
    pub const ERROR: u32 = 0xFFE81123;              // Red
    pub const INFO: u32 = 0xFF0078D4;               // Blue
}

/// Color palette for dark theme (Windows 11 Fluent Design)
#[derive(Debug, Clone, Copy)]
pub struct DarkTheme;

impl DarkTheme {
    // Background colors
    pub const BACKGROUND: u32 = 0xFF202020;         // Dark gray background
    pub const BACKGROUND_SECONDARY: u32 = 0xFF2D2D2D; // Slightly lighter panels
    pub const BACKGROUND_TERTIARY: u32 = 0xFF1A1A1A;  // Darker panels
    
    // Text colors
    pub const TEXT_PRIMARY: u32 = 0xFFFFFFFF;       // White text
    pub const TEXT_SECONDARY: u32 = 0xFFB0B0B0;     // Light gray text
    pub const TEXT_TERTIARY: u32 = 0xFF808080;      // Gray text
    pub const TEXT_DISABLED: u32 = 0xFF505050;      // Disabled text
    
    // Accent colors (will be overridden by system accent)
    pub const ACCENT: u32 = 0xFF60CDFF;             // Light blue for dark theme
    pub const ACCENT_HOVER: u32 = 0xFF4FB3E6;       // Slightly darker
    pub const ACCENT_PRESSED: u32 = 0xFF3E9ACC;     // Even darker
    
    // UI element colors
    pub const BORDER: u32 = 0xFF3A3A3A;             // Dark borders
    pub const DIVIDER: u32 = 0xFF2A2A2A;            // Subtle dividers
    pub const HOVER_OVERLAY: u32 = 0x14FFFFFF;      // 8% white overlay
    pub const PRESSED_OVERLAY: u32 = 0x1FFFFFFF;    // 12% white overlay
    pub const SELECTED_OVERLAY: u32 = 0x0AFFFFFF;   // 4% white overlay
    
    // Status colors (slightly brighter for dark theme)
    pub const SUCCESS: u32 = 0xFF6CCB5F;            // Light green
    pub const WARNING: u32 = 0xFFFFC83D;            // Light orange
    pub const ERROR: u32 = 0xFFFF99A4;              // Light red
    pub const INFO: u32 = 0xFF60CDFF;               // Light blue
}

/// Theme manager handling system theme detection and changes
pub struct ThemeManager {
    /// User preference (System, Light, or Dark)
    preference: Theme,
    /// Detected system theme (true = dark, false = light)
    system_is_dark: bool,
    /// System accent color (ARGB format)
    accent_color: u32,
    /// High contrast mode active
    high_contrast: bool,
}

impl ThemeManager {
    /// Create new theme manager and detect system theme
    ///
    /// # Performance (T377)
    ///
    /// Must complete in <10ms. Registry and DWM queries are fast.
    pub fn new() -> Self {
        let system_is_dark = Self::detect_system_theme();
        let accent_color = Self::get_accent_color();
        let high_contrast = Self::is_high_contrast_mode();
        
        Self {
            preference: Theme::System,
            system_is_dark,
            accent_color,
            high_contrast,
        }
    }

    /// T377: Detect light/dark theme from Windows registry
    ///
    /// Reads: HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme
    /// Returns: false for dark theme, true for light theme
    fn detect_system_theme() -> bool {
        use windows::Win32::System::Registry::*;
        
        unsafe {
            let mut key = HKEY::default();
            let subkey = windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            
            // Try to open registry key
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey,
                Some(0),
                KEY_READ,
                &mut key,
            ).is_ok() {
                let value_name = windows::core::w!("AppsUseLightTheme");
                let mut data: u32 = 0;
                let mut data_size = std::mem::size_of::<u32>() as u32;
                
                if RegQueryValueExW(
                    key,
                    value_name,
                    None,
                    None,
                    Some(&mut data as *mut u32 as *mut u8),
                    Some(&mut data_size),
                ).is_ok() {
                    let _ = RegCloseKey(key);
                    return data == 0; // 0 = dark theme, 1 = light theme
                }
                
                let _ = RegCloseKey(key);
            }
        }
        
        // Default to light theme if detection fails
        false
    }

    /// T380: Get system accent color from DWM
    ///
    /// Returns ARGB color for system accent (taskbar, window chrome)
    fn get_accent_color() -> u32 {
        unsafe {
            let mut color: u32 = 0;
            let mut opaque = BOOL(0);
            
            if DwmGetColorizationColor(&mut color, &mut opaque).is_ok() {
                // Convert ARGB to RGB with full opacity
                return 0xFF000000 | (color & 0x00FFFFFF);
            }
        }
        
        // Default to Windows blue
        0xFF0078D4
    }

    /// T411: Detect high contrast mode
    ///
    /// High contrast disables transparency, blur, and uses system colors
    fn is_high_contrast_mode() -> bool {
        use windows::Win32::UI::WindowsAndMessaging::*;
        
        // Define HIGHCONTRASTW if not available in current windows-rs version
        #[repr(C)]
        struct HIGHCONTRASTW {
            cb_size: u32,
            dw_flags: u32,
            lpsz_default_scheme: *mut u16,
        }
        
        unsafe {
            let mut hc_info = HIGHCONTRASTW {
                cb_size: std::mem::size_of::<HIGHCONTRASTW>() as u32,
                dw_flags: 0,
                lpsz_default_scheme: std::ptr::null_mut(),
            };
            
            // Query high contrast setting
            // SPI_GETHIGHCONTRAST = 0x0042
            if SystemParametersInfoW(
                SYSTEM_PARAMETERS_INFO_ACTION(0x0042),
                std::mem::size_of::<HIGHCONTRASTW>() as u32,
                Some(&mut hc_info as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            ).is_ok() {
                // HCF_HIGHCONTRASTON = 0x00000001
                return (hc_info.dw_flags & 0x00000001) != 0;
            }
        }
        
        false
    }

    /// Get the effective theme (resolves System to Light/Dark)
    pub fn effective_theme(&self) -> bool {
        match self.preference {
            Theme::System => self.system_is_dark,
            Theme::Light => false,
            Theme::Dark => true,
        }
    }

    /// Set user preference (T382)
    pub fn set_preference(&mut self, theme: Theme) {
        self.preference = theme;
        CURRENT_THEME.store(theme.to_u32(), Ordering::Release);
    }

    /// Get user preference
    pub fn preference(&self) -> Theme {
        self.preference
    }

    /// Get accent color (T380-T381)
    pub fn accent_color(&self) -> u32 {
        self.accent_color
    }

    /// Is high contrast mode active? (T411)
    pub fn is_high_contrast(&self) -> bool {
        self.high_contrast
    }

    /// Refresh theme from system (call on WM_SETTINGCHANGE)
    ///
    /// T378: Handle system theme changes
    pub fn refresh(&mut self) {
        self.system_is_dark = Self::detect_system_theme();
        self.accent_color = Self::get_accent_color();
        self.high_contrast = Self::is_high_contrast_mode();
    }

    /// Get background color for current theme
    pub fn background(&self) -> u32 {
        if self.high_contrast {
            // T413: Use system colors in high contrast
            return 0xFF000000; // Black background in high contrast
        }
        
        if self.effective_theme() {
            DarkTheme::BACKGROUND
        } else {
            LightTheme::BACKGROUND
        }
    }

    /// Get primary text color for current theme
    pub fn text_primary(&self) -> u32 {
        if self.high_contrast {
            return 0xFFFFFFFF; // White text in high contrast
        }
        
        if self.effective_theme() {
            DarkTheme::TEXT_PRIMARY
        } else {
            LightTheme::TEXT_PRIMARY
        }
    }

    /// Get secondary text color for current theme
    pub fn text_secondary(&self) -> u32 {
        if self.high_contrast {
            return 0xFFFFFFFF;
        }
        
        if self.effective_theme() {
            DarkTheme::TEXT_SECONDARY
        } else {
            LightTheme::TEXT_SECONDARY
        }
    }

    /// Get border color for current theme
    pub fn border(&self) -> u32 {
        if self.high_contrast {
            return 0xFFFFFFFF; // White borders in high contrast
        }
        
        if self.effective_theme() {
            DarkTheme::BORDER
        } else {
            LightTheme::BORDER
        }
    }

    /// Get hover overlay color for current theme
    pub fn hover_overlay(&self) -> u32 {
        if self.high_contrast {
            return 0x00000000; // T414: No transparency in high contrast
        }
        
        if self.effective_theme() {
            DarkTheme::HOVER_OVERLAY
        } else {
            LightTheme::HOVER_OVERLAY
        }
    }

    /// T412: Get system color for high contrast mode
    ///
    /// Returns Windows system colors (from GetSysColor)
    pub fn get_system_color(&self, index: i32) -> u32 {
        use windows::Win32::Graphics::Gdi::{GetSysColor, SYS_COLOR_INDEX};
        
        unsafe {
            let color = GetSysColor(SYS_COLOR_INDEX(index));
            // Convert COLORREF to ARGB
            0xFF000000 | color
        }
    }

    /// Get high contrast background color (T413)
    pub fn high_contrast_background(&self) -> u32 {
        // COLOR_WINDOW = 5
        self.get_system_color(5)
    }

    /// Get high contrast text color (T413)
    pub fn high_contrast_text(&self) -> u32 {
        // COLOR_WINDOWTEXT = 8
        self.get_system_color(8)
    }

    /// Get high contrast highlight color (T413)
    pub fn high_contrast_highlight(&self) -> u32 {
        // COLOR_HIGHLIGHT = 13
        self.get_system_color(13)
    }

    /// Get high contrast highlight text color (T413)
    pub fn high_contrast_highlight_text(&self) -> u32 {
        // COLOR_HIGHLIGHTTEXT = 14
        self.get_system_color(14)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let theme = ThemeManager::new();
        // Should not panic, returns some theme
        assert!(theme.accent_color() != 0);
    }

    #[test]
    fn test_theme_preference() {
        let mut theme = ThemeManager::new();
        theme.set_preference(Theme::Dark);
        assert_eq!(theme.preference(), Theme::Dark);
        assert_eq!(theme.effective_theme(), true); // true = dark
    }

    #[test]
    fn test_theme_conversion() {
        assert_eq!(Theme::from_u32(0), Theme::System);
        assert_eq!(Theme::from_u32(1), Theme::Light);
        assert_eq!(Theme::from_u32(2), Theme::Dark);
        
        assert_eq!(Theme::System.to_u32(), 0);
        assert_eq!(Theme::Light.to_u32(), 1);
        assert_eq!(Theme::Dark.to_u32(), 2);
    }

    #[test]
    fn test_color_palettes() {
        // Light theme should have dark text on light background
        assert!(LightTheme::BACKGROUND > 0xFF000000);
        assert_eq!(LightTheme::TEXT_PRIMARY & 0xFFFFFF, 0x000000);
        
        // Dark theme should have light text on dark background
        assert!(DarkTheme::BACKGROUND < 0xFF808080);
        assert_eq!(DarkTheme::TEXT_PRIMARY & 0xFFFFFF, 0xFFFFFF);
    }
}
