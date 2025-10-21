//! Arena allocator management using bumpalo for temporary allocations

use bumpalo::Bump;

/// Arena allocator for short-lived temporary allocations in hot paths
pub struct Arena {
    bump: Bump,
}

impl Arena {
    /// Create a new arena with default capacity
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Create a new arena with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    /// Reset the arena, freeing all allocations
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get underlying bumpalo arena
    pub fn bump(&self) -> &Bump {
        &self.bump
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
