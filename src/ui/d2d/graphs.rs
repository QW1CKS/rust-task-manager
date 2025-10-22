//! Direct2D graph rendering optimizations
//!
//! Provides high-performance graph rendering using Direct2D geometries.
//! T319: Uses bumpalo arena allocator for temporary rendering data.

use crate::util::arenas::Arena;
use windows::core::*;

/// Graph rendering utilities with arena-allocated temporary data (T319)
pub struct GraphRenderer {
    /// Arena for temporary allocations during rendering (T319)
    /// Reset after each frame to eliminate per-frame allocations
    arena: Arena,
}

impl GraphRenderer {
    /// Create a new graph renderer with 64KB arena (T319)
    pub fn new() -> Result<Self> {
        Ok(Self {
            arena: Arena::with_capacity(65536), // 64KB for temp data
        })
    }
    
    /// Reset arena after frame completion (T319)
    /// Should be called after each render() to free temporary allocations
    pub fn reset_arena(&mut self) {
        self.arena.reset();
    }
    
    /// Get arena reference for temporary allocations (T319)
    pub fn arena(&self) -> &Arena {
        &self.arena
    }
}

impl Default for GraphRenderer {
    fn default() -> Self {
        Self {
            arena: Arena::with_capacity(65536),
        }
    }
}
