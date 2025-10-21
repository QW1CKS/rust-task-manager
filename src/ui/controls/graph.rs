//! Graph rendering for performance visualization

use windows::{
    core::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
};

const MAX_DATA_POINTS: usize = 3600;

// Workaround for missing D2D_POINT_2F - use the Common module's D2D_POINT_2F if available
// Otherwise just skip rendering points for now

/// Graph rendering trait
pub trait Graph {
    /// Render the graph
    unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()>;
    /// Add a data point
    fn add_data_point(&mut self, value: f32);
    /// Get range (min, max)
    fn get_range(&self) -> (f32, f32);
    /// Get point count
    fn point_count(&self) -> usize;
}

/// Circular buffer for graph data
pub struct CircularBuffer {
    data: Vec<f32>,
    capacity: usize,
    head: usize,
    count: usize,
}

impl CircularBuffer {
    /// Creates a new circular buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0.0; capacity],
            capacity,
            head: 0,
            count: 0,
        }
    }

    /// Adds a value to the buffer, overwriting the oldest value if full
    pub fn push(&mut self, value: f32) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Gets the value at the specified index (0 = oldest, count-1 = newest)
    pub fn get(&self, index: usize) -> Option<f32> {
        if index >= self.count {
            return None;
        }
        let actual_index = (self.head + self.capacity - self.count + index) % self.capacity;
        Some(self.data[actual_index])
    }

    /// Returns the number of values currently in the buffer
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the buffer contains no values
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the min and max values in the buffer
    pub fn range(&self) -> (f32, f32) {
        if self.count == 0 {
            return (0.0, 100.0);
        }
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for i in 0..self.count {
            if let Some(val) = self.get(i) {
                min = min.min(val);
                max = max.max(val);
            }
        }
        (min, max)
    }

    /// Clears all values from the buffer
    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
    }
}

/// Y-axis scaling mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleMode {
    /// Automatically scale to fit data range
    Auto,
    /// Use fixed min/max values
    Fixed {
        /// Minimum Y value
        min: f32,
        /// Maximum Y value
        max: f32
    },
}

/// Line graph
pub struct LineGraph {
    buffer: CircularBuffer,
    scale_mode: ScaleMode,
    color: D2D1_COLOR_F,
    line_width: f32,
    title: String,
    zoom_level: f32,
    pan_offset: f32,
}

impl LineGraph {
    /// Creates a new line graph with the specified color and title
    pub fn new(color: D2D1_COLOR_F, title: impl Into<String>) -> Self {
        Self {
            buffer: CircularBuffer::new(MAX_DATA_POINTS),
            scale_mode: ScaleMode::Fixed { min: 0.0, max: 100.0 },
            color,
            line_width: 1.5,
            title: title.into(),
            zoom_level: 1.0,
            pan_offset: 0.0,
        }
    }

    /// Sets the Y-axis scaling mode
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
    }

    /// Sets the line color
    pub fn set_color(&mut self, color: D2D1_COLOR_F) {
        self.color = color;
    }

    /// Sets the line width in pixels
    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }

    /// Returns the graph title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Zooms the graph by the specified delta (positive = zoom in, negative = zoom out)
    pub fn zoom(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level + delta).max(0.5).min(10.0);
    }

    /// Pans the graph horizontally by the specified delta
    pub fn pan(&mut self, delta: f32) {
        self.pan_offset = (self.pan_offset + delta).max(-1.0).min(1.0);
    }

    /// Resets zoom and pan to default values
    pub fn reset_view(&mut self) {
        self.zoom_level = 1.0;
        self.pan_offset = 0.0;
    }
}

impl Graph for LineGraph {
    unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let _brush = unsafe { context.CreateSolidColorBrush(&self.color, None)? };
        
        // Get data range for coordinate transformation
        let (min_val, max_val) = self.get_range();
        let range = max_val - min_val;
        if range == 0.0 {
            return Ok(());
        }

        // Coordinate transformation: data space -> screen space
        let _width = bounds.right - bounds.left;
        let _height = bounds.bottom - bounds.top;
        let _count = self.buffer.len();
        
        // Draw lines between consecutive points (simplified without path geometry)
        // TODO: Use path geometry for proper line drawing once D2D types are resolved
        
