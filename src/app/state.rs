//! Application state management
//!
//! Connects the monitoring system, process store, and UI rendering.

use windows::core::Result;
use windows::Win32::Foundation::HWND;

// Note: Power status APIs not available in windows-rs 0.62
// Using stub implementation for performance mode detection

use crate::core::process::ProcessStore;
use crate::ui::d2d::renderer::Renderer;
use crate::ui::d2d::resources::ResourcePool;
use crate::windows::monitor::SystemMonitor;

/// Performance mode (T468)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    /// Full performance with all features enabled
    Performance,
    /// Battery saver mode with reduced features
    BatterySaver,
    /// User-selected mode (overrides automatic detection)
    Manual(bool), // true = performance, false = battery saver
}

/// Application state coordinating all components
pub struct AppState {
    /// Hardware-accelerated renderer
    renderer: Renderer,
    
    /// Resource pool for brushes and colors
    resources: ResourcePool,
    
    /// Process data store
    process_store: ProcessStore,
    
    /// System monitoring coordinator
    monitor: SystemMonitor,
    
    /// Frame counter for debugging
    frame_count: u64,
    
    /// Performance mode (T468)
    performance_mode: PerformanceMode,
    
    /// Active tab (0=Processes, 1=Performance, 2=Services, 3=GPU)
    active_tab: usize,
    
    /// Window dimensions
    width: u32,
    height: u32,
}

impl AppState {
    /// Create new application state
    /// 
    /// T314: Lazy initialization strategy:
    /// - Renderer: Created immediately (needed for window)
    /// - ResourcePool: Brushes created on first use (T312)
    /// - ProcessStore: Minimal allocation at init
    /// - SystemMonitor: Lightweight, no system queries until update()
    pub fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        // Initialize Direct2D renderer
        let renderer = Renderer::new(hwnd, width, height)?;
        
        // Create resource pool (T312: brushes lazily created on first use)
        let resources = ResourcePool::new(renderer.device_context(), renderer.dwrite_factory())?;
        
        // Initialize process store (SoA arrays, no data collection yet)
        let process_store = ProcessStore::new();
        
        // Create system monitor (lightweight, no queries until update())
        let monitor = SystemMonitor::new();
        
        // Detect initial performance mode (T468)
        let performance_mode = Self::detect_performance_mode();
        
