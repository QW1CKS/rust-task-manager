//! Status bar panel displaying system metrics
//!
//! Implements T429-T434:
//! - Process count display
//! - CPU usage percentage
//! - Memory usage (used / total)
//! - Update timestamp
//! - Elevation status (Administrator badge)

use windows::Win32::Foundation::RECT;
use std::time::Instant;

/// Status bar metrics and layout
pub struct StatusBarMetrics {
    /// Status bar height
    pub height: i32,
    /// Padding (left/right/top/bottom)
    pub padding: i32,
    /// Spacing between sections
    pub section_spacing: i32,
    /// Icon size for elevation badge
    pub icon_size: i32,
}

impl Default for StatusBarMetrics {
    fn default() -> Self {
        Self {
            height: 32,
            padding: 12,
            section_spacing: 24,
            icon_size: 16,
        }
    }
}

/// Status bar section identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSection {
    ProcessCount,
    CpuUsage,
    MemoryUsage,
    UpdateStatus,
    Elevation,
}

/// Status bar state
pub struct StatusBar {
    /// Bounds of the status bar
    bounds: RECT,
    /// Metrics for layout
    metrics: StatusBarMetrics,
    /// Process count
    process_count: u32,
    /// CPU usage percentage (0-100)
    cpu_usage: f32,
    /// Memory used (bytes)
    memory_used: u64,
    /// Memory total (bytes)
    memory_total: u64,
    /// Last update time
    last_update: Instant,
    /// Is running as administrator
    is_elevated: bool,
}

impl StatusBar {
    /// Create new status bar
    pub fn new() -> Self {
        Self {
            bounds: RECT::default(),
            metrics: StatusBarMetrics::default(),
            process_count: 0,
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            last_update: Instant::now(),
            is_elevated: Self::check_elevation(),
        }
    }

    /// Set status bar bounds
    pub fn set_bounds(&mut self, bounds: RECT) {
        self.bounds = bounds;
    }

    /// Get status bar height
    pub fn height(&self) -> i32 {
        self.metrics.height
    }

    /// Update process count (T430)
    pub fn set_process_count(&mut self, count: u32) {
        self.process_count = count;
    }

    /// Update CPU usage (T431)
    ///
    /// # Arguments
    /// * `usage` - CPU usage percentage (0.0 - 100.0)
    pub fn set_cpu_usage(&mut self, usage: f32) {
        self.cpu_usage = usage.clamp(0.0, 100.0);
    }

    /// Update memory usage (T432)
    ///
    /// # Arguments
    /// * `used` - Memory used in bytes
    /// * `total` - Total memory in bytes
    pub fn set_memory_usage(&mut self, used: u64, total: u64) {
        self.memory_used = used;
        self.memory_total = total;
    }

    /// Mark data as updated (T433)
    pub fn mark_updated(&mut self) {
        self.last_update = Instant::now();
    }

    /// Get process count text
    pub fn process_count_text(&self) -> String {
        format!("Processes: {}", self.process_count)
    }

    /// Get CPU usage text
    pub fn cpu_usage_text(&self) -> String {
        format!("CPU: {:.0}%", self.cpu_usage)
    }

    /// Get memory usage text
    pub fn memory_usage_text(&self) -> String {
        let used_gb = self.memory_used as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("Memory: {:.1} / {:.1} GB", used_gb, total_gb)
    }

    /// Get update status text (T433)
    pub fn update_status_text(&self) -> String {
        let elapsed = self.last_update.elapsed().as_secs();
        if elapsed == 0 {
            "Updated just now".to_string()
        } else if elapsed == 1 {
            "Updated 1s ago".to_string()
        } else if elapsed < 60 {
            format!("Updated {}s ago", elapsed)
        } else {
            let minutes = elapsed / 60;
            if minutes == 1 {
                "Updated 1m ago".to_string()
            } else {
                format!("Updated {}m ago", minutes)
            }
        }
    }