        // For now, skip point rendering due to D2D_POINT_2F type unavailability
        // Points would be rendered here using FillEllipse with proper point coordinates

        Ok(())
    }

    fn add_data_point(&mut self, value: f32) {
        self.buffer.push(value);
    }

    fn get_range(&self) -> (f32, f32) {
        match self.scale_mode {
            ScaleMode::Auto => self.buffer.range(),
            ScaleMode::Fixed { min, max } => (min, max),
        }
    }

    fn point_count(&self) -> usize {
        self.buffer.len()
    }
}

/// Multi-series graph
pub struct MultiLineGraph {
    series: Vec<LineGraph>,
    legend_visible: bool,
}

impl MultiLineGraph {
    /// Creates a new multi-line graph with no series
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            legend_visible: true,
        }
    }

    pub fn add_series(&mut self, graph: LineGraph) {
        self.series.push(graph);
    }

    pub fn add_data_point(&mut self, series_index: usize, value: f32) {
        if let Some(series) = self.series.get_mut(series_index) {
            series.add_data_point(value);
        }
    }

    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    pub fn set_legend_visible(&mut self, visible: bool) {
        self.legend_visible = visible;
    }

    pub fn toggle_series(&mut self, _series_index: usize) {
        // TODO: Implement series visibility toggling
    }

    /// Renders all series in the graph
    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        // Render all series
        for series in &self.series {
            unsafe { series.render(context, bounds)?; }
        }

        // Render legend if visible
        if self.legend_visible && !self.series.is_empty() {
            unsafe { self.render_legend(context, bounds)?; }
        }

        Ok(())
    }

    unsafe fn render_legend(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        let legend_x = bounds.right - 150.0;
        let mut legend_y = bounds.top + 10.0;
        let legend_width = 140.0;
        let legend_height = (self.series.len() as f32) * 25.0 + 10.0;

        // Draw legend background
        let bg_color = D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 0.8 };
        let bg_brush = unsafe { context.CreateSolidColorBrush(&bg_color, None)? };
        unsafe {
            context.FillRectangle(
                &D2D_RECT_F {
                    left: legend_x,
                    top: legend_y,
                    right: legend_x + legend_width,
                    bottom: legend_y + legend_height,
                },
                &bg_brush,
            );
        }

        // Draw series entries
        legend_y += 10.0;
        for series in &self.series {
            let line_brush = unsafe { context.CreateSolidColorBrush(&series.color, None)? };
            
            // Draw color indicator
            unsafe {
                context.FillRectangle(
                    &D2D_RECT_F {
                        left: legend_x + 10.0,
                        top: legend_y,
                        right: legend_x + 30.0,
                        bottom: legend_y + 15.0,
                    },
                    &line_brush,
                );
            }

            // TODO: Draw series name (requires DirectWrite text rendering)
            legend_y += 25.0;
        }

        Ok(())
    }
}

impl Default for MultiLineGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph axis renderer
pub struct GraphAxis {
    vertical: bool,
    labels: Vec<(f32, String)>,
    grid_visible: bool,
}

impl GraphAxis {
    /// Creates a vertical axis (Y-axis)
    pub fn vertical() -> Self {
        Self {
            vertical: true,
            labels: Vec::new(),
            grid_visible: true,
        }
    }

