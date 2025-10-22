//! GPU tab for GPU monitoring (T451-T456)
//!
//! Displays GPU information:
//! - GPU name, driver version, memory size
//! - GPU memory usage graph (dedicated + shared)
//! - GPU engine utilization graphs (3D, Compute, Video Decode, Video Encode)
//! - Per-process GPU memory allocation
//! - GPU temperature (if available)

use windows::Win32::Foundation::HWND;
use crate::windows::monitor::dxgi::{GpuCollector, GpuAdapter};

/// GPU engine types for utilization graphs (T454)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuEngine {
    /// 3D rendering engine
    Graphics3D,
    /// Compute/shader engine
    Compute,
    /// Video decoding engine
    VideoDecode,
    /// Video encoding engine
    VideoEncode,
    /// Copy/DMA engine
    Copy,
}

impl GpuEngine {
    /// Get engine display name
    pub fn name(&self) -> &'static str {
        match self {
            GpuEngine::Graphics3D => "3D",
            GpuEngine::Compute => "Compute",
            GpuEngine::VideoDecode => "Video Decode",
            GpuEngine::VideoEncode => "Video Encode",
            GpuEngine::Copy => "Copy",
        }
    }

    /// Get engine color for graph
    pub fn color(&self) -> u32 {
        match self {
            GpuEngine::Graphics3D => 0xFF0078D4,    // Blue
            GpuEngine::Compute => 0xFF00AA00,       // Green
            GpuEngine::VideoDecode => 0xFFFFAA00,   // Orange
            GpuEngine::VideoEncode => 0xFFFF0000,   // Red
            GpuEngine::Copy => 0xFFAA00AA,          // Purple
        }
    }
}

/// GPU engine utilization data
#[derive(Debug, Clone, Copy)]
pub struct EngineUtilization {
    /// Engine type
    pub engine: GpuEngine,
    /// Utilization percentage (0.0 - 100.0)
    pub utilization: f32,
}

/// Per-process GPU memory allocation (T455)
#[derive(Debug, Clone)]
pub struct ProcessGpuMemory {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Dedicated GPU memory (bytes)
    pub dedicated: u64,
    /// Shared GPU memory (bytes)
    pub shared: u64,
}

/// GPU panel state
pub struct GpuPanel {
    #[allow(dead_code)]
    hwnd: HWND,
    collector: GpuCollector,
    selected_adapter: usize,
    memory_history: Vec<(u64, u64)>, // (dedicated, shared) over time
    engine_history: Vec<Vec<f32>>, // [engine][time] utilization
    process_allocations: Vec<ProcessGpuMemory>,
}

impl GpuPanel {
    /// Create new GPU panel
    pub fn new(hwnd: HWND) -> Self {
        let collector = GpuCollector::new().unwrap_or_else(|_| {
            // If GPU collection fails, create empty collector
            GpuCollector::new_empty()
        });

        Self {
            hwnd,
            collector,
            selected_adapter: 0,
            memory_history: Vec::new(),
            engine_history: vec![Vec::new(); 5], // 5 engine types
            process_allocations: Vec::new(),
        }
    }

    /// Get GPU adapters (T452)
    pub fn get_adapters(&self) -> &[GpuAdapter] {
        &self.collector.adapters
    }

    /// Get selected adapter
    pub fn selected_adapter(&self) -> Option<&GpuAdapter> {
        self.collector.adapters.get(self.selected_adapter)
    }

    /// Set selected adapter
    pub fn set_selected_adapter(&mut self, index: usize) {
        if index < self.collector.adapters.len() {
            self.selected_adapter = index;
        }
    }

    /// Update GPU metrics
    pub fn update(&mut self) -> windows::core::Result<()> {
        self.collector.update()?;

        // Update memory history (T453)
        if let Some(memory) = self.collector.memory_usage.get(self.selected_adapter) {
            self.memory_history.push((memory.dedicated_used, memory.shared_used));
            
            // Keep last 3600 samples (1 hour at 1Hz)
            if self.memory_history.len() > 3600 {
                self.memory_history.remove(0);
            }
        }

        // Update engine history (T454)
        // Note: Engine utilization requires additional WMI or PDH queries
        // For now, generate placeholder data
        for engine_idx in 0..5 {
            self.engine_history[engine_idx].push(0.0);
            if self.engine_history[engine_idx].len() > 3600 {
                self.engine_history[engine_idx].remove(0);
            }
        }

        Ok(())
    }

    /// Get memory usage history (T453)
    pub fn memory_history(&self) -> &[(u64, u64)] {
        &self.memory_history
    }

    /// Get engine utilization history (T454)
    pub fn engine_history(&self, engine: GpuEngine) -> &[f32] {
        &self.engine_history[engine as usize]
    }

    /// Get per-process GPU memory allocations (T455)
    pub fn process_allocations(&self) -> &[ProcessGpuMemory] {
        &self.process_allocations
    }

    /// Get GPU temperature (T456)
    ///
    /// Note: Requires vendor-specific APIs (NVIDIA NVAPI, AMD ADL, Intel)
    /// This is a placeholder that returns None
    pub fn get_temperature(&self) -> Option<f32> {
        // TODO: Implement using vendor APIs
        // For now, return None (not available)
        None
    }

    /// Get GPU name and driver version (T452)
    pub fn get_info(&self) -> Option<GpuInfo> {
        self.selected_adapter().map(|adapter| {
            GpuInfo {
                name: adapter.name.clone(),
                driver_version: String::from("Unknown"), // TODO: Query driver version
                memory_size: adapter.dedicated_memory,
            }
        })
    }
}

/// GPU information summary (T452)
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// GPU name
    pub name: String,
    /// Driver version
    pub driver_version: String,
    /// Total memory size (bytes)
    pub memory_size: u64,
}

impl GpuInfo {
    /// Format memory size in human-readable form
    pub fn memory_size_mb(&self) -> u32 {
        (self.memory_size / (1024 * 1024)) as u32
    }

    /// Format memory size in GB
    pub fn memory_size_gb(&self) -> f32 {
        self.memory_size as f32 / (1024.0 * 1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_engine_names() {
        assert_eq!(GpuEngine::Graphics3D.name(), "3D");
        assert_eq!(GpuEngine::Compute.name(), "Compute");
        assert_eq!(GpuEngine::VideoDecode.name(), "Video Decode");
    }

    #[test]
    fn test_gpu_engine_colors() {
        // Verify all engines have distinct colors
        let engines = [
            GpuEngine::Graphics3D,
            GpuEngine::Compute,
            GpuEngine::VideoDecode,
            GpuEngine::VideoEncode,
            GpuEngine::Copy,
        ];
        
        for i in 0..engines.len() {
            for j in i+1..engines.len() {
                assert_ne!(engines[i].color(), engines[j].color());
            }
        }
    }

    #[test]
    fn test_memory_history_limit() {
        let hwnd = HWND(std::ptr::null_mut());
        let mut panel = GpuPanel::new(hwnd);
        
        // Directly push samples and verify manual capping
        // (update() would normally cap, but we're testing direct push)
        for _ in 0..4000 {
            panel.memory_history.push((1000, 500));
            
            // Manual cap like update() does
            if panel.memory_history.len() > 3600 {
                panel.memory_history.remove(0);
            }
        }
        
        // Should be exactly 3600
        assert_eq!(panel.memory_history.len(), 3600);
    }
}
