//! Time range selector for history length configuration

use windows::{
    core::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
};

/// History length options for time-series data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryLength {
    /// Last 1 minute of data
    OneMinute,
    /// Last 5 minutes of data
    FiveMinutes,
    /// Last 1 hour of data
    OneHour,
    /// Last 24 hours of data
    TwentyFourHours,
}

impl HistoryLength {
    /// Returns the duration in seconds
    pub fn seconds(&self) -> u32 {
        match self {
            HistoryLength::OneMinute => 60,
            HistoryLength::FiveMinutes => 300,
            HistoryLength::OneHour => 3600,
            HistoryLength::TwentyFourHours => 86400,
        }
    }

    /// Returns the required circular buffer size for this history length
    pub fn buffer_size(&self) -> usize {
        match self {
            HistoryLength::OneMinute => 60,
            HistoryLength::FiveMinutes => 300,
            HistoryLength::OneHour => 3600,
            HistoryLength::TwentyFourHours => 3600, // Sample every 24 seconds for 24h
        }
    }

    /// Returns the display name for this history length
    pub fn name(&self) -> &'static str {
        match self {
            HistoryLength::OneMinute => "1 minute",
            HistoryLength::FiveMinutes => "5 minutes",
            HistoryLength::OneHour => "1 hour",
            HistoryLength::TwentyFourHours => "24 hours",
        }
    }

    /// Returns a warning message if this history length has significant resource implications
    pub fn warning(&self) -> Option<&'static str> {
        match self {
            HistoryLength::TwentyFourHours => {
                Some("24-hour mode increases memory usage significantly")
            }
            _ => None,
        }
    }

    /// Returns all available history length options
    pub fn all() -> &'static [HistoryLength] {
        &[
            HistoryLength::OneMinute,
            HistoryLength::FiveMinutes,
            HistoryLength::OneHour,
            HistoryLength::TwentyFourHours,
        ]
    }
}

impl Default for HistoryLength {
    fn default() -> Self {
        HistoryLength::OneHour
    }
}

/// Time range selector widget
pub struct TimeRangeSelector {
    selected: HistoryLength,
    expanded: bool,
}

impl TimeRangeSelector {
    /// Creates a new time range selector with the default selection (5 minutes)
    pub fn new() -> Self {
        Self {
            selected: HistoryLength::default(),
            expanded: false,
        }
    }

    /// Returns the currently selected history length
    pub fn selected(&self) -> HistoryLength {
        self.selected
    }

    /// Sets the selected history length
    pub fn set_selected(&mut self, length: HistoryLength) {
        self.selected = length;
        self.expanded = false;
    }

    /// Toggles the dropdown expanded state
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Returns true if the dropdown is currently expanded
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Handles a mouse click at the specified coordinates, returns true if the click was handled
    pub fn handle_click(&mut self, x: f32, y: f32, bounds: &D2D_RECT_F) -> bool {
        if !self.is_in_bounds(x, y, bounds) {
            self.expanded = false;
            return false;
        }

        if !self.expanded {
            self.expanded = true;
            return true;
        }

        // Check which option was clicked
        let option_height = 30.0;
        let base_y = bounds.top + 40.0; // After header

        for (i, length) in HistoryLength::all().iter().enumerate() {
            let option_y = base_y + (i as f32 * option_height);
            if y >= option_y && y < option_y + option_height {
                self.selected = *length;
                self.expanded = false;
                return true;
            }
        }

        false
    }

    fn is_in_bounds(&self, x: f32, y: f32, bounds: &D2D_RECT_F) -> bool {
        x >= bounds.left && x <= bounds.right && y >= bounds.top && y <= bounds.bottom
    }

    /// Renders the time range selector dropdown
    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        // Draw background
        let bg_color = D2D1_COLOR_F { r: 0.15, g: 0.15, b: 0.15, a: 1.0 };
        let bg_brush = unsafe { context.CreateSolidColorBrush(&bg_color, None)? };
        unsafe {
            context.FillRectangle(bounds, &bg_brush);
        }

        // Draw border
        let border_color = D2D1_COLOR_F { r: 0.4, g: 0.4, b: 0.4, a: 1.0 };
        let border_brush = unsafe { context.CreateSolidColorBrush(&border_color, None)? };
        unsafe {
            context.DrawRectangle(bounds, &border_brush, 1.0, None);
        }

        // TODO: Render selected option text - requires DirectWrite
        // Format: "History: {}", self.selected.name()

        // Render dropdown options if expanded
        if self.expanded {
            unsafe { self.render_dropdown(context, bounds)?; }
        }

        Ok(())
    }

    unsafe fn render_dropdown(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        let option_height = 30.0;
        let dropdown_height = (HistoryLength::all().len() as f32) * option_height + 10.0;
        
        let dropdown_bounds = D2D_RECT_F {
            left: bounds.left,
            top: bounds.top + 40.0,
            right: bounds.right,
            bottom: bounds.top + 40.0 + dropdown_height,
        };

        // Draw dropdown background
        let bg_color = D2D1_COLOR_F { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
        let bg_brush = unsafe { context.CreateSolidColorBrush(&bg_color, None)? };
        unsafe {
            context.FillRectangle(&dropdown_bounds, &bg_brush);
        }

        // Draw options
        let mut option_y = dropdown_bounds.top + 5.0;
        for length in HistoryLength::all() {
            let option_bounds = D2D_RECT_F {
                left: dropdown_bounds.left + 5.0,
                top: option_y,
                right: dropdown_bounds.right - 5.0,
                bottom: option_y + option_height,
            };

            // Highlight selected option
            if *length == self.selected {
                let highlight_color = D2D1_COLOR_F { r: 0.3, g: 0.5, b: 0.7, a: 1.0 };
                let highlight_brush = unsafe { context.CreateSolidColorBrush(&highlight_color, None)? };
                unsafe {
                    context.FillRectangle(&option_bounds, &highlight_brush);
                }
            }

            // TODO: Render option text - requires DirectWrite
            // Format: length.name()

            option_y += option_height;
        }

        Ok(())
    }
}

impl Default for TimeRangeSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_length_seconds() {
        assert_eq!(HistoryLength::OneMinute.seconds(), 60);
        assert_eq!(HistoryLength::OneHour.seconds(), 3600);
    }

    #[test]
    fn test_history_length_buffer_size() {
        assert_eq!(HistoryLength::OneMinute.buffer_size(), 60);
        assert_eq!(HistoryLength::OneHour.buffer_size(), 3600);
    }

    #[test]
    fn test_selector() {
        let mut selector = TimeRangeSelector::new();
        assert_eq!(selector.selected(), HistoryLength::OneHour);
        selector.set_selected(HistoryLength::FiveMinutes);
        assert_eq!(selector.selected(), HistoryLength::FiveMinutes);
    }
}
