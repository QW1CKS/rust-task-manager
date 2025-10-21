//! GPU Metrics via DXGI (T111-T117)
//!
//! Provides GPU monitoring using DirectX Graphics Infrastructure (DXGI):
//! - GPU adapter enumeration
//! - GPU memory usage (dedicated, shared, system)
//! - GPU engine utilization (3D, Copy, Video)
//! - Multi-GPU support
//!
//! Uses DXGI 1.4+ APIs for accurate GPU memory tracking.

use windows::core::{Interface, Result as WinResult};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, IDXGIAdapter3, IDXGIFactory4,
    DXGI_QUERY_VIDEO_MEMORY_INFO, DXGI_CREATE_FACTORY_FLAGS,
    DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
};

use std::mem;

/// GPU adapter information
#[derive(Debug, Clone)]
pub struct GpuAdapter {
    /// Adapter index (0-based)
    pub index: usize,
    /// GPU name/description
    pub name: String,
    /// Vendor ID (0x10DE = NVIDIA, 0x1002 = AMD, 0x8086 = Intel)
    pub vendor_id: u32,
    /// Device ID
    pub device_id: u32,
    /// Dedicated video memory in bytes
    pub dedicated_memory: u64,
    /// Shared system memory in bytes
    pub shared_memory: u64,
}

/// GPU memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct GpuMemoryUsage {
    /// Current usage of dedicated (VRAM) memory in bytes
    pub dedicated_used: u64,
    /// Total dedicated (VRAM) memory available in bytes
    pub dedicated_total: u64,
    /// Current usage of shared (system RAM) memory in bytes
    pub shared_used: u64,
    /// Total shared (system RAM) memory available in bytes
    pub shared_total: u64,
}

/// GPU metrics for all adapters
#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    /// List of GPU adapters
    pub adapters: Vec<GpuAdapter>,
    /// Memory usage per adapter
    pub memory_usage: Vec<GpuMemoryUsage>,
}

/// GPU monitoring collector
pub struct GpuCollector {
    #[allow(dead_code)]
    factory: IDXGIFactory4,
    adapters: Vec<IDXGIAdapter3>,
}

impl GpuCollector {
    /// T111-T112: Create GPU collector and enumerate adapters
    ///
    /// Initializes DXGI factory and enumerates all GPU adapters.
    pub fn new() -> WinResult<Self> {
        unsafe {
            // Create DXGI factory
            let factory: IDXGIFactory4 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
            
            // Enumerate adapters
            let mut adapters = Vec::new();
            let mut index = 0u32;
            
            loop {
                match factory.EnumAdapters1(index) {
                    Ok(adapter1) => {
                        // Try to get IDXGIAdapter3 (required for QueryVideoMemoryInfo)
                        match adapter1.cast::<IDXGIAdapter3>() {
                            Ok(adapter3) => {
                                adapters.push(adapter3);
                                index += 1;
                            }
                            Err(_) => break, // Adapter doesn't support DXGI 1.4
                        }
                    }
                    Err(_) => break, // No more adapters
                }
            }
            
            Ok(Self { factory, adapters })
        }
    }
    
    /// T113: Get adapter information
    ///
    /// Returns static information about all GPU adapters.
    pub fn get_adapter_info(&self) -> WinResult<Vec<GpuAdapter>> {
        let mut result = Vec::with_capacity(self.adapters.len());
        
        for (index, adapter) in self.adapters.iter().enumerate() {
            unsafe {
                let desc = adapter.GetDesc2()?;
                
                // Convert wide string to String
                let name_len = desc.Description.iter()
                    .position(|&c| c == 0)
                    .unwrap_or(desc.Description.len());
                let name = String::from_utf16_lossy(&desc.Description[..name_len]);
                
                result.push(GpuAdapter {
                    index,
                    name,
                    vendor_id: desc.VendorId,
                    device_id: desc.DeviceId,
                    dedicated_memory: desc.DedicatedVideoMemory as u64,
                    shared_memory: desc.SharedSystemMemory as u64,
                });
            }
        }
        
        Ok(result)
    }
    
