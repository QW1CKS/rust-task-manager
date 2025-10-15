pub mod performance;
pub mod preferences;
pub mod process;
pub mod system;

// Re-export all model types for convenient imports
pub use performance::PerformanceMetrics;
#[allow(unused_imports)] // Used in Phase 6+ (User Story 6 - theme customization)
pub use preferences::{ThemeMode, UserPreferences, WindowState};
pub use process::ProcessInfo;
#[allow(unused_imports)] // Used in Phase 6+ for extended process info
pub use process::ProcessStatus;
pub use system::SystemInfo;
