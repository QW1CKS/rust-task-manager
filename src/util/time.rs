//! High-resolution timing utilities using QueryPerformanceCounter

use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

/// High-resolution timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Get current timestamp
    pub fn now() -> Self {
        let mut counter = 0i64;
        // SAFETY: QueryPerformanceCounter is always safe to call on Windows
        unsafe {
            QueryPerformanceCounter(&mut counter).expect("QueryPerformanceCounter failed");
        }
        Self(counter)
    }

    /// Get elapsed time since this timestamp in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        let now = Self::now();
        let mut frequency = 0i64;
        // SAFETY: QueryPerformanceFrequency is always safe to call on Windows
        unsafe {
            QueryPerformanceFrequency(&mut frequency).expect("QueryPerformanceFrequency failed");
        }

        let elapsed_counts = now.0 - self.0;
        (elapsed_counts as f64 / frequency as f64) * 1000.0
    }
}
