//! Core metrics abstraction and calculations

use std::time::Instant;

/// Type of metric being measured
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// CPU usage percentage
    Cpu,
    /// Memory usage in bytes
    Memory,
    /// Disk I/O bytes per second
    Disk,
    /// Network bytes per second
    Network,
    /// GPU usage/memory
    Gpu,
}

/// System-wide metrics snapshot
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    /// Timestamp when metrics were collected
    pub timestamp: Instant,
    /// Total CPU usage across all cores (0.0-100.0)
    pub cpu_total: f32,
    /// Per-core CPU usage
    pub cpu_cores: Vec<f32>,
    /// Total physical memory (bytes)
    pub memory_total: u64,
    /// Available physical memory (bytes)
    pub memory_available: u64,
    /// Memory load percentage (0-100)
    pub memory_load_percent: u32,
    /// Disk read bytes per second
    pub disk_read_bps: u64,
    /// Disk write bytes per second
    pub disk_write_bps: u64,
    /// Network receive bytes per second
    pub network_receive_bps: u64,
    /// Network transmit bytes per second
    pub network_transmit_bps: u64,
}

impl SystemMetrics {
    /// Create a new empty metrics snapshot
    pub fn new() -> Self {
        Self {
            timestamp: Instant::now(),
            cpu_total: 0.0,
            cpu_cores: Vec::new(),
            memory_total: 0,
            memory_available: 0,
            memory_load_percent: 0,
            disk_read_bps: 0,
            disk_write_bps: 0,
            network_receive_bps: 0,
            network_transmit_bps: 0,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate CPU percentage from time deltas
///
/// # Arguments
///
/// * `delta_user` - User-mode CPU time delta (100ns units)
/// * `delta_kernel` - Kernel-mode CPU time delta (100ns units)
/// * `elapsed_ms` - Elapsed time in milliseconds
/// * `num_cpus` - Number of CPU cores
///
/// # Returns
///
/// CPU percentage (0.0-100.0 * num_cpus, so 200.0 on dual-core at 100%)
pub fn calculate_cpu_percentage(
    delta_user: u64,
    delta_kernel: u64,
    elapsed_ms: u64,
    num_cpus: usize,
) -> f32 {
    if elapsed_ms == 0 {
        return 0.0;
    }

    // Convert 100ns units to milliseconds
    let delta_total_ms = (delta_user + delta_kernel) / 10_000;
    
    // CPU% = (cpu_time / elapsed_time) * 100 * num_cpus
    let percentage = (delta_total_ms as f32 / elapsed_ms as f32) * 100.0 * num_cpus as f32;
    
    // Clamp to reasonable range
    percentage.min(100.0 * num_cpus as f32).max(0.0)
}

/// Calculate rate from cumulative counters
///
/// # Arguments
///
/// * `current` - Current counter value
/// * `previous` - Previous counter value
/// * `elapsed_ms` - Elapsed time in milliseconds
///
/// # Returns
///
/// Rate per second (e.g., bytes per second)
pub fn calculate_rate(current: u64, previous: u64, elapsed_ms: u64) -> u64 {
    if elapsed_ms == 0 {
        return 0;
    }

    let delta = current.saturating_sub(previous);
    // Convert to per-second rate
    (delta * 1000) / elapsed_ms
}

/// Metric aggregation functions
/// 
/// # Performance (T327)
/// 
/// Uses AVX2 SIMD instructions when available for min/max/sum operations.
/// Falls back to scalar implementation if AVX2 not supported.
pub struct MetricAggregation {
    values: Vec<f32>,
}

impl MetricAggregation {
    /// Create new aggregation from values
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    /// Calculate minimum value
    /// 
    /// # Performance (T327)
    /// 
    /// Uses AVX2 for vectorized min when len >= 8 and feature available
    pub fn min(&self) -> Option<f32> {
        if self.values.is_empty() {
            return None;
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && self.values.len() >= 8 {
                return Some(unsafe { simd_min(&self.values) });
            }
        }
        
        self.values.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Calculate maximum value
    /// 
    /// # Performance (T327)
    /// 
    /// Uses AVX2 for vectorized max when len >= 8 and feature available
    pub fn max(&self) -> Option<f32> {
        if self.values.is_empty() {
            return None;
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && self.values.len() >= 8 {
                return Some(unsafe { simd_max(&self.values) });
            }
        }
        
        self.values.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    /// Calculate average value
    /// 
    /// # Performance (T327)
    /// 
    /// Uses AVX2 for vectorized sum when len >= 8 and feature available
    pub fn avg(&self) -> Option<f32> {
        if self.values.is_empty() {
            return None;
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && self.values.len() >= 8 {
                let sum = unsafe { simd_sum(&self.values) };
                return Some(sum / self.values.len() as f32);
            }
        }
        
        Some(self.values.iter().sum::<f32>() / self.values.len() as f32)
    }

    /// Calculate 95th percentile
    pub fn p95(&self) -> Option<f32> {
        if self.values.is_empty() {
            return None;
        }

        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((sorted.len() as f32) * 0.95) as usize;
        Some(sorted[index.min(sorted.len() - 1)])
    }
}

/// SIMD-accelerated metric aggregation (T327)
/// 
/// Uses AVX2 instructions for 8x f32 parallel operations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn simd_min(values: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    // SAFETY: Function has target_feature(avx2), so AVX2 intrinsics are safe
    unsafe {
        let mut min_vec = _mm256_set1_ps(f32::MAX);
        let chunks = values.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for chunk in chunks {
            let vec = _mm256_loadu_ps(chunk.as_ptr());
            min_vec = _mm256_min_ps(min_vec, vec);
        }
        
        // Horizontal min across 8 lanes
        let mut result = [0.0f32; 8];
        _mm256_storeu_ps(result.as_mut_ptr(), min_vec);
        let mut min_val = result[0];
        for &val in &result[1..] {
            min_val = min_val.min(val);
        }
        
        // Process remainder
        for &val in remainder {
            min_val = min_val.min(val);
        }
        
        min_val
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn simd_max(values: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    // SAFETY: Function has target_feature(avx2), so AVX2 intrinsics are safe
    unsafe {
        let mut max_vec = _mm256_set1_ps(f32::MIN);
        let chunks = values.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for chunk in chunks {
            let vec = _mm256_loadu_ps(chunk.as_ptr());
            max_vec = _mm256_max_ps(max_vec, vec);
        }
        
        // Horizontal max across 8 lanes
        let mut result = [0.0f32; 8];
        _mm256_storeu_ps(result.as_mut_ptr(), max_vec);
        let mut max_val = result[0];
        for &val in &result[1..] {
            max_val = max_val.max(val);
        }
        
        // Process remainder
        for &val in remainder {
            max_val = max_val.max(val);
        }
        
        max_val
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn simd_sum(values: &[f32]) -> f32 {
    use std::arch::x86_64::*;
    
    // SAFETY: Function has target_feature(avx2), so AVX2 intrinsics are safe
    unsafe {
        let mut sum_vec = _mm256_setzero_ps();
        let chunks = values.chunks_exact(8);
        let remainder = chunks.remainder();
        
        for chunk in chunks {
            let vec = _mm256_loadu_ps(chunk.as_ptr());
            sum_vec = _mm256_add_ps(sum_vec, vec);
        }
        
        // Horizontal sum across 8 lanes
        let mut result = [0.0f32; 8];
        _mm256_storeu_ps(result.as_mut_ptr(), sum_vec);
        let mut sum = 0.0f32;
        for &val in &result {
            sum += val;
        }
        
        // Process remainder
        for &val in remainder {
            sum += val;
        }
        
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cpu_percentage() {
        // 1 second of CPU time over 1 second elapsed on 1 CPU = 100%
        let delta_user = 10_000_000; // 1 second in 100ns units
        let delta_kernel = 0;
        let elapsed_ms = 1000;
        let num_cpus = 1;

        let percentage = calculate_cpu_percentage(delta_user, delta_kernel, elapsed_ms, num_cpus);
        assert!((percentage - 100.0).abs() < 1.0, "Should be ~100%");
    }

    #[test]
    fn test_calculate_rate() {
        // 1000 bytes over 1 second = 1000 bytes/sec
        let current = 2000;
        let previous = 1000;
        let elapsed_ms = 1000;

        let rate = calculate_rate(current, previous, elapsed_ms);
        assert_eq!(rate, 1000);
    }

    #[test]
    fn test_metric_aggregation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let agg = MetricAggregation::new(values);

        assert_eq!(agg.min(), Some(1.0));
        assert_eq!(agg.max(), Some(10.0));
        assert_eq!(agg.avg(), Some(5.5));
        
        // P95 of 10 values: 0.95 * 10 = 9.5, rounds to index 9 = value 10.0
        let p95 = agg.p95().unwrap();
        assert!((p95 - 10.0).abs() < 0.1, "P95 should be ~10.0, got {}", p95);
    }
}
