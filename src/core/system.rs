//! Historical data storage using circular buffer

use std::time::Instant;

/// Historical data point
#[derive(Debug, Clone, Copy)]
pub struct DataPoint<T: Copy> {
    /// Timestamp when data was collected
    pub timestamp: Instant,
    /// Value at this timestamp
    pub value: T,
}

/// Circular buffer for time-series data
///
/// # Zero Allocations
///
/// Fixed-size buffer allocated once during construction. push() performs no allocations.
///
/// # Performance
///
/// - push(): O(1)
/// - get_range(): O(n) where n is requested range size
/// - Target: <50μs for push/query operations
pub struct CircularBuffer<T: Copy> {
    /// Fixed-size buffer
    buffer: Vec<DataPoint<T>>,
    /// Current write position (wraps around)
    head: usize,
    /// Number of valid samples (up to capacity)
    count: usize,
}

impl<T: Copy> CircularBuffer<T> {
    /// Create a new circular buffer with fixed capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of samples to store
    ///
    /// # Memory
    ///
    /// For SystemMetrics at 1Hz:
    /// - 3600 samples (1 hour) = ~288KB
    /// - 86400 samples (24 hours) = ~6.9MB
    pub fn new(capacity: usize) -> Self
    where
        T: Default,
    {
        Self {
            buffer: vec![
                DataPoint {
                    timestamp: Instant::now(),
                    value: T::default()
                };
                capacity
            ],
            head: 0,
            count: 0,
        }
    }

    /// Push a new value into the circular buffer
    ///
    /// If buffer is full, oldest value is automatically evicted.
    ///
    /// # Performance
    ///
    /// O(1) - no allocations
    pub fn push(&mut self, value: T) {
        self.buffer[self.head] = DataPoint {
            timestamp: Instant::now(),
            value,
        };

        self.head = (self.head + 1) % self.buffer.len();
        self.count = (self.count + 1).min(self.buffer.len());
    }

    /// Get data points within a time window
    ///
    /// # Arguments
    ///
    /// * `duration_secs` - How many seconds of history to retrieve
    ///
    /// # Returns
    ///
    /// Vec of DataPoint<T> within the time window, sorted oldest to newest
    ///
    /// # Performance
    ///
    /// O(n) where n is the number of points in range
    pub fn get_range(&self, duration_secs: u64) -> Vec<DataPoint<T>> {
        if self.count == 0 {
            return Vec::new();
        }

        let now = Instant::now();
        let threshold = std::time::Duration::from_secs(duration_secs);

        let mut result = Vec::with_capacity(self.count);

        // Start from oldest entry (head if buffer is full, or 0 if not)
        let start = if self.count < self.buffer.len() {
            0
        } else {
            self.head
        };

        for i in 0..self.count {
            let index = (start + i) % self.buffer.len();
            let point = self.buffer[index];

            if now.duration_since(point.timestamp) <= threshold {
                result.push(point);
            }
        }

        result
    }

    /// Get all data points
    pub fn get_all(&self) -> Vec<DataPoint<T>> {
        if self.count == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.count);

        let start = if self.count < self.buffer.len() {
            0
        } else {
            self.head
        };

        for i in 0..self.count {
            let index = (start + i) % self.buffer.len();
            result.push(self.buffer[index]);
        }

        result
    }

    /// Get the most recent value
    pub fn latest(&self) -> Option<DataPoint<T>> {
        if self.count == 0 {
            return None;
        }

        let latest_index = if self.head == 0 {
            self.buffer.len() - 1
        } else {
            self.head - 1
        };

        Some(self.buffer[latest_index])
    }

    /// Get current count of stored samples
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
    }
}

/// Predefined history duration: 1 minute (60 samples at 1Hz)
pub const HISTORY_1_MIN: usize = 60;
/// Predefined history duration: 5 minutes (300 samples at 1Hz)
pub const HISTORY_5_MIN: usize = 300;
/// Predefined history duration: 1 hour (3600 samples at 1Hz)
pub const HISTORY_1_HOUR: usize = 3600;
/// Predefined history duration: 24 hours (86400 samples at 1Hz)
pub const HISTORY_24_HOUR: usize = 86400;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_circular_buffer_create() {
        let buffer: CircularBuffer<f32> = CircularBuffer::new(100);
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 100);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_circular_buffer_push() {
        let mut buffer = CircularBuffer::new(10);
        
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);

        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_circular_buffer_wraparound() {
        let mut buffer = CircularBuffer::new(3);
        
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);
        buffer.push(4.0); // Should wrap around and evict 1.0

        assert_eq!(buffer.len(), 3);
        
        let all = buffer.get_all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].value, 2.0);
        assert_eq!(all[1].value, 3.0);
        assert_eq!(all[2].value, 4.0);
    }

    #[test]
    fn test_circular_buffer_latest() {
        let mut buffer = CircularBuffer::new(10);
        
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);

        let latest = buffer.latest();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().value, 3.0);
    }

    #[test]
    fn test_circular_buffer_get_range() {
        let mut buffer = CircularBuffer::new(100);
        
        for i in 0..10 {
            buffer.push(i as f32);
            thread::sleep(Duration::from_millis(10));
        }

        // Get last 1 second of data (should get most/all of it)
        let range = buffer.get_range(1);
        assert!(!range.is_empty());
    }
}
