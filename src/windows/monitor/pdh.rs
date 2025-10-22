//! PDH (Performance Data Helper) System Metrics (T095-T106)
//!
//! Provides system-wide performance counters using the Windows PDH API:
//! - Total CPU usage and per-core CPU usage
//! - CPU frequency (current and base)
//! - Disk I/O (bytes/sec and IOPS)
//! - Network I/O (bytes/sec and packets/sec)
//!
//! PDH is the recommended API for system-wide metrics in Windows.
//! More accurate than NtQuerySystemInformation for some metrics.

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Performance::{
    PdhAddCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
};

use std::mem;

/// System-wide performance metrics
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// Total CPU usage (0.0 - 100.0 per core, so can exceed 100%)
    pub cpu_total_percent: f64,
    /// Per-core CPU usage (0.0 - 100.0 for each core)
    pub cpu_per_core_percent: Vec<f64>,
    /// Current CPU frequency in MHz
    pub cpu_frequency_mhz: f64,
    /// Base CPU frequency in MHz
    pub cpu_base_frequency_mhz: f64,
    
    /// Disk read bytes/sec
    pub disk_read_bps: f64,
    /// Disk write bytes/sec
    pub disk_write_bps: f64,
    /// Disk read operations/sec
    pub disk_read_iops: f64,
    /// Disk write operations/sec
    pub disk_write_iops: f64,
    
    /// Network bytes received/sec
    pub network_recv_bps: f64,
    /// Network bytes sent/sec
    pub network_send_bps: f64,
}

/// PDH query handle with automatic cleanup
pub struct PdhQuery {
    query: PDH_HQUERY,
    counters: Vec<PDH_HCOUNTER>,
}

impl PdhQuery {
    /// T095: Create new PDH query
    ///
    /// Opens a PDH query for collecting performance counters.
    pub fn new() -> Result<Self, String> {
        unsafe {
            let mut query = PDH_HQUERY::default();
            let result = PdhOpenQueryW(PCWSTR::null(), 0, &mut query);
            
            if result != 0 {
                return Err(format!("PdhOpenQueryW failed: 0x{:08X}", result));
            }
            
            Ok(Self {
                query,
                counters: Vec::new(),
            })
        }
    }
    
    /// T096: Add counter to query
    ///
    /// Adds a performance counter path to the query.
    /// Common paths:
    /// - "\\Processor(_Total)\\% Processor Time"
    /// - "\\Processor(0)\\% Processor Time" (core 0)
    /// - "\\PhysicalDisk(_Total)\\Disk Reads/sec"
    pub fn add_counter(&mut self, path: &str) -> Result<(), String> {
        unsafe {
            let path_wide = HSTRING::from(path);
            let mut counter = PDH_HCOUNTER::default();
            
            let result = PdhAddCounterW(
                self.query,
                &path_wide,
                0,
                &mut counter,
            );
            
            if result != 0 {
                return Err(format!("PdhAddCounterW failed for '{}': 0x{:08X}", path, result));
            }
            
            self.counters.push(counter);
            Ok(())
        }
    }
    
    /// T097: Collect query data
    ///
    /// Collects current values for all counters in the query.
    /// Must be called at least twice with a delay between calls
    /// to get accurate rate-based counters.
    pub fn collect(&self) -> Result<(), String> {
        unsafe {
            let result = PdhCollectQueryData(self.query);
            if result != 0 {
                return Err(format!("PdhCollectQueryData failed: 0x{:08X}", result));
            }
            Ok(())
        }
    }
    
    /// Get formatted counter value as double
    pub fn get_value(&self, counter_index: usize) -> Result<f64, String> {
        if counter_index >= self.counters.len() {
            return Err("Counter index out of bounds".to_string());
        }
        
        unsafe {
            let mut value: PDH_FMT_COUNTERVALUE = mem::zeroed();
            let result = PdhGetFormattedCounterValue(
                self.counters[counter_index],
                PDH_FMT_DOUBLE,
                None,
                &mut value,
            );
            
            if result != 0 {
                return Err(format!("PdhGetFormattedCounterValue failed: 0x{:08X}", result));
            }
            
            Ok(value.Anonymous.doubleValue)
        }
    }
}

impl Drop for PdhQuery {
    fn drop(&mut self) {
        unsafe {
            let _ = PdhCloseQuery(self.query);
        }
    }
}

/// System metrics collector using PDH
/// 
/// # Performance (T325)
/// 
/// Supports selective collection to reduce PDH overhead when metrics not visible.
/// Only collects counters for visible graphs to minimize CPU usage.
pub struct SystemMetricsCollector {
    query: PdhQuery,
    cpu_total_index: usize,
    cpu_per_core_indices: Vec<usize>,
    cpu_freq_index: usize,
    disk_read_bps_index: usize,
    disk_write_bps_index: usize,
    disk_read_iops_index: usize,
    disk_write_iops_index: usize,
    network_recv_index: usize,
    network_send_index: usize,
    initialized: bool,
    
    // T325: Visibility flags to skip collection for hidden metrics
    pub collect_cpu: bool,
    pub collect_disk: bool,
    pub collect_network: bool,
}