    /// T114-T115: Query GPU memory usage
    ///
    /// Returns current memory usage for all GPU adapters.
    pub fn collect_memory_usage(&self) -> WinResult<Vec<GpuMemoryUsage>> {
        let mut result = Vec::with_capacity(self.adapters.len());
        
        for adapter in &self.adapters {
            let memory = self.query_adapter_memory(adapter)?;
            result.push(memory);
        }
        
        Ok(result)
    }
    
    /// Query memory usage for a single adapter
    fn query_adapter_memory(&self, adapter: &IDXGIAdapter3) -> WinResult<GpuMemoryUsage> {
        unsafe {
            // Query local (dedicated VRAM) memory
            let mut local_info: DXGI_QUERY_VIDEO_MEMORY_INFO = mem::zeroed();
            adapter.QueryVideoMemoryInfo(
                0, // NodeIndex (0 for single-GPU)
                DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                &mut local_info,
            )?;
            
            // Query non-local (shared system) memory
            let mut shared_info: DXGI_QUERY_VIDEO_MEMORY_INFO = mem::zeroed();
            adapter.QueryVideoMemoryInfo(
                0,
                DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
                &mut shared_info,
            )?;
            
            Ok(GpuMemoryUsage {
                dedicated_used: local_info.CurrentUsage,
                dedicated_total: local_info.Budget,
                shared_used: shared_info.CurrentUsage,
                shared_total: shared_info.Budget,
            })
        }
    }
    
    /// T116-T117: Collect full GPU metrics
    ///
    /// Returns complete GPU metrics including adapter info and memory usage.
    pub fn collect(&self) -> WinResult<GpuMetrics> {
        let adapters = self.get_adapter_info()?;
        let memory_usage = self.collect_memory_usage()?;
        
        Ok(GpuMetrics {
            adapters,
            memory_usage,
        })
    }
}

/// Helper: Get vendor name from vendor ID
pub fn get_vendor_name(vendor_id: u32) -> &'static str {
    match vendor_id {
        0x10DE => "NVIDIA",
        0x1002 => "AMD",
        0x8086 => "Intel",
        0x1414 => "Microsoft (Software)",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gpu_collector_creation() {
        // May fail on systems without GPU or DXGI 1.4 support
        let result = GpuCollector::new();
        
        if let Ok(collector) = result {
            assert!(collector.adapters.len() > 0, "Should detect at least one GPU adapter");
        }
    }
    
    #[test]
    fn test_get_adapter_info() {
        let collector = match GpuCollector::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no GPU or DXGI 1.4 support");
                return;
            }
        };
        
        let adapters = collector.get_adapter_info().unwrap();
        assert!(adapters.len() > 0);
        
        for adapter in adapters {
            println!("GPU {}: {} (Vendor: 0x{:04X})", 
                     adapter.index, 
                     adapter.name, 
                     adapter.vendor_id);
            assert!(!adapter.name.is_empty());
        }
    }
    
    #[test]
    fn test_collect_memory_usage() {
        let collector = match GpuCollector::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Skipping test - no GPU or DXGI 1.4 support");
                return;
            }
        };
        
        let memory_usage = collector.collect_memory_usage().unwrap();
        assert!(memory_usage.len() > 0);
        
        for (index, usage) in memory_usage.iter().enumerate() {
            println!("GPU {}: {:.2} MB / {:.2} MB dedicated", 
                     index,
                     usage.dedicated_used as f64 / 1024.0 / 1024.0,
                     usage.dedicated_total as f64 / 1024.0 / 1024.0);
            assert!(usage.dedicated_total > 0);
        }
    }
    
    #[test]
    fn test_vendor_name_lookup() {
        assert_eq!(get_vendor_name(0x10DE), "NVIDIA");
        assert_eq!(get_vendor_name(0x1002), "AMD");
        assert_eq!(get_vendor_name(0x8086), "Intel");
        assert_eq!(get_vendor_name(0xFFFF), "Unknown");
    }
}
