pub mod process_manager;
pub mod system_monitor;

// Re-export main functions
pub use process_manager::ProcessManager;
#[allow(unused_imports)] // Used internally by commands
pub use system_monitor::{SYSTEM, collect_performance_metrics, get_process_list};
