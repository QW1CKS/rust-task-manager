pub mod windows;

// Re-export commonly used items
#[allow(unused_imports)] // Used by process_manager service
pub use windows::{CRITICAL_PROCESSES, is_critical_process};