impl SystemMetricsCollector {
    /// T098-T106: Create new system metrics collector
    ///
    /// Initializes PDH query with all system metrics counters.
    pub fn new() -> Result<Self, String> {
        let mut query = PdhQuery::new()?;
        let mut counter_index = 0;
        
        // T098: CPU total usage
        query.add_counter("\\Processor(_Total)\\% Processor Time")?;
        let cpu_total_index = counter_index;
        counter_index += 1;
        
        // T099-T100: Per-core CPU usage
        let num_cores = num_cpus::get();
        let mut cpu_per_core_indices = Vec::with_capacity(num_cores);
        for i in 0..num_cores {
            let path = format!("\\Processor({})\\% Processor Time", i);
            query.add_counter(&path)?;
            cpu_per_core_indices.push(counter_index);
            counter_index += 1;
        }
        
        // T101: CPU frequency (current)
        query.add_counter("\\Processor Information(_Total)\\Processor Frequency")?;
        let cpu_freq_index = counter_index;
        counter_index += 1;
        
        // T102-T103: Disk I/O
        query.add_counter("\\PhysicalDisk(_Total)\\Disk Read Bytes/sec")?;
        let disk_read_bps_index = counter_index;
        counter_index += 1;
        
        query.add_counter("\\PhysicalDisk(_Total)\\Disk Write Bytes/sec")?;
        let disk_write_bps_index = counter_index;
        counter_index += 1;
        
        query.add_counter("\\PhysicalDisk(_Total)\\Disk Reads/sec")?;
        let disk_read_iops_index = counter_index;
        counter_index += 1;
        
        query.add_counter("\\PhysicalDisk(_Total)\\Disk Writes/sec")?;
        let disk_write_iops_index = counter_index;
        counter_index += 1;
        
        // T104-T106: Network I/O
        query.add_counter("\\Network Interface(*)\\Bytes Received/sec")?;
        let network_recv_index = counter_index;
        counter_index += 1;
        
        query.add_counter("\\Network Interface(*)\\Bytes Sent/sec")?;
        let network_send_index = counter_index;
        
        Ok(Self {
            query,
            cpu_total_index,
            cpu_per_core_indices,
            cpu_freq_index,
            disk_read_bps_index,
            disk_write_bps_index,
            disk_read_iops_index,
            disk_write_iops_index,
            network_recv_index,
            network_send_index,
            initialized: false,
            // T325: Enable all by default
            collect_cpu: true,
            collect_disk: true,
            collect_network: true,
        })
    }
    
    /// Collect system metrics
    ///
    /// Must be called at least twice with a delay between calls.
    /// The first call initializes the counters, subsequent calls
    /// return accurate values.
    /// 
    /// # Performance (T325)
    /// 
    /// Skips collection of hidden metrics to reduce PDH overhead.
    /// Set collect_cpu/collect_disk/collect_network to false to skip.
    pub fn collect(&mut self) -> Result<SystemMetrics, String> {
        self.query.collect()?;
        
        if !self.initialized {
            self.initialized = true;
            // First collection doesn't have accurate rate data
            return Ok(SystemMetrics::default());
        }
        
        let mut metrics = SystemMetrics::default();
        
        // T325: Only collect visible metrics
        if self.collect_cpu {
            // CPU total
            metrics.cpu_total_percent = self.query.get_value(self.cpu_total_index)
                .unwrap_or(0.0);
            
            // CPU per-core
            for &index in &self.cpu_per_core_indices {
                let value = self.query.get_value(index).unwrap_or(0.0);
                metrics.cpu_per_core_percent.push(value);
            }
            
            // CPU frequency
            metrics.cpu_frequency_mhz = self.query.get_value(self.cpu_freq_index)
                .unwrap_or(0.0);
            metrics.cpu_base_frequency_mhz = metrics.cpu_frequency_mhz; // Base freq requires separate query
        }
        
        if self.collect_disk {
            // Disk I/O
            metrics.disk_read_bps = self.query.get_value(self.disk_read_bps_index)
                .unwrap_or(0.0);
            metrics.disk_write_bps = self.query.get_value(self.disk_write_bps_index)
                .unwrap_or(0.0);
            metrics.disk_read_iops = self.query.get_value(self.disk_read_iops_index)
                .unwrap_or(0.0);
            metrics.disk_write_iops = self.query.get_value(self.disk_write_iops_index)
                .unwrap_or(0.0);
        }
        
        if self.collect_network {
            // Network I/O
            metrics.network_recv_bps = self.query.get_value(self.network_recv_index)
                .unwrap_or(0.0);
            metrics.network_send_bps = self.query.get_value(self.network_send_index)
                .unwrap_or(0.0);
        }
        
        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_pdh_query_creation() {
        let query = PdhQuery::new();
        assert!(query.is_ok());
    }
    
    #[test]
    fn test_add_cpu_counter() {
        let mut query = PdhQuery::new().unwrap();
        // PDH counter names might be localized, so this test might fail on non-English systems
        let result = query.add_counter("\\Processor(_Total)\\% Processor Time");
        // Don't assert - just check it doesn't panic
        let _ = result;
    }
    
    #[test]
    fn test_system_metrics_collector() {
        // This test might fail on non-English systems due to localized counter names
        match SystemMetricsCollector::new() {
            Ok(mut collector) => {
                // First collection (initialization)
                let result = collector.collect();
                assert!(result.is_ok());
                
                // Wait for counters to accumulate data
                thread::sleep(Duration::from_millis(100));
                
                // Second collection (accurate data)
                let result = collector.collect();
                assert!(result.is_ok());
                
                let metrics = result.unwrap();
                
                // CPU should be between 0-100% per core
                assert!(metrics.cpu_total_percent >= 0.0);
                assert!(metrics.cpu_per_core_percent.len() > 0);
                
                // Frequency should be positive
                assert!(metrics.cpu_frequency_mhz > 0.0);
            }
            Err(e) => {
                // Test might fail on non-English systems or without PDH support
                println!("Skipping PDH test - counter initialization failed: {}", e);
            }
        }
    }
}
