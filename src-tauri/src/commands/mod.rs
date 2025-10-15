/// Tauri command module exports
///
/// This module contains all Tauri commands exposed to the frontend.
/// Commands are grouped by functionality (system info, performance, processes, etc.)
pub mod performance;
pub mod system_info;

// Re-export command functions for convenience
pub use performance::get_performance_data;
pub use system_info::get_system_info;
