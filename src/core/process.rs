//! Process data structures and store (Structure of Arrays layout)

use static_assertions::const_assert;

/// Maximum number of processes supported (constitutional requirement)
pub const MAX_PROCESSES: usize = 2048;

// Compile-time assertion to prevent accidental capacity reduction
const_assert!(MAX_PROCESSES == 2048);

/// Process store using Structure of Arrays (SoA) for cache efficiency
pub struct ProcessStore {
    /// Current number of processes
    count: usize,

    /// Process IDs (fixed capacity)
    #[allow(dead_code)]
    pids: Box<[u32; MAX_PROCESSES]>,

    /// Process names (fixed capacity)
    #[allow(dead_code)]
    names: Box<[String; MAX_PROCESSES]>,
    // Memory layout: 2048 processes × ~200 bytes/process = ~410KB for SoA storage
    // (well within <15MB idle budget per constitution)
}

impl ProcessStore {
    /// Create a new empty process store
    pub fn new() -> Self {
        Self {
            count: 0,
            pids: Box::new([0; MAX_PROCESSES]),
            names: Box::new(std::array::from_fn(|_| String::new())),
        }
    }

    /// Get current process count
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Default for ProcessStore {
    fn default() -> Self {
        Self::new()
    }
}
