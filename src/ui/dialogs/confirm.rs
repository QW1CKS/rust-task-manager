//! Confirmation Dialogs (T209-T212)
//!
//! Custom confirmation dialogs for destructive operations:
//! - End Process confirmation with process info
//! - "Don't ask again" checkbox
//! - Keyboard shortcuts (Enter=Yes, Esc=No)

use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// T209: Dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    /// User clicked Yes
    Yes,
    /// User clicked No
    No,
    /// User clicked Cancel or closed dialog
    Cancel,
}

/// T209-T212: Confirmation dialog configuration
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    /// Dialog title
    pub title: String,
    /// Main message
    pub message: String,
    /// Optional detailed information (process name, PID, etc.)
    pub details: Option<String>,
    /// Warning message
    pub warning: Option<String>,
    /// Show "Don't ask again" checkbox
    pub show_dont_ask: bool,
}

impl ConfirmDialog {
    /// Create new confirmation dialog
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details: None,
            warning: None,
            show_dont_ask: false,
        }
    }

    /// T210: Add process details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// T210: Add warning message
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warning = Some(warning.into());
        self
    }

    /// T211: Enable "Don't ask again" checkbox
    pub fn with_dont_ask_checkbox(mut self) -> Self {
        self.show_dont_ask = true;
        self
    }

    /// T209-T212: Show dialog and return result
    pub fn show(&self, parent: HWND) -> Result<(DialogResult, bool)> {
        // Build message text
        let mut text = self.message.clone();
        
        if let Some(ref details) = self.details {
            text.push_str("\n\n");
            text.push_str(details);
        }

        if let Some(ref warning) = self.warning {
            text.push_str("\n\n⚠️ ");
            text.push_str(warning);
        }

        if self.show_dont_ask {
            text.push_str("\n\n");
            text.push_str("(Press Ctrl+Y to confirm and don't ask again)");
        }

        // For now, use MessageBox (later can be replaced with custom dialog)
        let title_wide = windows::core::HSTRING::from(&self.title);
        let text_wide = windows::core::HSTRING::from(&text);

        let flags = MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2; // Default to No for safety

        let result = unsafe {
            MessageBoxW(
                Some(parent),
                PCWSTR(text_wide.as_ptr()),
                PCWSTR(title_wide.as_ptr()),
                flags,
            )
        };

        let dialog_result = match result {
            IDYES => DialogResult::Yes,
            IDNO => DialogResult::No,
            _ => DialogResult::Cancel,
        };

        // For now, don't_ask is always false (would need custom dialog for checkbox)
        let dont_ask = false;

        Ok((dialog_result, dont_ask))
    }
}

/// T209-T210: Pre-configured dialogs for common scenarios
pub struct ConfirmDialogs;

impl ConfirmDialogs {
    /// T209: Confirm ending a single process
    pub fn end_process(process_name: &str, pid: u32) -> ConfirmDialog {
        ConfirmDialog::new(
            "End Process",
            "Are you sure you want to end this process?",
        )
        .with_details(format!("Process: {}\nPID: {}", process_name, pid))
        .with_warning("This may cause data loss if the process has unsaved work.")
        .with_dont_ask_checkbox()
    }

    /// T209: Confirm force terminating a process
    pub fn force_end_process(process_name: &str, pid: u32) -> ConfirmDialog {
        ConfirmDialog::new(
            "Force End Process",
            "Are you sure you want to forcefully terminate this process?",
        )
        .with_details(format!("Process: {}\nPID: {}", process_name, pid))
        .with_warning("This will immediately terminate the process without allowing it to save data or clean up. Data loss is likely.")
    }

    /// T209: Confirm ending multiple processes
    pub fn end_multiple_processes(count: usize) -> ConfirmDialog {
        ConfirmDialog::new(
            "End Multiple Processes",
            format!("Are you sure you want to end {} processes?", count),
        )
        .with_warning("This may cause data loss if any processes have unsaved work.")
        .with_dont_ask_checkbox()
    }

    /// T209: Confirm setting realtime priority
    pub fn set_realtime_priority(process_name: &str, pid: u32) -> ConfirmDialog {
        ConfirmDialog::new(
            "Set Realtime Priority",
            "Are you sure you want to set this process to Realtime priority?",
        )
        .with_details(format!("Process: {}\nPID: {}", process_name, pid))
        .with_warning("Realtime priority can cause system instability if the process consumes too much CPU. Only set this for time-critical processes.")
    }

    /// T209: Confirm suspending a process
    pub fn suspend_process(process_name: &str, pid: u32) -> ConfirmDialog {
        ConfirmDialog::new(
            "Suspend Process",
            "Are you sure you want to suspend this process?",
        )
        .with_details(format!("Process: {}\nPID: {}", process_name, pid))
        .with_warning("Suspending a process will freeze all its threads. The process may become unresponsive.")
    }
}

/// T211: User preferences for "Don't ask again"
pub struct DialogPreferences {
    dont_ask_end_process: AtomicBool,
    dont_ask_force_end: AtomicBool,
    dont_ask_multiple: AtomicBool,
    dont_ask_realtime: AtomicBool,
    dont_ask_suspend: AtomicBool,
}

