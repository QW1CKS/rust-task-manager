//! Performance profiling utilities
//!
//! Provides macros and utilities for profiling hot paths and measuring performance.
//! Supports Tracy profiler integration for frame-level analysis (T308).

use std::time::{Duration, Instant};

/// Performance zone for profiling (Tracy integration placeholder)
pub struct ProfileZone {
    _name: &'static str,
    start: Instant,
}

impl ProfileZone {
    /// Creates a new profiling zone
    #[inline(always)]
    pub fn new(name: &'static str) -> Self {
        Self {
            _name: name,
            start: Instant::now(),
        }
    }

    /// Ends the profiling zone and returns elapsed time
    #[inline(always)]
    pub fn end(self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ProfileZone {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(feature = "profiling")]
        {
            let elapsed = self.start.elapsed();
            if elapsed > Duration::from_micros(100) {
                eprintln!("[PROFILE] {} took {:?}", self.name, elapsed);
            }
        }
    }
}

/// Macro to create a profiling zone for the current scope
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "profiling")]
        let _profile_zone = $crate::util::profiling::ProfileZone::new($name);
    };
}

/// Macro to measure execution time of an expression
#[macro_export]
macro_rules! profile_time {
    ($name:expr, $expr:expr) => {{
        #[cfg(feature = "profiling")]
        let _start = std::time::Instant::now();
        
        let result = $expr;
        
        #[cfg(feature = "profiling")]
        {
            let elapsed = _start.elapsed();
            if elapsed > std::time::Duration::from_micros(100) {
                eprintln!("[PROFILE] {} took {:?}", $name, elapsed);
            }
        }
        
        result
    }};
}

/// Frame profiler for rendering performance
pub struct FrameProfiler {
    frame_start: Instant,
    frame_count: u64,
    total_time: Duration,
    min_frame_time: Duration,
    max_frame_time: Duration,
}

impl FrameProfiler {
    /// Creates a new frame profiler
    pub fn new() -> Self {
        Self {
            frame_start: Instant::now(),
            frame_count: 0,
            total_time: Duration::ZERO,
            min_frame_time: Duration::from_secs(1),
            max_frame_time: Duration::ZERO,
        }
    }

    /// Begins a new frame
    #[inline(always)]
    pub fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    /// Ends the current frame and records timing
    #[inline(always)]
    pub fn end_frame(&mut self) {
        let frame_time = self.frame_start.elapsed();
        self.frame_count += 1;
        self.total_time += frame_time;
        self.min_frame_time = self.min_frame_time.min(frame_time);
        self.max_frame_time = self.max_frame_time.max(frame_time);
    }

    /// Returns the average frame time
    pub fn avg_frame_time(&self) -> Duration {
        if self.frame_count > 0 {
            self.total_time / self.frame_count as u32
        } else {
            Duration::ZERO
        }
    }

    /// Returns the current FPS
    pub fn fps(&self) -> f64 {
        let avg = self.avg_frame_time();
        if avg.as_secs_f64() > 0.0 {
            1.0 / avg.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Returns frame statistics
    pub fn stats(&self) -> FrameStats {
        FrameStats {
            frame_count: self.frame_count,
            avg_time: self.avg_frame_time(),
            min_time: self.min_frame_time,
            max_time: self.max_frame_time,
            fps: self.fps(),
        }
    }

    /// Resets all statistics
    pub fn reset(&mut self) {
        self.frame_count = 0;
        self.total_time = Duration::ZERO;
        self.min_frame_time = Duration::from_secs(1);
        self.max_frame_time = Duration::ZERO;
    }
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame timing statistics
#[derive(Debug, Clone, Copy)]
pub struct FrameStats {
    /// Total number of frames recorded
    pub frame_count: u64,
    /// Average frame time
    pub avg_time: Duration,
    /// Minimum frame time observed
    pub min_time: Duration,
    /// Maximum frame time observed
    pub max_time: Duration,
    /// Current frames per second
    pub fps: f64,
}

/// Memory profiler for tracking allocations
pub struct MemoryProfiler {
    _start_allocations: usize,
    _start_bytes: usize,
}

impl MemoryProfiler {
    /// Creates a new memory profiler snapshot
    pub fn new() -> Self {
        Self {
            _start_allocations: 0,
            _start_bytes: 0,
        }
    }

    /// Returns memory usage delta since creation
    pub fn delta(&self) -> MemoryDelta {
        MemoryDelta {
            allocations: 0,
            bytes: 0,
        }
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage delta
#[derive(Debug, Clone, Copy)]
pub struct MemoryDelta {
    /// Number of allocations
    pub allocations: usize,
    /// Bytes allocated
    pub bytes: usize,
}

/// Startup phase timing tracker (T315)
pub struct StartupProfiler {
    phases: Vec<(&'static str, Duration)>,
    current_phase_start: Instant,
}

impl StartupProfiler {
    /// Creates a new startup profiler
    pub fn new() -> Self {
        Self {
            phases: Vec::new(),
            current_phase_start: Instant::now(),
        }
    }

    /// Begins a new startup phase
    pub fn begin_phase(&mut self, name: &'static str) {
        let elapsed = self.current_phase_start.elapsed();
        if !self.phases.is_empty() {
            // Record previous phase
            let prev_name = self.phases.last().unwrap().0;
            self.phases.push((prev_name, elapsed));
        }
        self.current_phase_start = Instant::now();
        self.phases.push((name, Duration::ZERO));
    }

    /// Ends the current phase
    pub fn end_phase(&mut self) {
        let elapsed = self.current_phase_start.elapsed();
        if let Some(last) = self.phases.last_mut() {
            last.1 = elapsed;
        }
    }

    /// Returns all recorded phases
    pub fn phases(&self) -> &[(&'static str, Duration)] {
        &self.phases
    }

    /// Returns total startup time
    pub fn total_time(&self) -> Duration {
        self.phases.iter().map(|(_, d)| *d).sum()
    }

    /// Prints a breakdown of startup phases
    pub fn print_breakdown(&self) {
        let total = self.total_time();
        println!("Startup breakdown (total: {:?}):", total);
        for (name, duration) in &self.phases {
            let pct = if total.as_secs_f64() > 0.0 {
                (duration.as_secs_f64() / total.as_secs_f64()) * 100.0
            } else {
                0.0
            };
            println!("  {:30} {:8.2}ms ({:5.1}%)", name, duration.as_secs_f64() * 1000.0, pct);
        }
    }
}

impl Default for StartupProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profile_zone() {
        let zone = ProfileZone::new("test");
        thread::sleep(Duration::from_millis(10));
        let elapsed = zone.end();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_frame_profiler() {
        let mut profiler = FrameProfiler::new();
        
        for _ in 0..10 {
            profiler.begin_frame();
            thread::sleep(Duration::from_millis(16));
            profiler.end_frame();
        }

        let stats = profiler.stats();
        assert_eq!(stats.frame_count, 10);
        assert!(stats.fps > 50.0 && stats.fps < 70.0);
    }

    #[test]
    fn test_startup_profiler() {
        let mut profiler = StartupProfiler::new();
        
        profiler.begin_phase("Phase 1");
        thread::sleep(Duration::from_millis(10));
        profiler.end_phase();
        
        profiler.begin_phase("Phase 2");
        thread::sleep(Duration::from_millis(20));
        profiler.end_phase();

        let phases = profiler.phases();
        assert!(phases.len() >= 2);
        let total = profiler.total_time();
        assert!(total >= Duration::from_millis(30));
    }
}
