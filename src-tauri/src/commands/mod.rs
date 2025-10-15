/// Tauri command module exports
///
/// This module contains all Tauri commands exposed to the frontend.
/// Commands are grouped by functionality (system info, performance, processes, etc.)
pub mod performance;
pub mod process_ops;
pub mod processes;
pub mod system_info;

// Re-export command functions for convenience
pub use performance::get_performance_data;
pub use process_ops::kill_process;
pub use processes::get_processes;
pub use system_info::get_system_info;
