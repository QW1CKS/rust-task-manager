//! Context Menu for Process Control (T204-T208)
//!
//! Right-click context menu for process table with:
//! - End Process (graceful and forceful)
//! - Set Priority submenu
//! - Go to Details
//! - Open File Location
//! - UAC shield icons for privileged operations

use windows::core::Result;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::*;

/// T204: Context menu items
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuItem {
    /// End process gracefully
    EndProcessGraceful = 1001,
    /// Force terminate process
    EndProcessForce = 1002,
    /// Set process priority to realtime
    SetPriorityRealtime = 1010,
    /// Set process priority to high
    SetPriorityHigh = 1011,
    /// Set process priority to above normal
    SetPriorityAboveNormal = 1012,
    /// Set process priority to normal
    SetPriorityNormal = 1013,
    /// Set process priority to below normal
    SetPriorityBelowNormal = 1014,
    /// Set process priority to idle
    SetPriorityIdle = 1015,
    /// Navigate to process details
    GoToDetails = 1020,
    /// Open the file location in explorer
    OpenFileLocation = 1021,
    /// Suspend process execution
    SuspendProcess = 1030,
    /// Resume suspended process
    ResumeProcess = 1031,
}

impl ContextMenuItem {
    fn id(&self) -> u16 {
        *self as u16
    }

    fn from_id(id: u16) -> Option<Self> {
        match id {
            1001 => Some(Self::EndProcessGraceful),
            1002 => Some(Self::EndProcessForce),
            1010 => Some(Self::SetPriorityRealtime),
            1011 => Some(Self::SetPriorityHigh),
            1012 => Some(Self::SetPriorityAboveNormal),
            1013 => Some(Self::SetPriorityNormal),
            1014 => Some(Self::SetPriorityBelowNormal),
            1015 => Some(Self::SetPriorityIdle),
            1020 => Some(Self::GoToDetails),
            1021 => Some(Self::OpenFileLocation),
            1030 => Some(Self::SuspendProcess),
            1031 => Some(Self::ResumeProcess),
            _ => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::EndProcessGraceful => "End Process",
            Self::EndProcessForce => "End Process (Force)",
            Self::SetPriorityRealtime => "Realtime",
            Self::SetPriorityHigh => "High",
            Self::SetPriorityAboveNormal => "Above Normal",
            Self::SetPriorityNormal => "Normal",
            Self::SetPriorityBelowNormal => "Below Normal",
            Self::SetPriorityIdle => "Idle",
            Self::GoToDetails => "Go to Details",
            Self::OpenFileLocation => "Open File Location",
            Self::SuspendProcess => "Suspend",
            Self::ResumeProcess => "Resume",
        }
    }

    /// T205: Check if item requires elevated privileges
    #[allow(dead_code)]
    fn requires_elevation(&self) -> bool {
        matches!(
            self,
            Self::EndProcessForce | Self::SetPriorityRealtime
        )
    }
}

/// T204-T208: Process context menu
pub struct ProcessContextMenu {
    menu: HMENU,
    priority_submenu: HMENU,
}

impl ProcessContextMenu {
    /// Create new context menu
    pub fn new() -> Result<Self> {
        unsafe {
            let menu = CreatePopupMenu()?;
            let priority_submenu = CreatePopupMenu()?;

            // T204: Add menu items
            let mut menu_instance = Self {
                menu,
                priority_submenu,
            };

            menu_instance.build_menu()?;
            Ok(menu_instance)
        }
    }

    /// T204-T207: Build menu structure
    fn build_menu(&mut self) -> Result<()> {
        unsafe {
            // End Process (graceful)
            AppendMenuW(
                self.menu,
                MF_STRING,
                ContextMenuItem::EndProcessGraceful.id() as usize,
                windows::core::w!("End Process"),
            )?;

            // End Process (force) - with separator before
            AppendMenuW(
                self.menu,
                MF_STRING,
                ContextMenuItem::EndProcessForce.id() as usize,
                windows::core::w!("End Process (Force)"),
            )?;

            AppendMenuW(self.menu, MF_SEPARATOR, 0, None)?;

            // T206: Priority submenu
            self.build_priority_submenu()?;
            AppendMenuW(
                self.menu,
                MF_STRING | MF_POPUP,
                self.priority_submenu.0 as usize,
                windows::core::w!("Set Priority"),
            )?;

            AppendMenuW(self.menu, MF_SEPARATOR, 0, None)?;

            // Suspend/Resume
            AppendMenuW(
                self.menu,
                MF_STRING,
                ContextMenuItem::SuspendProcess.id() as usize,
                windows::core::w!("Suspend"),
            )?;

            AppendMenuW(
                self.menu,
                MF_STRING,
                ContextMenuItem::ResumeProcess.id() as usize,
                windows::core::w!("Resume"),
            )?;

            AppendMenuW(self.menu, MF_SEPARATOR, 0, None)?;

            // T208: Go to Details
            AppendMenuW(
                self.menu,
                MF_STRING,
                ContextMenuItem::GoToDetails.id() as usize,
                windows::core::w!("Go to Details"),
            )?;

            // T208: Open File Location
            AppendMenuW(
                self.menu,
                MF_STRING,
                ContextMenuItem::OpenFileLocation.id() as usize,
                windows::core::w!("Open File Location"),
            )?;

            Ok(())
        }
    }

