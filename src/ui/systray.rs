// System tray integration (T457-T462)
//
// Implements notification area (system tray) icon with menu and tooltips.
// Uses Shell_NotifyIconW for tray icon management.

use windows::Win32::Foundation::{HWND, LPARAM, POINT};
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, TrackPopupMenu, SetForegroundWindow,
    AppendMenuW, DestroyMenu, GetCursorPos, LoadIconW, HMENU,
    MF_STRING, MF_SEPARATOR, TPM_LEFTALIGN, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON,
    WM_APP, WM_LBUTTONDBLCLK, WM_RBUTTONUP, IDI_APPLICATION,
};

/// System tray icon manager (T457)
pub struct SystemTray {
    hwnd: HWND,
    icon_data: NOTIFYICONDATAW,
    context_menu: HMENU,
}

// Custom message IDs for tray menu
const WM_TRAYICON: u32 = WM_APP + 1;
const ID_TRAY_SHOW: usize = 1001;
const ID_TRAY_HIDE: usize = 1002;
const ID_TRAY_EXIT: usize = 1003;

impl SystemTray {
    /// Create new system tray icon (T458)
    pub fn new(hwnd: HWND) -> Result<Self, String> {
        unsafe {
            // Load icon (using system icon for now, can be replaced with custom icon)
            let hicon = LoadIconW(None, IDI_APPLICATION)
                .map_err(|e| format!("Failed to load icon: {:?}", e))?;

            // Initialize NOTIFYICONDATAW
            let mut icon_data = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: hicon,
                ..Default::default()
            };

            // Set tooltip (T461)
            let tooltip = "Task Manager";
            let tooltip_wide: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
            icon_data.szTip[..tooltip_wide.len().min(128)].copy_from_slice(&tooltip_wide[..tooltip_wide.len().min(128)]);

            // Add icon to system tray
            let result = Shell_NotifyIconW(NIM_ADD, &icon_data);
            if !result.as_bool() {
                return Err("Failed to add tray icon".to_string());
            }

            // Set version for better behavior
            // Note: uVersion field may not be available in all windows-rs versions
            // Skipping NIM_SETVERSION for compatibility

            // Create context menu (T459)
            let menu = CreatePopupMenu()
                .map_err(|e| format!("Failed to create menu: {:?}", e))?;

            // Add menu items
            let show_text: Vec<u16> = "Show\0".encode_utf16().collect();
            AppendMenuW(menu, MF_STRING, ID_TRAY_SHOW, PCWSTR(show_text.as_ptr()))
                .map_err(|e| format!("Failed to add Show: {:?}", e))?;

            let hide_text: Vec<u16> = "Hide\0".encode_utf16().collect();
            AppendMenuW(menu, MF_STRING, ID_TRAY_HIDE, PCWSTR(hide_text.as_ptr()))
                .map_err(|e| format!("Failed to add Hide: {:?}", e))?;

            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())
                .map_err(|e| format!("Failed to add separator: {:?}", e))?;

            let exit_text: Vec<u16> = "Exit\0".encode_utf16().collect();
            AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, PCWSTR(exit_text.as_ptr()))
                .map_err(|e| format!("Failed to add Exit: {:?}", e))?;

            Ok(Self {
                hwnd,
                icon_data,
                context_menu: menu,
            })
        }
    }

    /// Update tooltip with current stats (T461)
    pub fn update_tooltip(&mut self, cpu: f32, memory_used_gb: f32, memory_total_gb: f32) {
        let tooltip = format!(
            "Task Manager\nCPU: {:.1}%\nMemory: {:.1}/{:.1} GB",
            cpu, memory_used_gb, memory_total_gb
        );
        
        let tooltip_wide: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
        self.icon_data.szTip[..tooltip_wide.len().min(128)].copy_from_slice(&tooltip_wide[..tooltip_wide.len().min(128)]);
        
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &self.icon_data);
        }
    }

    /// Handle tray icon messages (T459, T460, T462)
    pub fn handle_message(&self, lparam: LPARAM) -> Option<TrayAction> {
        let msg = lparam.0 as u32;
        
        match msg {
            // Double-click to show/hide window (T462)
            WM_LBUTTONDBLCLK => {
                Some(TrayAction::ToggleWindow)
            }
            
            // Right-click to show menu (T459)
            WM_RBUTTONUP => {
                unsafe {
                    // Get cursor position
                    let mut pt = POINT { x: 0, y: 0 };
                    let _ = GetCursorPos(&mut pt);
                    
                    // Required for menu to close properly when clicking outside
                    let _ = SetForegroundWindow(self.hwnd);
                    
                    // Show context menu
                    let cmd = TrackPopupMenu(
                        self.context_menu,
                        TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
                        pt.x,
                        pt.y,
                        Some(0),
                        self.hwnd,
                        None,
                    );
                    
                    // Process menu selection
                    if cmd.0 == ID_TRAY_SHOW as i32 {
                        Some(TrayAction::ShowWindow)
                    } else if cmd.0 == ID_TRAY_HIDE as i32 {
                        Some(TrayAction::HideWindow)
                    } else if cmd.0 == ID_TRAY_EXIT as i32 {
                        Some(TrayAction::Exit)
                    } else {
                        None
                    }
                }
            }
            
            _ => None,
        }
    }

    /// Remove tray icon on cleanup
    pub fn remove(&mut self) -> Result<(), String> {
        unsafe {
            let result = Shell_NotifyIconW(NIM_DELETE, &self.icon_data);
            if !result.as_bool() {
                return Err("Failed to remove tray icon".to_string());
            }
            
            DestroyMenu(self.context_menu)
                .map_err(|e| format!("Failed to destroy menu: {:?}", e))?;
            
            Ok(())
        }
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        let _ = self.remove();
    }
}

/// Actions triggered by tray icon (T459, T460, T462)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    /// Show the main window
    ShowWindow,
    /// Hide the main window (minimize to tray)
    HideWindow,
    /// Toggle window visibility
    ToggleWindow,
    /// Exit the application
    Exit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_action_variants() {
        // Verify all action variants are defined
        let actions = [
            TrayAction::ShowWindow,
            TrayAction::HideWindow,
            TrayAction::ToggleWindow,
            TrayAction::Exit,
        ];
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn test_message_ids() {
        // Verify message IDs don't overlap
        assert_ne!(ID_TRAY_SHOW, ID_TRAY_HIDE);
        assert_ne!(ID_TRAY_SHOW, ID_TRAY_EXIT);
        assert_ne!(ID_TRAY_HIDE, ID_TRAY_EXIT);
    }
}