        Ok(Self {
            renderer,
            resources,
            process_store,
            monitor,
            frame_count: 0,
            performance_mode,
            active_tab: 0,
            width,
            height,
        })
    }
    
    /// Update process data from system
    pub fn update(&mut self) -> Result<()> {
        // Collect current system state
        match self.monitor.collect_all() {
            Ok(snapshot) => {
                // Update process store with new data
                self.process_store.update(snapshot.processes);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to collect system data: {}", e);
                Err(windows::core::Error::from(windows::Win32::Foundation::E_FAIL))
            }
        }
    }
    
    /// Render one frame
    pub fn render(&mut self) -> Result<()> {
        self.frame_count += 1;
        
        // Begin frame
        self.renderer.begin_frame();
        
        // Render full UI
        self.render_ui()?;
        
        // End frame and present
        self.renderer.end_frame()?;
        
        Ok(())
    }
    
    /// Render the full UI with tabs, tables, and controls
    fn render_ui(&mut self) -> Result<()> {
        use windows::Win32::Graphics::Direct2D::Common::*;
        
        let dc = self.renderer.device_context();
        let width = self.width as f32;
        let height = self.height as f32;
        
        unsafe {
            // Background
            let bg_rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: width,
                bottom: height,
            };
            dc.FillRectangle(&bg_rect, self.resources.background_brush());
            
            // Tab bar (top 40px)
            let tabs = vec!["Processes", "Performance", "Services", "GPU"];
            let tab_width = width / tabs.len() as f32;
            for (i, tab) in tabs.iter().enumerate() {
                let tab_rect = D2D_RECT_F {
                    left: i as f32 * tab_width,
                    top: 0.0,
                    right: (i + 1) as f32 * tab_width,
                    bottom: 40.0,
                };
                
                // Highlight active tab
                let brush = if i == self.active_tab {
                    self.resources.blue_brush()
                } else {
                    self.resources.header_brush()
                };
                dc.FillRectangle(&tab_rect, brush);
                dc.DrawRectangle(&tab_rect, self.resources.border_brush(), 1.0, None);
                
                // Tab label
                let tab_text: Vec<u16> = tab.encode_utf16().collect();
                let text_rect = D2D_RECT_F {
                    left: tab_rect.left + 10.0,
                    top: tab_rect.top + 8.0,
                    right: tab_rect.right - 10.0,
                    bottom: tab_rect.bottom - 8.0,
                };
                dc.DrawText(
                    &tab_text,
                    if i == self.active_tab {
                        self.resources.bold_text_format()
                    } else {
                        self.resources.default_text_format()
                    },
                    &text_rect,
                    if i == self.active_tab {
                        self.resources.white_brush()
                    } else {
                        self.resources.text_brush()
                    },
                    windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
                    windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                );
            }
            
            // Status bar (bottom 30px)
            let status_rect = D2D_RECT_F {
                left: 0.0,
                top: height - 30.0,
                right: width,
                bottom: height,
            };
            dc.FillRectangle(&status_rect, self.resources.header_brush());
            dc.DrawRectangle(&status_rect, self.resources.border_brush(), 1.0, None);
            
            let status = format!(
                "Processes: {} | CPU: {:.1}% | Memory: N/A | Frame: {}",
                self.process_store.count(),
                0.0, // TODO: Get CPU usage from monitor
                self.frame_count
            );
            let status_wide: Vec<u16> = status.encode_utf16().collect();
            let status_text_rect = D2D_RECT_F {
                left: 10.0,
                top: height - 25.0,
                right: width - 10.0,
                bottom: height - 5.0,
            };
            dc.DrawText(
                &status_wide,
                self.resources.default_text_format(),
                &status_text_rect,
                self.resources.text_brush(),
                windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
                windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
            );
            
            // Content area (between tabs and status bar)
            let content_rect = D2D_RECT_F {
                left: 0.0,
                top: 40.0,
                right: width,
                bottom: height - 30.0,
            };
            
            // Render active tab content
            match self.active_tab {
                0 => self.render_processes_tab(dc, &content_rect)?,
                1 => self.render_performance_tab(dc, &content_rect)?,
                2 => self.render_services_tab(dc, &content_rect)?,
                3 => self.render_gpu_tab(dc, &content_rect)?,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Render Processes tab
    fn render_processes_tab(
        &self,
        dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext,
        rect: &windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    ) -> Result<()> {
        use windows::Win32::Graphics::Direct2D::Common::*;
        
        unsafe {
            // Table header background
            let header_rect = D2D_RECT_F {
                left: rect.left + 10.0,
                top: rect.top + 10.0,
                right: rect.right - 10.0,
                bottom: rect.top + 40.0,
            };
            dc.FillRectangle(&header_rect, self.resources.header_brush());
            dc.DrawRectangle(&header_rect, self.resources.border_brush(), 1.0, None);
            
            // Column headers
            let headers = vec![
                ("Name", 250.0),
                ("PID", 80.0),
                ("CPU %", 100.0),
                ("Memory", 120.0),
                ("Status", 100.0),
            ];
            
            let mut x = rect.left + 20.0;
            for (header, width_col) in headers {
                let text_wide: Vec<u16> = header.encode_utf16().collect();
                let text_rect = D2D_RECT_F {
                    left: x,
                    top: rect.top + 15.0,
                    right: x + width_col,
                    bottom: rect.top + 35.0,
                };
                
                dc.DrawText(
                    &text_wide,
                    self.resources.bold_text_format(),
                    &text_rect,
                    self.resources.text_brush(),
                    windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
                    windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                );
                
                x += width_col;
            }
            
            // Render process table data - show first 20 processes
            let process_count = self.process_store.count().min(20);
            let row_height = 25.0;
            
            for i in 0..process_count {
                let y = rect.top + 45.0 + (i as f32 * row_height);
                
                // Alternate row background
                if i % 2 == 0 {
                    let row_rect = D2D_RECT_F {
                        left: rect.left + 10.0,
                        top: y,
                        right: rect.right - 10.0,
                        bottom: y + row_height,
                    };
                    dc.FillRectangle(&row_rect, self.resources.light_gray_brush());
                }
                
                // Process data (mock for now - TODO: get from process_store)
                let process_text = format!("process_{}.exe\t{}\t{:.1}%\t{} MB\tRunning", 
                    i, 1000 + i, i as f64 * 2.5, (i + 1) * 50);
                let text_wide: Vec<u16> = process_text.encode_utf16().collect();
                
                let text_rect = D2D_RECT_F {
                    left: rect.left + 20.0,
                    top: y + 5.0,
                    right: rect.right - 20.0,
                    bottom: y + row_height - 5.0,
                };
                
                dc.DrawText(
                    &text_wide,
                    self.resources.default_text_format(),
                    &text_rect,
                    self.resources.text_brush(),
                    windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
                    windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }
        
        Ok(())
    }
    
    /// Render Performance tab
    fn render_performance_tab(
        &self,
        dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext,
        rect: &windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    ) -> Result<()> {
        use windows::Win32::Graphics::Direct2D::Common::*;
        use windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE;
        use windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL;
        
        unsafe {
            // Title
            let title = "Performance Monitor";
            let title_wide: Vec<u16> = title.encode_utf16().collect();
            let title_rect = D2D_RECT_F {
                left: rect.left + 20.0,
                top: rect.top + 20.0,
                right: rect.right - 20.0,
                bottom: rect.top + 50.0,
            };
            
            dc.DrawText(
                &title_wide,
                self.resources.title_text_format(),
                &title_rect,
                self.resources.text_brush(),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
            
            // CPU Graph placeholder
            let cpu_graph_rect = D2D_RECT_F {
                left: rect.left + 20.0,
                top: rect.top + 60.0,
                right: rect.left + (rect.right - rect.left) / 2.0 - 10.0,
                bottom: rect.top + 300.0,
            };
            dc.FillRectangle(&cpu_graph_rect, self.resources.white_brush());
            dc.DrawRectangle(&cpu_graph_rect, self.resources.border_brush(), 2.0, None);
            
            let cpu_label = "CPU Usage";
            let cpu_label_wide: Vec<u16> = cpu_label.encode_utf16().collect();
            let cpu_label_rect = D2D_RECT_F {
                left: cpu_graph_rect.left + 10.0,
                top: cpu_graph_rect.top + 10.0,
                right: cpu_graph_rect.right - 10.0,
                bottom: cpu_graph_rect.top + 30.0,
            };
            dc.DrawText(
                &cpu_label_wide,
                self.resources.default_text_format(),
                &cpu_label_rect,
                self.resources.text_brush(),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
            
            // Memory Graph placeholder
            let mem_graph_rect = D2D_RECT_F {
                left: rect.left + (rect.right - rect.left) / 2.0 + 10.0,
                top: rect.top + 60.0,
                right: rect.right - 20.0,
                bottom: rect.top + 300.0,
            };
            dc.FillRectangle(&mem_graph_rect, self.resources.white_brush());
            dc.DrawRectangle(&mem_graph_rect, self.resources.border_brush(), 2.0, None);
            
            let mem_label = "Memory Usage";
            let mem_label_wide: Vec<u16> = mem_label.encode_utf16().collect();
            let mem_label_rect = D2D_RECT_F {
                left: mem_graph_rect.left + 10.0,
                top: mem_graph_rect.top + 10.0,
                right: mem_graph_rect.right - 10.0,
                bottom: mem_graph_rect.top + 30.0,
            };
            dc.DrawText(
                &mem_label_wide,
                self.resources.default_text_format(),
                &mem_label_rect,
                self.resources.text_brush(),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
        
        Ok(())
    }
    
    /// Render Services tab
    fn render_services_tab(
        &self,
        dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext,
        rect: &windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    ) -> Result<()> {
        self.render_placeholder_tab(dc, rect, "Services")
    }
    
    /// Render GPU tab
    fn render_gpu_tab(
        &self,
        dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext,
        rect: &windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    ) -> Result<()> {
        self.render_placeholder_tab(dc, rect, "GPU Monitoring")
    }
    
    /// Render placeholder for unimplemented tabs
    fn render_placeholder_tab(
        &self,
        dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext,
        rect: &windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F,
        title: &str,
    ) -> Result<()> {
        use windows::Win32::Graphics::Direct2D::Common::*;
        use windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE;
        use windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL;
        
        unsafe {
            let message = format!("{}\n\nComing soon...", title);
            let message_wide: Vec<u16> = message.encode_utf16().collect();
            let message_rect = D2D_RECT_F {
                left: rect.left + 50.0,
                top: rect.top + 100.0,
                right: rect.right - 50.0,
                bottom: rect.bottom - 100.0,
            };
            
            dc.DrawText(
                &message_wide,
                self.resources.title_text_format(),
                &message_rect,
                self.resources.text_brush(),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
        
        Ok(())
    }
    
    /// Handle window resize
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        self.renderer.resize(width, height)
    }
    
    /// Get process count
    pub fn process_count(&self) -> usize {
        self.process_store.count()
    }

    /// Detect performance mode based on power status (T468)
    /// Note: Full API not available in windows-rs 0.62 - defaulting to Performance mode
    fn detect_performance_mode() -> PerformanceMode {
        // TODO: Implement GetSystemPowerStatus when API becomes available
        // For now, default to performance mode
        PerformanceMode::Performance
    }

    /// Get current performance mode
    pub fn performance_mode(&self) -> PerformanceMode {
        self.performance_mode
    }

    /// Set manual performance mode (T471)
    pub fn set_performance_mode(&mut self, mode: PerformanceMode) {
        self.performance_mode = mode;
    }

    /// Update performance mode based on power status
    pub fn update_performance_mode(&mut self) {
        // Only update if not in manual mode
        if !matches!(self.performance_mode, PerformanceMode::Manual(_)) {
            self.performance_mode = Self::detect_performance_mode();
        }
    }

    /// Check if animations should be enabled (T470)
    pub fn animations_enabled(&self) -> bool {
        match self.performance_mode {
            PerformanceMode::Performance => true,
            PerformanceMode::BatterySaver => false,
            PerformanceMode::Manual(perf) => perf,
        }
    }

    /// Get suggested refresh rate in milliseconds (T469)
    pub fn suggested_refresh_rate_ms(&self) -> u32 {
        match self.performance_mode {
            PerformanceMode::Performance => 1000,      // 1 Hz
            PerformanceMode::BatterySaver => 2000,     // 0.5 Hz
            PerformanceMode::Manual(true) => 1000,     // 1 Hz
            PerformanceMode::Manual(false) => 2000,    // 0.5 Hz
        }
    }
}