    /// Creates a horizontal axis (X-axis)
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            labels: Vec::new(),
            grid_visible: true,
        }
    }

    /// Sets whether grid lines should be visible
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.grid_visible = visible;
    }

    /// Adds a label at the specified position (0.0 = start, 1.0 = end)
    pub fn add_label(&mut self, position: f32, text: impl Into<String>) {
        self.labels.push((position, text.into()));
    }

    /// Clears all labels
    pub fn clear_labels(&mut self) {
        self.labels.clear();
    }

    /// Sets time-based labels for the X-axis
    pub fn set_time_labels(&mut self, max_seconds: u32) {
        self.clear_labels();
        // Add time labels from most recent (0s) to oldest
        for i in 0..=4 {
            let seconds = (max_seconds as f32 * i as f32 / 4.0) as u32;
            let pos = 1.0 - (i as f32 / 4.0); // Reverse: 1.0 = now, 0.0 = oldest
            self.add_label(pos, format!("{seconds}s"));
        }
    }

    /// Sets percentage labels for the Y-axis (0%, 25%, 50%, 75%, 100%)
    pub fn set_percent_labels(&mut self) {
        self.clear_labels();
        for i in 0..=4 {
            let percent = i * 25;
            let pos = i as f32 / 4.0;
            self.add_label(pos, format!("{percent}%"));
        }
    }

    /// Renders the axis with labels and grid lines
    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        if !self.grid_visible {
            return Ok(());
        }

        let grid_color = D2D1_COLOR_F { r: 0.5, g: 0.5, b: 0.5, a: 0.15 };
        let _brush = unsafe { context.CreateSolidColorBrush(&grid_color, None)? };

        // Draw grid lines at label positions
        for (position, _label) in &self.labels {
            if self.vertical {
                // Horizontal grid lines for vertical axis
                let _y = bounds.bottom - (position * (bounds.bottom - bounds.top));
                // TODO: DrawLine calls require D2D_POINT_2F - pending type resolution
                // context.DrawLine(D2D_POINT_2F { x: bounds.left, y }, D2D_POINT_2F { x: bounds.right, y }, &brush, 0.5, None);
            } else {
                // Vertical grid lines for horizontal axis
                let _x = bounds.left + (position * (bounds.right - bounds.left));
                // TODO: DrawLine calls require D2D_POINT_2F - pending type resolution
                // context.DrawLine(D2D_POINT_2F { x, y: bounds.top }, D2D_POINT_2F { x, y: bounds.bottom }, &brush, 0.5, None);
            }
        }

        // TODO: Render label text (requires DirectWrite)

        Ok(())
    }
}

/// Interactive tooltip
pub struct GraphTooltip {
    visible: bool,
    value: f32,
    timestamp: String,
    pinned: bool,
    position: (f32, f32),
}

impl GraphTooltip {
    /// Creates a new hidden tooltip
    pub fn new() -> Self {
        Self {
            visible: false,
            value: 0.0,
            timestamp: String::new(),
            pinned: false,
            position: (0.0, 0.0),
        }
    }

    pub fn show(&mut self, value: f32, timestamp: impl Into<String>, x: f32, y: f32) {
        self.visible = true;
        self.value = value;
        self.timestamp = timestamp.into();
        self.position = (x, y);
    }

    pub fn hide(&mut self) {
        if !self.pinned {
            self.visible = false;
        }
    }

    /// Toggles whether the tooltip is pinned (stays visible)
    pub fn toggle_pin(&mut self) {
        self.pinned = !self.pinned;
    }

    /// Returns true if the tooltip is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns true if the tooltip is pinned
    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    /// Renders the tooltip at its current position
    pub unsafe fn render(&self, context: &ID2D1DeviceContext) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let tooltip_width = 120.0;
        let tooltip_height = 50.0;
        let (x, y) = self.position;

        // Draw tooltip background
        let bg_color = D2D1_COLOR_F { r: 0.2, g: 0.2, b: 0.2, a: 0.9 };
        let bg_brush = unsafe { context.CreateSolidColorBrush(&bg_color, None)? };
        
        unsafe {
            context.FillRectangle(
                &D2D_RECT_F {
                    left: x,
                    top: y - tooltip_height - 10.0,
                    right: x + tooltip_width,
                    bottom: y - 10.0,
                },
                &bg_brush,
            );

            // Draw border
            let border_color = D2D1_COLOR_F { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
            let border_brush = context.CreateSolidColorBrush(&border_color, None)?;
            context.DrawRectangle(
                &D2D_RECT_F {
                    left: x,
                    top: y - tooltip_height - 10.0,
                    right: x + tooltip_width,
                    bottom: y - 10.0,
                },
                &border_brush,
                1.0,
                None,
            );
        }

        // TODO: Render tooltip text (requires DirectWrite)
        // Text should show: self.timestamp and self.value

        Ok(())
    }
}

impl Default for GraphTooltip {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_buffer() {
        let mut buffer = CircularBuffer::new(5);
        buffer.push(1.0);
        buffer.push(2.0);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.get(0), Some(1.0));
    }

    #[test]
    fn test_line_graph() {
        let color = D2D1_COLOR_F { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        let mut graph = LineGraph::new(color, "Test");
        graph.add_data_point(50.0);
        assert_eq!(graph.point_count(), 1);
    }
}
