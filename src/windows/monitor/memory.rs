//! Memory system metrics using GlobalMemoryStatusEx

use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// Memory metrics for the system
#[derive(Debug, Clone, Copy)]
pub struct MemoryMetrics {
    /// Memory load percentage (0-100)
    pub load_percent: u32,
    /// Total physical memory (bytes)
    pub total_physical: u64,
    /// Available physical memory (bytes)
    pub available_physical: u64,
    /// Total page file (bytes)
    pub total_page_file: u64,
    /// Available page file (bytes)
    pub available_page_file: u64,
    /// Total virtual memory (bytes)
    pub total_virtual: u64,
    /// Available virtual memory (bytes)
    pub available_virtual: u64,
}

/// Get current memory statistics
///
/// # Returns
///
/// Ok(MemoryMetrics) with current memory status, or Err if GlobalMemoryStatusEx fails
///
/// # Performance
///
/// This is a fast Win32 API call (<0.1ms typically)
pub fn get_memory_metrics() -> Result<MemoryMetrics, String> {
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };

    // SAFETY: Call GlobalMemoryStatusEx with properly initialized structure
    let result = unsafe { GlobalMemoryStatusEx(&mut mem_status) };

    if result.is_err() {
        return Err("GlobalMemoryStatusEx failed".to_string());
    }

    Ok(MemoryMetrics {
        load_percent: mem_status.dwMemoryLoad,
        total_physical: mem_status.ullTotalPhys,
        available_physical: mem_status.ullAvailPhys,
        total_page_file: mem_status.ullTotalPageFile,
        available_page_file: mem_status.ullAvailPageFile,
        total_virtual: mem_status.ullTotalVirtual,
        available_virtual: mem_status.ullAvailVirtual,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_memory_metrics() {
        let metrics = get_memory_metrics();
        assert!(metrics.is_ok(), "Should get memory metrics");

        let m = metrics.unwrap();
        
        // Sanity checks
        assert!(m.total_physical > 0, "Should have physical memory");
        assert!(m.available_physical <= m.total_physical, "Available <= Total");
        assert!(m.load_percent <= 100, "Load percent should be 0-100");
    }
}
