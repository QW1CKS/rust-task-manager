//! Filter Box UI Control (T200-T203)
//!
//! Text input for real-time filtering with:
//! - Debounced input (50ms)
//! - Clear button
//! - Filter presets dropdown

use std::time::{Duration, Instant};

/// T200: Filter box state
pub struct FilterBox {
    /// Current filter text
    text: String,
    /// Last change time (for debouncing)
    last_change: Option<Instant>,
    /// Debounce duration
    debounce_duration: Duration,
    /// Is focused
    focused: bool,
}

impl FilterBox {
    /// Create new filter box
    pub fn new() -> Self {
        Self {
            text: String::new(),
            last_change: None,
            debounce_duration: Duration::from_millis(50), // T201: 50ms debounce
            focused: false,
        }
    }

    /// T200: Set filter text
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.last_change = Some(Instant::now());
    }

    /// Get current filter text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// T202: Clear filter text
    pub fn clear(&mut self) {
        self.text.clear();
        self.last_change = Some(Instant::now());
    }

    /// Check if filter should be applied (debounce elapsed)
    pub fn should_apply_filter(&self) -> bool {
        if let Some(last_change) = self.last_change {
            last_change.elapsed() >= self.debounce_duration
        } else {
            false
        }
    }

    /// Mark filter as applied
    pub fn mark_applied(&mut self) {
        self.last_change = None;
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Handle character input
    pub fn on_char(&mut self, ch: char) {
        if !ch.is_control() {
            self.text.push(ch);
            self.last_change = Some(Instant::now());
        }
    }

    /// Handle backspace
    pub fn on_backspace(&mut self) {
        if !self.text.is_empty() {
            self.text.pop();
            self.last_change = Some(Instant::now());
        }
    }

    /// Handle delete
    pub fn on_delete(&mut self) {
        self.clear();
    }
}

impl Default for FilterBox {
    fn default() -> Self {
        Self::new()
    }
}

/// T203: Filter presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterPreset {
    /// No filter applied
    None,
    /// Show only high CPU processes (CPU > 25%)
    HighCpu,
    /// Show only high memory processes (Memory > 100MB)
    HighMemory,
    /// Show only system-owned processes
    SystemProcesses,
    /// Show only user-owned processes
    UserProcesses,
}

impl FilterPreset {
    /// Get preset label
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "All Processes",
            Self::HighCpu => "High CPU (>25%)",
            Self::HighMemory => "High Memory (>100MB)",
            Self::SystemProcesses => "System Processes",
            Self::UserProcesses => "User Processes",
        }
    }

    /// Get all presets
    pub fn all() -> &'static [FilterPreset] {
        &[
            Self::None,
            Self::HighCpu,
            Self::HighMemory,
            Self::SystemProcesses,
            Self::UserProcesses,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_filter_box_creation() {
        let filter_box = FilterBox::new();
        assert_eq!(filter_box.text(), "");
        assert!(!filter_box.is_focused());
    }

    #[test]
    fn test_set_text() {
        let mut filter_box = FilterBox::new();
        filter_box.set_text("chrome".to_string());
        assert_eq!(filter_box.text(), "chrome");
    }

    #[test]
    fn test_clear() {
        let mut filter_box = FilterBox::new();
        filter_box.set_text("test".to_string());
        filter_box.clear();
        assert_eq!(filter_box.text(), "");
    }

    #[test]
    fn test_on_char() {
        let mut filter_box = FilterBox::new();
        filter_box.on_char('t');
        filter_box.on_char('e');
        filter_box.on_char('s');
        filter_box.on_char('t');
        assert_eq!(filter_box.text(), "test");
    }

    #[test]
    fn test_on_backspace() {
        let mut filter_box = FilterBox::new();
        filter_box.set_text("test".to_string());
        filter_box.on_backspace();
        assert_eq!(filter_box.text(), "tes");
        filter_box.on_backspace();
        assert_eq!(filter_box.text(), "te");
    }

    #[test]
    fn test_debouncing() {
        let mut filter_box = FilterBox::new();
        filter_box.set_text("test".to_string());
        
        // Immediately after change, should not apply
        assert!(!filter_box.should_apply_filter());
        
        // Wait for debounce
        thread::sleep(Duration::from_millis(60));
        assert!(filter_box.should_apply_filter());
        
        // After marking applied, should not apply again
        filter_box.mark_applied();
        assert!(!filter_box.should_apply_filter());
    }

    #[test]
    fn test_focus_state() {
        let mut filter_box = FilterBox::new();
        assert!(!filter_box.is_focused());
        
        filter_box.set_focused(true);
        assert!(filter_box.is_focused());
        
        filter_box.set_focused(false);
        assert!(!filter_box.is_focused());
    }

    #[test]
    fn test_filter_presets() {
        let presets = FilterPreset::all();
        assert_eq!(presets.len(), 5);
        
        for preset in presets {
            assert!(!preset.label().is_empty());
        }
    }

    #[test]
    fn test_preset_labels() {
        assert_eq!(FilterPreset::None.label(), "All Processes");
        assert_eq!(FilterPreset::HighCpu.label(), "High CPU (>25%)");
        assert_eq!(FilterPreset::HighMemory.label(), "High Memory (>100MB)");
    }
}
