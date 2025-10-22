//! Tab view control with Windows 11 Fluent Design
//!
//! Implements T435-T440:
//! - Tab system foundation
//! - Keyboard navigation (Ctrl+Tab, Ctrl+1-6)
//! - Tab overflow handling
//! - Registry persistence for active tab
//! - Fluent styling with accent color

use windows::Win32::Foundation::RECT;
use windows::Win32::System::Registry::*;
use windows::core::PCWSTR;
use std::sync::atomic::{AtomicU32, Ordering};

/// Tab identifiers for all panels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TabId {
    Processes = 0,
    Performance = 1,
    Startup = 2,
    Services = 3,
    Users = 4,
    Details = 5,
    Gpu = 6,
}

impl TabId {
    /// Convert from u32 index
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(TabId::Processes),
            1 => Some(TabId::Performance),
            2 => Some(TabId::Startup),
            3 => Some(TabId::Services),
            4 => Some(TabId::Users),
            5 => Some(TabId::Details),
            6 => Some(TabId::Gpu),
            _ => None,
        }
    }

    /// Get tab display name
    pub fn name(&self) -> &'static str {
        match self {
            TabId::Processes => "Processes",
            TabId::Performance => "Performance",
            TabId::Startup => "Startup",
            TabId::Services => "Services",
            TabId::Users => "Users",
            TabId::Details => "Details",
            TabId::Gpu => "GPU",
        }
    }

    /// Get tab keyboard shortcut (Ctrl+1 through Ctrl+6)
    pub fn hotkey(&self) -> u16 {
        match self {
            TabId::Processes => 0x31,   // '1'
            TabId::Performance => 0x32, // '2'
            TabId::Startup => 0x33,     // '3'
            TabId::Services => 0x34,    // '4'
            TabId::Users => 0x35,       // '5'
            TabId::Details => 0x36,     // '6'
            TabId::Gpu => 0x37,         // '7'
        }
    }

    /// Next tab (circular)
    pub fn next(&self) -> Self {
        TabId::from_u32((*self as u32 + 1) % 7).unwrap()
    }

    /// Previous tab (circular)
    pub fn prev(&self) -> Self {
        let current = *self as u32;
        TabId::from_u32(if current == 0 { 6 } else { current - 1 }).unwrap()
    }
}

/// Tab view metrics (UI specification)
pub struct TabMetrics {
    /// Tab bar height (standard mode)
    pub bar_height: i32,
    /// Tab bar height (compact mode)
    pub bar_height_compact: i32,
    /// Horizontal padding per tab
    pub tab_padding_h: i32,
    /// Vertical padding per tab
    pub tab_padding_v: i32,
    /// Icon size (scaled for DPI)
    pub icon_size: i32,
    /// Icon-text spacing
    pub icon_text_spacing: i32,
    /// Active tab indicator height
    pub active_indicator_height: i32,
    /// Hover overlay opacity (0-255)
    pub hover_opacity: u8,
}

impl Default for TabMetrics {
    fn default() -> Self {
        Self {
            bar_height: 48,
            bar_height_compact: 32,
            tab_padding_h: 16,
            tab_padding_v: 12,
            icon_size: 16,
            icon_text_spacing: 8,
            active_indicator_height: 3,
            hover_opacity: 20, // 8% of 255
        }
    }
}

/// Tab view state
pub struct TabView {
    /// Currently active tab
    active_tab: AtomicU32,
    /// Tab bar bounds
    #[allow(dead_code)]
    bounds: RECT,
    /// Window width for overflow detection
    window_width: i32,
    /// Compact mode enabled
    compact_mode: bool,
    /// Metrics for layout
    metrics: TabMetrics,
    /// Hovered tab (for hover effects)
    hovered_tab: Option<TabId>,
}

impl TabView {
    /// Create new tab view
    pub fn new() -> Self {
        // Load last active tab from registry
        let active_tab = Self::load_active_tab();

        Self {
            active_tab: AtomicU32::new(active_tab as u32),
            bounds: RECT::default(),
            window_width: 1024,
            compact_mode: false,
            metrics: TabMetrics::default(),
            hovered_tab: None,
        }
    }

    /// Get currently active tab
    pub fn active_tab(&self) -> TabId {
        TabId::from_u32(self.active_tab.load(Ordering::Relaxed)).unwrap()
    }

    /// Set active tab
    pub fn set_active_tab(&self, tab: TabId) {
        self.active_tab.store(tab as u32, Ordering::Relaxed);
        Self::save_active_tab(tab);
    }

