//! CPU heat map visualization for multi-core systems
//!
//! Provides a grid-based visualization of per-core CPU usage with color gradients.

use windows::{
    core::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
};

/// CPU heat map widget displaying per-core usage
pub struct HeatMap {
    /// Per-core CPU usage values (0.0-100.0)
    core_values: Vec<f32>,
    /// Previous values for smooth transitions
    prev_values: Vec<f32>,
    /// Grid layout (rows, cols)
    layout: (usize, usize),
    /// Show core labels
    show_labels: bool,
    /// Interpolation factor for smooth transitions (0.0-1.0)
    interpolation: f32,
}

impl HeatMap {
    /// Create a new heat map with the specified core count
    pub fn new(core_count: usize) -> Self {
        let layout = Self::calculate_layout(core_count);
        Self {
            core_values: vec![0.0; core_count],
            prev_values: vec![0.0; core_count],
            layout,
            show_labels: true,
            interpolation: 0.7, // 70% new value, 30% old value for smoothing
        }
    }

    /// Calculate optimal grid layout for the given core count
    fn calculate_layout(core_count: usize) -> (usize, usize) {
        let cols = (core_count as f32).sqrt().ceil() as usize;
        let rows = (core_count + cols - 1) / cols;
        (rows, cols)
    }

    /// Update CPU values for all cores with smooth transition
    pub fn update(&mut self, values: &[f32]) {
        self.prev_values.copy_from_slice(&self.core_values);
        let len = self.core_values.len().min(values.len());
        for i in 0..len {
            // Interpolate between previous and new value for smoothness
            self.core_values[i] = self.prev_values[i] * (1.0 - self.interpolation)
                + values[i] * self.interpolation;
        }
    }

    /// Update CPU value for a single core
    pub fn update_core(&mut self, core_index: usize, value: f32) {
        if core_index < self.core_values.len() {
            self.prev_values[core_index] = self.core_values[core_index];
            self.core_values[core_index] = self.prev_values[core_index] * (1.0 - self.interpolation)
                + value * self.interpolation;
        }
    }

    /// Set interpolation factor (0.0 = no interpolation, 1.0 = instant change)
    pub fn set_interpolation(&mut self, factor: f32) {
        self.interpolation = factor.clamp(0.0, 1.0);
    }

    /// Set label visibility
    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    /// Get the core index at the given point (for hit testing)
    pub fn hit_test(&self, bounds: &D2D_RECT_F, x: f32, y: f32) -> Option<usize> {
        let (rows, cols) = self.layout;
        let padding = 2.0;
        let cell_width = (bounds.right - bounds.left - padding * (cols + 1) as f32) / cols as f32;
        let cell_height = (bounds.bottom - bounds.top - padding * (rows + 1) as f32) / rows as f32;

        let rel_x = x - bounds.left;
        let rel_y = y - bounds.top;

        for index in 0..self.core_values.len() {
            let row = index / cols;
            let col = index % cols;
            let cell_x = padding + (cell_width + padding) * col as f32;
            let cell_y = padding + (cell_height + padding) * row as f32;

            if rel_x >= cell_x && rel_x <= cell_x + cell_width
                && rel_y >= cell_y && rel_y <= cell_y + cell_height
            {
                return Some(index);
            }
        }

        None
    }

    /// Map CPU usage to color gradient (blue -> cyan -> green -> yellow -> red)
    fn value_to_color(&self, value: f32) -> D2D1_COLOR_F {
        let normalized = (value / 100.0).clamp(0.0, 1.0);
        let (r, g, b) = if normalized < 0.25 {
            let t = normalized / 0.25;
            (0.0, t, 1.0)
        } else if normalized < 0.5 {
            let t = (normalized - 0.25) / 0.25;
            (0.0, 1.0, 1.0 - t)
        } else if normalized < 0.75 {
            let t = (normalized - 0.5) / 0.25;
            (t, 1.0, 0.0)
        } else {
            let t = (normalized - 0.75) / 0.25;
            (1.0, 1.0 - t, 0.0)
        };
        D2D1_COLOR_F { r, g, b, a: 1.0 }
    }

    /// Render the heat map
    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        let (rows, cols) = self.layout;
        let padding = 2.0;
        let cell_width = (bounds.right - bounds.left - padding * (cols + 1) as f32) / cols as f32;
        let cell_height = (bounds.bottom - bounds.top - padding * (rows + 1) as f32) / rows as f32;

        for (index, &value) in self.core_values.iter().enumerate() {
            let row = index / cols;
            let col = index % cols;
            let cell_x = bounds.left + padding + (cell_width + padding) * col as f32;
            let cell_y = bounds.top + padding + (cell_height + padding) * row as f32;

            let cell_rect = D2D_RECT_F {
                left: cell_x,
                top: cell_y,
                right: cell_x + cell_width,
                bottom: cell_y + cell_height,
            };

            let color = self.value_to_color(value);
            let brush = unsafe { context.CreateSolidColorBrush(&color, None)? };
            unsafe { context.FillRectangle(&cell_rect, &brush); }

            let border_color = D2D1_COLOR_F { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
            let border_brush = unsafe { context.CreateSolidColorBrush(&border_color, None)? };
            unsafe { context.DrawRectangle(&cell_rect, &border_brush, 1.0, None); }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heatmap_creation() {
        let heatmap = HeatMap::new(8);
        assert_eq!(heatmap.core_values.len(), 8);
        assert_eq!(heatmap.layout, (3, 3));
    }

    #[test]
    fn test_layout_calculation() {
        assert_eq!(HeatMap::calculate_layout(4), (2, 2));
        assert_eq!(HeatMap::calculate_layout(16), (4, 4));
    }
}
