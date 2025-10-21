//! Performance monitoring panel with hardware-accelerated graphs

use crate::core::metrics::SystemMetrics;
use crate::ui::controls::graph::{Graph, LineGraph, MultiLineGraph, ScaleMode, GraphAxis};
use crate::ui::layout::Rect;
use windows::{
    core::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
};

/// Performance panel displaying system metrics graphs
pub struct PerformancePanel {
    cpu_graph: LineGraph,
    memory_graph: LineGraph,
    per_core_graph: MultiLineGraph,
    percent_axis: GraphAxis,
    time_axis: GraphAxis,
    layout_mode: LayoutMode,
    sync_timeline: bool,
    sync_crosshair: bool,
}

/// Layout mode for the performance panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// 2x2 grid layout
    Grid2x2,
    /// 3x2 grid layout
    Grid3x2,
    /// Single maximized graph
    Maximized,
}

impl PerformancePanel {
    /// Create a new performance panel
    pub fn new() -> Self {
        let cpu_color = D2D1_COLOR_F { r: 0.2, g: 0.6, b: 1.0, a: 1.0 };
        let mut cpu_graph = LineGraph::new(cpu_color, "CPU Usage");
        cpu_graph.set_scale_mode(ScaleMode::Fixed { min: 0.0, max: 100.0 });

        let memory_color = D2D1_COLOR_F { r: 0.2, g: 0.8, b: 0.4, a: 1.0 };
        let mut memory_graph = LineGraph::new(memory_color, "Memory Usage");
        memory_graph.set_scale_mode(ScaleMode::Auto);

        let per_core_graph = MultiLineGraph::new();

        let mut percent_axis = GraphAxis::vertical();
        percent_axis.set_percent_labels();

        let mut time_axis = GraphAxis::horizontal();
        time_axis.set_time_labels(60);

        Self {
            cpu_graph,
            memory_graph,
            per_core_graph,
            percent_axis,
            time_axis,
            layout_mode: LayoutMode::Grid2x2,
            sync_timeline: true,
            sync_crosshair: true,
        }
    }

    /// Set layout mode
    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
    }

    /// Get current layout mode
    pub fn layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }

    /// Enable/disable timeline synchronization
    pub fn set_sync_timeline(&mut self, sync: bool) {
        self.sync_timeline = sync;
    }

    /// Enable/disable crosshair synchronization
    pub fn set_sync_crosshair(&mut self, sync: bool) {
        self.sync_crosshair = sync;
    }

    /// Update graphs with new system metrics
    pub fn update(&mut self, metrics: &SystemMetrics) {
        self.cpu_graph.add_data_point(metrics.cpu_total);
        let memory_used = metrics.memory_total - metrics.memory_available;
        let memory_mb = memory_used as f32 / 1024.0 / 1024.0;
        self.memory_graph.add_data_point(memory_mb);

        for (core_index, &core_usage) in metrics.cpu_cores.iter().enumerate() {
            self.per_core_graph.add_data_point(core_index, core_usage);
        }
    }

    /// Render the performance panel with layout
    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: Rect) -> Result<()> {
        unsafe {
            match self.layout_mode {
                LayoutMode::Grid2x2 => self.render_grid_2x2(context, bounds),
                LayoutMode::Grid3x2 => self.render_grid_3x2(context, bounds),
                LayoutMode::Maximized => self.render_maximized(context, bounds),
            }
        }
    }

    unsafe fn render_grid_2x2(&self, context: &ID2D1DeviceContext, bounds: Rect) -> Result<()> {
        let half_width = bounds.width / 2.0;
        let half_height = bounds.height / 2.0;
        let padding = 5.0;

        // Top-left: CPU graph
        let cpu_rect = D2D_RECT_F {
            left: bounds.x + padding,
            top: bounds.y + padding,
            right: bounds.x + half_width - padding,
            bottom: bounds.y + half_height - padding,
        };
        unsafe {
            self.percent_axis.render(context, &cpu_rect)?;
            self.cpu_graph.render(context, &cpu_rect)?;
        }

        // Top-right: Memory graph
        let mem_rect = D2D_RECT_F {
            left: bounds.x + half_width + padding,
            top: bounds.y + padding,
            right: bounds.x + bounds.width - padding,
            bottom: bounds.y + half_height - padding,
        };
        unsafe {
            self.memory_graph.render(context, &mem_rect)?;
        }

        // Bottom: Per-core graph
        let core_rect = D2D_RECT_F {
            left: bounds.x + padding,
            top: bounds.y + half_height + padding,
            right: bounds.x + bounds.width - padding,
            bottom: bounds.y + bounds.height - padding,
        };
        unsafe {
            self.per_core_graph.render(context, &core_rect)?;
        }

        Ok(())
    }

    unsafe fn render_grid_3x2(&self, context: &ID2D1DeviceContext, bounds: Rect) -> Result<()> {
        // Similar to 2x2 but with 3 columns
        let third_width = bounds.width / 3.0;
        let half_height = bounds.height / 2.0;
        let padding = 5.0;

        let cpu_rect = D2D_RECT_F {
            left: bounds.x + padding,
            top: bounds.y + padding,
            right: bounds.x + third_width - padding,
            bottom: bounds.y + half_height - padding,
        };
        unsafe {
            self.cpu_graph.render(context, &cpu_rect)?;
        }

        Ok(())
    }

    unsafe fn render_maximized(&self, context: &ID2D1DeviceContext, bounds: Rect) -> Result<()> {
        let rect = D2D_RECT_F {
            left: bounds.x,
            top: bounds.y,
            right: bounds.x + bounds.width,
            bottom: bounds.y + bounds.height,
        };

        unsafe {
            self.percent_axis.render(context, &rect)?;
            self.time_axis.render(context, &rect)?;
            self.cpu_graph.render(context, &rect)?;
        }

        Ok(())
    }
}

impl Default for PerformancePanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple statistics panel for a single metric
pub struct StatisticsPanel {
    current: f32,
    _name: String,
}

impl StatisticsPanel {
    /// Create a new statistics panel
    pub fn new(name: impl Into<String>, _unit: impl Into<String>) -> Self {
        Self {
            current: 0.0,
            _name: name.into(),
        }
    }

    /// Update statistics with new value
    pub fn update(&mut self, value: f32) {
        self.current = value;
    }

    /// Render the statistics panel
    pub unsafe fn render(&self, context: &ID2D1DeviceContext, bounds: &D2D_RECT_F) -> Result<()> {
        let bg_color = D2D1_COLOR_F { r: 0.15, g: 0.15, b: 0.15, a: 0.8 };
        let brush = unsafe { context.CreateSolidColorBrush(&bg_color, None)? };
        unsafe { context.FillRectangle(bounds, &brush); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_panel_creation() {
        let _panel = PerformancePanel::new();
    }

    #[test]
    fn test_statistics_panel() {
        let mut stats = StatisticsPanel::new("CPU", "%");
        stats.update(50.0);
        assert_eq!(stats.current, 50.0);
    }
}