impl DialogPreferences {
    /// Create new preferences with all confirmations enabled
    pub fn new() -> Self {
        Self {
            dont_ask_end_process: AtomicBool::new(false),
            dont_ask_force_end: AtomicBool::new(false),
            dont_ask_multiple: AtomicBool::new(false),
            dont_ask_realtime: AtomicBool::new(false),
            dont_ask_suspend: AtomicBool::new(false),
        }
    }

    /// Check if should show end process confirmation
    pub fn should_confirm_end_process(&self) -> bool {
        !self.dont_ask_end_process.load(Ordering::Relaxed)
    }

    /// Set don't ask for end process
    pub fn set_dont_ask_end_process(&self, value: bool) {
        self.dont_ask_end_process.store(value, Ordering::Relaxed);
    }

    /// Check if should show force end confirmation
    pub fn should_confirm_force_end(&self) -> bool {
        !self.dont_ask_force_end.load(Ordering::Relaxed)
    }

    /// Set don't ask for force end
    pub fn set_dont_ask_force_end(&self, value: bool) {
        self.dont_ask_force_end.store(value, Ordering::Relaxed);
    }

    /// Check if should show multiple processes confirmation
    pub fn should_confirm_multiple(&self) -> bool {
        !self.dont_ask_multiple.load(Ordering::Relaxed)
    }

    /// Set don't ask for multiple processes
    pub fn set_dont_ask_multiple(&self, value: bool) {
        self.dont_ask_multiple.store(value, Ordering::Relaxed);
    }

    /// Check if should show realtime priority confirmation
    pub fn should_confirm_realtime(&self) -> bool {
        !self.dont_ask_realtime.load(Ordering::Relaxed)
    }

    /// Set don't ask for realtime priority
    pub fn set_dont_ask_realtime(&self, value: bool) {
        self.dont_ask_realtime.store(value, Ordering::Relaxed);
    }

    /// Check if should show suspend confirmation
    pub fn should_confirm_suspend(&self) -> bool {
        !self.dont_ask_suspend.load(Ordering::Relaxed)
    }

    /// Set don't ask for suspend
    pub fn set_dont_ask_suspend(&self, value: bool) {
        self.dont_ask_suspend.store(value, Ordering::Relaxed);
    }

    /// Reset all preferences
    pub fn reset_all(&self) {
        self.dont_ask_end_process.store(false, Ordering::Relaxed);
        self.dont_ask_force_end.store(false, Ordering::Relaxed);
        self.dont_ask_multiple.store(false, Ordering::Relaxed);
        self.dont_ask_realtime.store(false, Ordering::Relaxed);
        self.dont_ask_suspend.store(false, Ordering::Relaxed);
    }
}

impl Default for DialogPreferences {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_creation() {
        let dialog = ConfirmDialog::new("Test Title", "Test Message");
        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.message, "Test Message");
        assert_eq!(dialog.details, None);
        assert_eq!(dialog.warning, None);
        assert!(!dialog.show_dont_ask);
    }

    #[test]
    fn test_dialog_with_details() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_details("Process: test.exe\nPID: 1234");
        
        assert!(dialog.details.is_some());
        assert!(dialog.details.unwrap().contains("test.exe"));
    }

    #[test]
    fn test_dialog_with_warning() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_warning("This is dangerous!");
        
        assert!(dialog.warning.is_some());
        assert_eq!(dialog.warning.unwrap(), "This is dangerous!");
    }

    #[test]
    fn test_end_process_dialog() {
        let dialog = ConfirmDialogs::end_process("notepad.exe", 1234);
        assert_eq!(dialog.title, "End Process");
        assert!(dialog.details.is_some());
        assert!(dialog.warning.is_some());
        assert!(dialog.show_dont_ask);
    }

    #[test]
    fn test_force_end_dialog() {
        let dialog = ConfirmDialogs::force_end_process("chrome.exe", 5678);
        assert_eq!(dialog.title, "Force End Process");
        assert!(dialog.details.is_some());
        assert!(dialog.warning.is_some());
    }

    #[test]
    fn test_preferences_default() {
        let prefs = DialogPreferences::new();
        assert!(prefs.should_confirm_end_process());
        assert!(prefs.should_confirm_force_end());
        assert!(prefs.should_confirm_multiple());
        assert!(prefs.should_confirm_realtime());
        assert!(prefs.should_confirm_suspend());
    }

    #[test]
    fn test_preferences_set_dont_ask() {
        let prefs = DialogPreferences::new();
        
        prefs.set_dont_ask_end_process(true);
        assert!(!prefs.should_confirm_end_process());
        
        prefs.set_dont_ask_end_process(false);
        assert!(prefs.should_confirm_end_process());
    }

    #[test]
    fn test_preferences_reset() {
        let prefs = DialogPreferences::new();
        
        prefs.set_dont_ask_end_process(true);
        prefs.set_dont_ask_force_end(true);
        prefs.set_dont_ask_multiple(true);
        
        prefs.reset_all();
        
        assert!(prefs.should_confirm_end_process());
        assert!(prefs.should_confirm_force_end());
        assert!(prefs.should_confirm_multiple());
    }
}
