//! Application state management
//!
//! Connects the monitoring system, process store, and UI rendering.

use windows::core::Result;
use windows::Win32::Foundation::HWND;

use crate::core::process::ProcessStore;
use crate::ui::d2d::renderer::Renderer;
use crate::ui::d2d::resources::ResourcePool;
use crate::windows::monitor::SystemMonitor;

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
}

impl AppState {
    /// Create new application state
    pub fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        // Initialize Direct2D renderer
        let renderer = Renderer::new(hwnd, width, height)?;
        
        // Create resource pool
        let resources = ResourcePool::new(renderer.device_context(), renderer.dwrite_factory())?;
        
        // Initialize process store
        let process_store = ProcessStore::new();
        
        // Create system monitor
        let monitor = SystemMonitor::new();
        
        Ok(Self {
            renderer,
            resources,
            process_store,
            monitor,
            frame_count: 0,
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
        
        // Render simple process information (MVP)
        self.render_simple_info()?;
        
        // End frame and present
        self.renderer.end_frame()?;
        
        Ok(())
    }
    
    /// Render simple process information (minimal working version)
    fn render_simple_info(&self) -> Result<()> {
        use windows::Win32::Graphics::Direct2D::Common::*;
        use windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE;
        use windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL;
        
        let dc = self.renderer.device_context();
        
        unsafe {
            // Draw a simple colored rectangle to verify rendering works
            let rect = D2D_RECT_F {
                left: 100.0,
                top: 100.0,
                right: 1100.0,
                bottom: 700.0,
            };
            
            // Fill with white
            dc.FillRectangle(&rect, self.resources.white_brush());
            
            // Border with black
            dc.DrawRectangle(&rect, self.resources.black_brush(), 2.0, None);
            
            // Draw process count as simple text
            let count = self.process_store.count();
            let title = format!("Rust Task Manager\n\n{} Processes Running\n\nFrame: {}", count, self.frame_count);
            let title_wide: Vec<u16> = title.encode_utf16().collect();
            
            let text_rect = D2D_RECT_F {
                left: 150.0,
                top: 150.0,
                right: 1050.0,
                bottom: 650.0,
            };
            
            // DrawText signature: text, textFormat, layoutRect, defaultFillBrush, options, measuringMode
            dc.DrawText(
                &title_wide,
                self.resources.default_text_format(),
                &text_rect,
                self.resources.black_brush(),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
        
        Ok(())
    }
    
    /// Handle window resize
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.renderer.resize(width, height)
    }
    
    /// Get process count
    pub fn process_count(&self) -> usize {
        self.process_store.count()
    }
}