    /// Get elevation status text (T434)
    pub fn elevation_text(&self) -> Option<&'static str> {
        if self.is_elevated {
            Some("Administrator")
        } else {
            None
        }
    }

    /// Check if running as administrator (T434)
    ///
    /// Uses Windows Token APIs to check elevation
    fn check_elevation() -> bool {
        use windows::Win32::Foundation::{HANDLE, CloseHandle};
        use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token: HANDLE = Default::default();
            
            // Open process token
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut token,
            ).is_err() {
                return false;
            }

            // Query elevation status
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length: u32 = 0;

            let result = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut TOKEN_ELEVATION as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            ).is_ok();

            let _ = CloseHandle(token);

            result && elevation.TokenIsElevated != 0
        }
    }

    /// Get section bounds for rendering
    ///
    /// Returns left, top, right, bottom for each section
    pub fn section_bounds(&self, section: StatusSection) -> RECT {
        let left_start = self.bounds.left + self.metrics.padding;
        let top = self.bounds.top + (self.metrics.height - 20) / 2; // Center text vertically
        let bottom = top + 20;

        match section {
            StatusSection::ProcessCount => {
                // "Processes: 157" - leftmost
                RECT {
                    left: left_start,
                    top,
                    right: left_start + 150,
                    bottom,
                }
            }
            StatusSection::CpuUsage => {
                // "CPU: 23%" - after processes
                RECT {
                    left: left_start + 150 + self.metrics.section_spacing,
                    top,
                    right: left_start + 150 + self.metrics.section_spacing + 100,
                    bottom,
                }
            }
            StatusSection::MemoryUsage => {
                // "Memory: 8.2 / 16 GB" - after CPU
                RECT {
                    left: left_start + 250 + self.metrics.section_spacing * 2,
                    top,
                    right: left_start + 250 + self.metrics.section_spacing * 2 + 200,
                    bottom,
                }
            }
            StatusSection::UpdateStatus => {
                // "Updated 1s ago" - center-right
                RECT {
                    left: left_start + 450 + self.metrics.section_spacing * 3,
                    top,
                    right: left_start + 450 + self.metrics.section_spacing * 3 + 150,
                    bottom,
                }
            }
            StatusSection::Elevation => {
                // "Administrator" with shield icon - rightmost
                let right_edge = self.bounds.right - self.metrics.padding;
                RECT {
                    left: right_edge - 150,
                    top,
                    right: right_edge,
                    bottom,
                }
            }
        }
    }

    /// Get all visible sections
    pub fn visible_sections(&self) -> Vec<StatusSection> {
        let mut sections = vec![
            StatusSection::ProcessCount,
            StatusSection::CpuUsage,
            StatusSection::MemoryUsage,
        ];

        // Update status optional
        sections.push(StatusSection::UpdateStatus);

        // Elevation badge only if elevated
        if self.is_elevated {
            sections.push(StatusSection::Elevation);
        }

        sections
    }

    /// Check if mouse is over a section
    pub fn hit_test(&self, x: i32, y: i32) -> Option<StatusSection> {
        if y < self.bounds.top || y >= self.bounds.bottom {
            return None;
        }

        for &section in &self.visible_sections() {
            let bounds = self.section_bounds(section);
            if x >= bounds.left && x < bounds.right {
                return Some(section);
            }
        }

        None
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_creation() {
        let status_bar = StatusBar::new();
        assert_eq!(status_bar.height(), 32);
        assert_eq!(status_bar.process_count, 0);
        assert_eq!(status_bar.cpu_usage, 0.0);
    }

    #[test]
    fn test_process_count_text() {
        let mut status_bar = StatusBar::new();
        status_bar.set_process_count(157);
        assert_eq!(status_bar.process_count_text(), "Processes: 157");
    }

    #[test]
    fn test_cpu_usage_text() {
        let mut status_bar = StatusBar::new();
        status_bar.set_cpu_usage(23.456);
        assert_eq!(status_bar.cpu_usage_text(), "CPU: 23%");
    }

    #[test]
    fn test_cpu_usage_clamping() {
        let mut status_bar = StatusBar::new();
        status_bar.set_cpu_usage(150.0);
        assert_eq!(status_bar.cpu_usage, 100.0);
        
        status_bar.set_cpu_usage(-10.0);
        assert_eq!(status_bar.cpu_usage, 0.0);
    }

    #[test]
    fn test_memory_usage_text() {
        let mut status_bar = StatusBar::new();
        let gb = 1024u64 * 1024 * 1024;
        status_bar.set_memory_usage(8 * gb + 200 * 1024 * 1024, 16 * gb);
        let text = status_bar.memory_usage_text();
        assert!(text.contains("8."));
        assert!(text.contains("16.0"));
        assert!(text.contains("GB"));
    }

    #[test]
    fn test_update_status_text() {
        let status_bar = StatusBar::new();
        let text = status_bar.update_status_text();
        assert!(text.contains("Updated"));
    }

    #[test]
    fn test_elevation_status() {
        let status_bar = StatusBar::new();
        // Result depends on whether tests are run elevated
        if status_bar.is_elevated {
            assert_eq!(status_bar.elevation_text(), Some("Administrator"));
        } else {
            assert_eq!(status_bar.elevation_text(), None);
        }
    }

    #[test]
    fn test_visible_sections() {
        let status_bar = StatusBar::new();
        let sections = status_bar.visible_sections();
        
        // Always visible: processes, cpu, memory, update
        assert!(sections.contains(&StatusSection::ProcessCount));
        assert!(sections.contains(&StatusSection::CpuUsage));
        assert!(sections.contains(&StatusSection::MemoryUsage));
        assert!(sections.contains(&StatusSection::UpdateStatus));
    }

    #[test]
    fn test_section_bounds() {
        let mut status_bar = StatusBar::new();
        status_bar.set_bounds(RECT {
            left: 0,
            top: 568,
            right: 1000,
            bottom: 600,
        });

        let bounds = status_bar.section_bounds(StatusSection::ProcessCount);
        assert_eq!(bounds.left, 12); // padding
        assert_eq!(bounds.top, 574); // (568 + (32-20)/2)
        assert_eq!(bounds.right, 162); // 12 + 150
        assert_eq!(bounds.bottom, 594); // 574 + 20
    }

    #[test]
    fn test_hit_testing() {
        let mut status_bar = StatusBar::new();
        status_bar.set_bounds(RECT {
            left: 0,
            top: 568,
            right: 1000,
            bottom: 600,
        });

        // Click in process count area
        assert_eq!(
            status_bar.hit_test(50, 580),
            Some(StatusSection::ProcessCount)
        );

        // Click outside status bar
        assert_eq!(status_bar.hit_test(50, 500), None);
    }
}
