pub mod system_monitor;

// Re-export main functions
pub use system_monitor::{SYSTEM, collect_performance_metrics, get_process_list};
