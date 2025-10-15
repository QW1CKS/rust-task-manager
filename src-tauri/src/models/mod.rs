pub mod performance;
pub mod preferences;
pub mod process;
pub mod system;

// Re-export all model types for convenient imports
pub use performance::PerformanceMetrics;
pub use preferences::{ThemeMode, UserPreferences, WindowState};
pub use process::{ProcessInfo, ProcessStatus};
pub use system::SystemInfo;