    /// T206: Build priority submenu
    fn build_priority_submenu(&mut self) -> Result<()> {
        unsafe {
            let priorities = [
                ContextMenuItem::SetPriorityRealtime,
                ContextMenuItem::SetPriorityHigh,
                ContextMenuItem::SetPriorityAboveNormal,
                ContextMenuItem::SetPriorityNormal,
                ContextMenuItem::SetPriorityBelowNormal,
                ContextMenuItem::SetPriorityIdle,
            ];

            for priority in priorities {
                AppendMenuW(
                    self.priority_submenu,
                    MF_STRING,
                    priority.id() as usize,
                    &windows::core::HSTRING::from(priority.label()),
                )?;
            }

            Ok(())
        }
    }

    /// T205: Enable/disable menu items based on privileges
    pub fn update_for_process(&self, _pid: u32, is_elevated: bool, can_control: bool) {
        unsafe {
            // Disable all items if can't control
            if !can_control {
                let _ = EnableMenuItem(self.menu, ContextMenuItem::EndProcessGraceful.id() as u32, MF_GRAYED);
                let _ = EnableMenuItem(self.menu, ContextMenuItem::EndProcessForce.id() as u32, MF_GRAYED);
                let _ = EnableMenuItem(self.menu, ContextMenuItem::SuspendProcess.id() as u32, MF_GRAYED);
                let _ = EnableMenuItem(self.menu, ContextMenuItem::ResumeProcess.id() as u32, MF_GRAYED);
                
                // Disable priority submenu
                for item in [
                    ContextMenuItem::SetPriorityRealtime,
                    ContextMenuItem::SetPriorityHigh,
                    ContextMenuItem::SetPriorityAboveNormal,
                    ContextMenuItem::SetPriorityNormal,
                    ContextMenuItem::SetPriorityBelowNormal,
                    ContextMenuItem::SetPriorityIdle,
                ] {
                    let _ = EnableMenuItem(self.priority_submenu, item.id() as u32, MF_GRAYED);
                }
            } else {
                // Enable all if we can control
                let _ = EnableMenuItem(self.menu, ContextMenuItem::EndProcessGraceful.id() as u32, MF_ENABLED);
                let _ = EnableMenuItem(self.menu, ContextMenuItem::EndProcessForce.id() as u32, MF_ENABLED);
                let _ = EnableMenuItem(self.menu, ContextMenuItem::SuspendProcess.id() as u32, MF_ENABLED);
                let _ = EnableMenuItem(self.menu, ContextMenuItem::ResumeProcess.id() as u32, MF_ENABLED);

                // Disable Realtime priority if not elevated
                if is_elevated {
                    let _ = EnableMenuItem(self.priority_submenu, ContextMenuItem::SetPriorityRealtime.id() as u32, MF_ENABLED);
                } else {
                    let _ = EnableMenuItem(self.priority_submenu, ContextMenuItem::SetPriorityRealtime.id() as u32, MF_GRAYED);
                }

                // Enable other priorities
                for item in [
                    ContextMenuItem::SetPriorityHigh,
                    ContextMenuItem::SetPriorityAboveNormal,
                    ContextMenuItem::SetPriorityNormal,
                    ContextMenuItem::SetPriorityBelowNormal,
                    ContextMenuItem::SetPriorityIdle,
                ] {
                    let _ = EnableMenuItem(self.priority_submenu, item.id() as u32, MF_ENABLED);
                }
            }
        }
    }

    /// T207: Show context menu at position
    pub fn show(&self, hwnd: HWND, x: i32, y: i32) -> Result<Option<ContextMenuItem>> {
        unsafe {
            let result = TrackPopupMenuEx(
                self.menu,
                (TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RETURNCMD).0 as u32,
                x,
                y,
                hwnd,
                None,
            );

            if result.as_bool() {
                Ok(ContextMenuItem::from_id(result.0 as u16))
            } else {
                Ok(None)
            }
        }
    }

    /// Show at mouse cursor position
    pub fn show_at_cursor(&self, hwnd: HWND) -> Result<Option<ContextMenuItem>> {
        unsafe {
            let mut point = POINT::default();
            GetCursorPos(&mut point)?;
            self.show(hwnd, point.x, point.y)
        }
    }
}

impl Drop for ProcessContextMenu {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyMenu(self.menu);
            let _ = DestroyMenu(self.priority_submenu);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_conversion() {
        let item = ContextMenuItem::EndProcessGraceful;
        assert_eq!(item.id(), 1001);
        assert_eq!(ContextMenuItem::from_id(1001), Some(item));
        assert_eq!(item.label(), "End Process");
    }

    #[test]
    fn test_requires_elevation() {
        assert!(ContextMenuItem::SetPriorityRealtime.requires_elevation());
        assert!(ContextMenuItem::EndProcessForce.requires_elevation());
        assert!(!ContextMenuItem::EndProcessGraceful.requires_elevation());
        assert!(!ContextMenuItem::SetPriorityNormal.requires_elevation());
    }

    #[test]
    fn test_all_menu_items_have_labels() {
        let items = [
            ContextMenuItem::EndProcessGraceful,
            ContextMenuItem::EndProcessForce,
            ContextMenuItem::SetPriorityRealtime,
            ContextMenuItem::SetPriorityHigh,
            ContextMenuItem::SetPriorityAboveNormal,
            ContextMenuItem::SetPriorityNormal,
            ContextMenuItem::SetPriorityBelowNormal,
            ContextMenuItem::SetPriorityIdle,
            ContextMenuItem::GoToDetails,
            ContextMenuItem::OpenFileLocation,
            ContextMenuItem::SuspendProcess,
            ContextMenuItem::ResumeProcess,
        ];

        for item in items {
            assert!(!item.label().is_empty());
        }
    }

    #[test]
    fn test_menu_item_roundtrip() {
        for id in 1001..=1031 {
            if let Some(item) = ContextMenuItem::from_id(id) {
                assert_eq!(item.id(), id);
            }
        }
    }
}