    /// Handle keyboard navigation
    ///
    /// Returns true if key was handled
    pub fn handle_key(&self, vk_code: u16, ctrl_down: bool, shift_down: bool) -> bool {
        if !ctrl_down {
            return false;
        }

        match vk_code {
            0x09 => {
                // VK_TAB = 0x09: Ctrl+Tab next, Ctrl+Shift+Tab previous
                let current = self.active_tab();
                let next = if shift_down { current.prev() } else { current.next() };
                self.set_active_tab(next);
                true
            }
            0x31..=0x37 => {
                // Ctrl+1 through Ctrl+7: direct tab access
                if let Some(tab) = TabId::from_u32((vk_code - 0x31) as u32) {
                    self.set_active_tab(tab);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if tab should be in overflow menu
    ///
    /// UI spec: overflow menu when window width < 1024px
    pub fn is_overflow_tab(&self, tab: TabId) -> bool {
        if self.window_width >= 1024 {
            return false;
        }

        // Hide Details, Users, GPU in overflow menu
        matches!(tab, TabId::Details | TabId::Users | TabId::Gpu)
    }

    /// Get visible tabs (non-overflow)
    pub fn visible_tabs(&self) -> Vec<TabId> {
        let all_tabs = [
            TabId::Processes,
            TabId::Performance,
            TabId::Startup,
            TabId::Services,
            TabId::Users,
            TabId::Details,
            TabId::Gpu,
        ];

        all_tabs
            .iter()
            .filter(|&&tab| !self.is_overflow_tab(tab))
            .copied()
            .collect()
    }

    /// Get overflow tabs
    pub fn overflow_tabs(&self) -> Vec<TabId> {
        let all_tabs = [
            TabId::Processes,
            TabId::Performance,
            TabId::Startup,
            TabId::Services,
            TabId::Users,
            TabId::Details,
            TabId::Gpu,
        ];

        all_tabs
            .iter()
            .filter(|&&tab| self.is_overflow_tab(tab))
            .copied()
            .collect()
    }

    /// Update window width for overflow calculation
    pub fn set_window_width(&mut self, width: i32) {
        self.window_width = width;
    }

    /// Set compact mode
    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    /// Get tab bar height based on mode
    pub fn bar_height(&self) -> i32 {
        if self.compact_mode {
            self.metrics.bar_height_compact
        } else {
            self.metrics.bar_height
        }
    }

    /// Calculate tab bounds for given index
    pub fn tab_bounds(&self, index: usize, total_visible: usize) -> RECT {
        let tab_width = if total_visible > 0 {
            self.window_width / total_visible as i32
        } else {
            100
        };

        RECT {
            left: (index as i32) * tab_width,
            top: 0,
            right: (index as i32 + 1) * tab_width,
            bottom: self.bar_height(),
        }
    }

    /// Check if point is inside a tab
    pub fn hit_test(&self, x: i32, y: i32) -> Option<TabId> {
        if y < 0 || y >= self.bar_height() {
            return None;
        }

        let visible_tabs = self.visible_tabs();
        for (index, &tab) in visible_tabs.iter().enumerate() {
            let bounds = self.tab_bounds(index, visible_tabs.len());
            if x >= bounds.left && x < bounds.right {
                return Some(tab);
            }
        }

        None
    }

    /// Update hovered tab for hover effects
    pub fn set_hovered_tab(&mut self, tab: Option<TabId>) {
        self.hovered_tab = tab;
    }

    /// Get hovered tab
    pub fn hovered_tab(&self) -> Option<TabId> {
        self.hovered_tab
    }

    /// Load active tab from registry
    ///
    /// Registry: HKCU\Software\TaskManager\ActiveTab
    fn load_active_tab() -> TabId {
        // SAFETY: Registry API with proper error handling
        unsafe {
            let mut hkey = Default::default();
            let subkey = windows::core::w!("Software\\TaskManager");
            
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey,
                Some(0),
                KEY_READ,
                &mut hkey,
            ).is_ok() {
                let mut data: u32 = 0;
                let mut data_size = std::mem::size_of::<u32>() as u32;
                let value_name = windows::core::w!("ActiveTab");

                if RegQueryValueExW(
                    hkey,
                    value_name,
                    Some(std::ptr::null_mut()),
                    Some(std::ptr::null_mut()),
                    Some(&mut data as *mut u32 as *mut u8),
                    Some(&mut data_size),
                ).is_ok() {
                    let _ = RegCloseKey(hkey);
                    return TabId::from_u32(data).unwrap_or(TabId::Processes);
                }

                let _ = RegCloseKey(hkey);
            }
        }

        TabId::Processes // Default
    }

    /// Save active tab to registry
    fn save_active_tab(tab: TabId) {
        // SAFETY: Registry API with proper error handling
        unsafe {
            let mut hkey = Default::default();
            let subkey = windows::core::w!("Software\\TaskManager");
            
            // Create key if doesn't exist
            let _ = RegCreateKeyExW(
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

            let value_name = windows::core::w!("ActiveTab");
            let data = tab as u32;
            let data_bytes = std::slice::from_raw_parts(
                &data as *const u32 as *const u8,
                std::mem::size_of::<u32>()
            );

            let _ = RegSetValueExW(
                hkey,
                value_name,
                Some(0),
                REG_DWORD,
                Some(data_bytes),
            );

            let _ = RegCloseKey(hkey);
        }
    }

    /// Get active indicator bounds for rendering
    pub fn active_indicator_bounds(&self, tab_index: usize, total_visible: usize) -> RECT {
        let tab_bounds = self.tab_bounds(tab_index, total_visible);
        RECT {
            left: tab_bounds.left,
            top: tab_bounds.bottom - self.metrics.active_indicator_height,
            right: tab_bounds.right,
            bottom: tab_bounds.bottom,
        }
    }
}

impl Default for TabView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_navigation() {
        // Test circular navigation
        assert_eq!(TabId::Processes.next(), TabId::Performance);
        assert_eq!(TabId::Gpu.next(), TabId::Processes); // Wraps around
        assert_eq!(TabId::Processes.prev(), TabId::Gpu); // Wraps backward
    }

    #[test]
    fn test_tab_from_u32() {
        assert_eq!(TabId::from_u32(0), Some(TabId::Processes));
        assert_eq!(TabId::from_u32(6), Some(TabId::Gpu));
        assert_eq!(TabId::from_u32(99), None);
    }

    #[test]
    fn test_overflow_tabs() {
        let mut tab_view = TabView::new();
        
        // Wide window: no overflow
        tab_view.set_window_width(1200);
        assert_eq!(tab_view.visible_tabs().len(), 7);
        assert_eq!(tab_view.overflow_tabs().len(), 0);

        // Narrow window: overflow Details, Users, GPU
        tab_view.set_window_width(800);
        assert_eq!(tab_view.visible_tabs().len(), 4);
        assert_eq!(tab_view.overflow_tabs().len(), 3);
    }

    #[test]
    fn test_compact_mode() {
        let mut tab_view = TabView::new();
        
        // Standard mode
        assert_eq!(tab_view.bar_height(), 48);
        
        // Compact mode
        tab_view.set_compact_mode(true);
        assert_eq!(tab_view.bar_height(), 32);
    }

    #[test]
    fn test_keyboard_shortcuts() {
        let tab_view = TabView::new();
        
        // Ctrl+1 = Processes
        assert!(tab_view.handle_key(0x31, true, false));
        assert_eq!(tab_view.active_tab(), TabId::Processes);

        // Ctrl+3 = Startup
        assert!(tab_view.handle_key(0x33, true, false));
        assert_eq!(tab_view.active_tab(), TabId::Startup);

        // Without Ctrl, no action
        assert!(!tab_view.handle_key(0x31, false, false));
    }

    #[test]
    fn test_hit_testing() {
        let mut tab_view = TabView::new();
        tab_view.set_window_width(1024);

        // 7 visible tabs, each ~146px wide
        // Tab 0 (Processes): 0-146
        assert_eq!(tab_view.hit_test(50, 20), Some(TabId::Processes));
        
        // Tab 1 (Performance): 146-292
        assert_eq!(tab_view.hit_test(200, 20), Some(TabId::Performance));

        // Outside tab bar
        assert_eq!(tab_view.hit_test(50, 100), None);
    }

    #[test]
    fn test_tab_bounds() {
        let mut tab_view = TabView::new();
        tab_view.set_window_width(1000);

        let bounds = tab_view.tab_bounds(0, 5); // 5 visible tabs
        assert_eq!(bounds.left, 0);
        assert_eq!(bounds.right, 200); // 1000 / 5
        assert_eq!(bounds.bottom, 48); // Standard height
    }

    #[test]
    fn test_active_indicator_bounds() {
        let mut tab_view = TabView::new();
        tab_view.set_window_width(1000);

        let indicator = tab_view.active_indicator_bounds(0, 5);
        assert_eq!(indicator.left, 0);
        assert_eq!(indicator.right, 200);
        assert_eq!(indicator.top, 45); // 48 - 3
        assert_eq!(indicator.bottom, 48);
    }
}
