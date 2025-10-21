//! Statistical summaries panel for performance metrics

use windows::{
    core::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
};

// Note: Sparkline rendering simplified due to D2D_POINT_2F type unavailability

/// Statistics for a metric
#[derive(Debug, Clone, Copy)]
pub struct MetricStatistics {
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub p95: f32,
}

impl MetricStatistics {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            min: f32::MAX,
            max: f32::MIN,
            avg: 0.0,
            p95: 0.0,
        }
    }

    /// Updates the current value
    pub fn update(&mut self, value: f32) {
        self.current = value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Computes statistics from an array of values
    pub fn compute_from_values(&mut self, values: &[f32]) {
        if values.is_empty() {
            return;
        }

        self.current = *values.last().unwrap_or(&0.0);
        self.min = values.iter().copied().fold(f32::MAX, f32::min);
        self.max = values.iter().copied().fold(f32::MIN, f32::max);
        self.avg = values.iter().sum::<f32>() / values.len() as f32;

        // Calculate p95
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = (sorted.len() as f32 * 0.95) as usize;
        self.p95 = sorted.get(p95_index).copied().unwrap_or(self.max);
    }
}

impl Default for MetricStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Status level for color coding
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusLevel {
    Good,
    Warning,
    Critical,
}

impl StatusLevel {
    pub fn from_percentage(value: f32) -> Self {
        if value < 60.0 {
            StatusLevel::Good
        } else if value < 85.0 {
            StatusLevel::Warning
        } else {
            StatusLevel::Critical
        }
    }

    /// Returns the color associated with this status level
    pub fn color(&self) -> D2D1_COLOR_F {
        match self {
            StatusLevel::Good => D2D1_COLOR_F { r: 0.2, g: 0.8, b: 0.4, a: 1.0 },
            StatusLevel::Warning => D2D1_COLOR_F { r: 1.0, g: 0.8, b: 0.0, a: 1.0 },
            StatusLevel::Critical => D2D1_COLOR_F { r: 1.0, g: 0.2, b: 0.2, a: 1.0 },
        }
    }
}

/// Statistics panel displaying min/max/avg/p95 for metrics
pub struct StatisticsPanel {
    stats: MetricStatistics,
    _title: String,
    show_sparkline: bool,
    sparkline_data: Vec<f32>,
}

impl StatisticsPanel {
    /// Creates a new statistics panel with the specified title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            stats: MetricStatistics::new(),
            _title: title.into(),
            show_sparkline: true,
            sparkline_data: Vec::with_capacity(60),
        }
    }

    /// Updates the panel with a new value and adds it to the sparkline
    pub fn update(&mut self, value: f32) {
        self.stats.update(value);
        
        // Update sparkline data
        self.sparkline_data.push(value);
        if self.sparkline_data.len() > 60 {
            self.sparkline_data.remove(0);
        }

        // Recompute statistics from sparkline
        self.stats.compute_from_values(&self.sparkline_data);
    }

    pub fn set_show_sparkline(&mut self, show: bool) {
        self.show_sparkline = show;
    }

    pub fn statistics(&self) -> &MetricStatistics {
        &self.stats
    }

    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        // Determine status level based on current value
        let status = StatusLevel::from_percentage(self.stats.current);
        let status_color = status.color();

        // Draw current value with large font (primary focus)
        let _value_brush = unsafe { context.CreateSolidColorBrush(&status_color, None)? };
        
        // Draw background
        let bg_color = D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 0.5 };
        let bg_brush = unsafe { context.CreateSolidColorBrush(&bg_color, None)? };
        unsafe {
            context.FillRectangle(bounds, &bg_brush);
        }

        // TODO: Render current value text (large) - requires DirectWrite
        // Format: "{:.1}%", self.stats.current

        // TODO: Render historical statistics (small) - requires DirectWrite
        // Min: {:.1}%  Max: {:.1}%  Avg: {:.1}%  P95: {:.1}%

        // Draw sparkline if enabled
        if self.show_sparkline && !self.sparkline_data.is_empty() {
            unsafe { self.render_sparkline(context, bounds)?; }
        }

        Ok(())
    }

    unsafe fn render_sparkline(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        let sparkline_height = 30.0;
        let sparkline_bounds = D2D_RECT_F {
            left: bounds.left + 10.0,
            top: bounds.bottom - sparkline_height - 10.0,
            right: bounds.right - 10.0,
            bottom: bounds.bottom - 10.0,
        };

        let _width = sparkline_bounds.right - sparkline_bounds.left;
        let _height = sparkline_bounds.bottom - sparkline_bounds.top;
        let count = self.sparkline_data.len();

        if count == 0 {
            return Ok(());
        }

        // Get data range
        let min_val = self.stats.min;
        let max_val = self.stats.max;
        let range = max_val - min_val;

        if range == 0.0 {
            return Ok(());
        }

        // Draw sparkline
        let _line_color = D2D1_COLOR_F { r: 0.6, g: 0.6, b: 0.6, a: 0.8 };
        let _line_brush = unsafe { context.CreateSolidColorBrush(&_line_color, None)? };

        // TODO: Render sparkline points once D2D_POINT_2F type is available
        // For now, skip point rendering
        let _ = (count, min_val, max_val, range);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_update() {
        let mut stats = MetricStatistics::new();
        stats.update(50.0);
        assert_eq!(stats.current, 50.0);
        assert_eq!(stats.min, 50.0);
        assert_eq!(stats.max, 50.0);
    }

    #[test]
    fn test_status_level() {
        assert_eq!(StatusLevel::from_percentage(30.0), StatusLevel::Good);
        assert_eq!(StatusLevel::from_percentage(70.0), StatusLevel::Warning);
        assert_eq!(StatusLevel::from_percentage(90.0), StatusLevel::Critical);
    }

    #[test]
    fn test_p95_calculation() {
        let mut stats = MetricStatistics::new();
        let values: Vec<f32> = (0..100).map(|i| i as f32).collect();
        stats.compute_from_values(&values);
        assert!((stats.p95 - 95.0).abs() < 1.0);
    }
}
