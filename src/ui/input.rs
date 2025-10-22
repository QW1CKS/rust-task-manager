//! Input handling for mouse and keyboard events

#![allow(missing_docs)]
#![allow(dead_code)]

use windows::Win32::Foundation::{LPARAM, WPARAM};

// Mouse button flags (from Windows SDK)
const MK_LBUTTON: u32 = 0x0001;
const MK_RBUTTON: u32 = 0x0002;
const MK_SHIFT: u32 = 0x0004;
const MK_CONTROL: u32 = 0x0008;
const MK_MBUTTON: u32 = 0x0010;

// Helper macros (from Windows SDK windowsx.h)
fn get_x_lparam(lp: LPARAM) -> i32 {
    (lp.0 as i16) as i32
}

fn get_y_lparam(lp: LPARAM) -> i32 {
    ((lp.0 >> 16) as i16) as i32
}

fn get_wheel_delta_wparam(wp: WPARAM) -> i16 {
    (wp.0 >> 16) as i16
}

/// Mouse button state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse event type
#[derive(Debug, Clone, Copy)]
pub enum MouseEvent {
    ButtonDown { button: MouseButton, x: i32, y: i32, modifiers: KeyModifiers },
    ButtonUp { button: MouseButton, x: i32, y: i32, modifiers: KeyModifiers },
    Move { x: i32, y: i32, modifiers: KeyModifiers },
    Wheel { delta: i16, x: i32, y: i32, modifiers: KeyModifiers },
}

/// Keyboard event type
#[derive(Debug, Clone, Copy)]
pub enum KeyboardEvent {
    KeyDown { vkey: u16, repeat_count: u16, modifiers: KeyModifiers },
    KeyUp { vkey: u16, modifiers: KeyModifiers },
    Char { character: char, repeat_count: u16 },
}

/// Keyboard modifiers (Ctrl, Shift, Alt)
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyModifiers {
    /// Parse modifiers from WPARAM (for mouse events)
    pub fn from_mouse_wparam(wparam: WPARAM) -> Self {
        let flags = wparam.0 as u32;
        Self {
            ctrl: (flags & MK_CONTROL) != 0,
            shift: (flags & MK_SHIFT) != 0,
            alt: false, // Not available in mouse messages
        }
    }

    /// Parse modifiers from GetKeyState (for keyboard events)
    pub fn from_keyboard_state() -> Self {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_SHIFT, VK_MENU};
        unsafe {
            Self {
                ctrl: GetKeyState(VK_CONTROL.0 as i32) < 0,
                shift: GetKeyState(VK_SHIFT.0 as i32) < 0,
                alt: GetKeyState(VK_MENU.0 as i32) < 0,
            }
        }
    }
}

/// Parse WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN
pub fn parse_button_down(button: MouseButton, lparam: LPARAM, wparam: WPARAM) -> MouseEvent {
    MouseEvent::ButtonDown {
        button,
        x: get_x_lparam(lparam),
        y: get_y_lparam(lparam),
        modifiers: KeyModifiers::from_mouse_wparam(wparam),
    }
}

/// Parse WM_LBUTTONUP, WM_RBUTTONUP, WM_MBUTTONUP
pub fn parse_button_up(button: MouseButton, lparam: LPARAM, wparam: WPARAM) -> MouseEvent {
    MouseEvent::ButtonUp {
        button,
        x: get_x_lparam(lparam),
        y: get_y_lparam(lparam),
        modifiers: KeyModifiers::from_mouse_wparam(wparam),
    }
}

/// Parse WM_MOUSEMOVE
pub fn parse_mouse_move(lparam: LPARAM, wparam: WPARAM) -> MouseEvent {
    MouseEvent::Move {
        x: get_x_lparam(lparam),
        y: get_y_lparam(lparam),
        modifiers: KeyModifiers::from_mouse_wparam(wparam),
    }
}

/// Parse WM_MOUSEWHEEL
pub fn parse_mouse_wheel(lparam: LPARAM, wparam: WPARAM) -> MouseEvent {
    let delta = get_wheel_delta_wparam(wparam);
    MouseEvent::Wheel {
        delta,
        x: get_x_lparam(lparam),
        y: get_y_lparam(lparam),
        modifiers: KeyModifiers::from_mouse_wparam(wparam),
    }
}

/// Parse WM_KEYDOWN, WM_SYSKEYDOWN
pub fn parse_key_down(wparam: WPARAM, lparam: LPARAM) -> KeyboardEvent {
    let vkey = wparam.0 as u16;
    let repeat_count = (lparam.0 & 0xFFFF) as u16;
    KeyboardEvent::KeyDown {
        vkey,
        repeat_count,
        modifiers: KeyModifiers::from_keyboard_state(),
    }
}

/// Parse WM_KEYUP, WM_SYSKEYUP
pub fn parse_key_up(wparam: WPARAM, _lparam: LPARAM) -> KeyboardEvent {
    let vkey = wparam.0 as u16;
    KeyboardEvent::KeyUp {
        vkey,
        modifiers: KeyModifiers::from_keyboard_state(),
    }
}

/// Parse WM_CHAR
pub fn parse_char(wparam: WPARAM, lparam: LPARAM) -> Option<KeyboardEvent> {
    let char_code = wparam.0 as u32;
    let repeat_count = (lparam.0 & 0xFFFF) as u16;
    
    // Convert to char (handles UTF-16 surrogates) - use map per clippy suggestion
    char::from_u32(char_code).map(|character| KeyboardEvent::Char { character, repeat_count })
}

/// Hit testing result for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestResult {
    None,
    TitleBar,
    ProcessTable { row: usize },
    Button { id: u32 },
    TabHeader { index: usize },
    GraphArea,
}

/// Focus manager for keyboard navigation (T405)
pub struct FocusManager {
    focused_element: Option<u32>,
    focusable_elements: Vec<u32>,
    current_index: Option<usize>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused_element: None,
            focusable_elements: Vec::new(),
            current_index: None,
        }
    }

    /// Register a focusable element
    pub fn register(&mut self, id: u32) {
        if !self.focusable_elements.contains(&id) {
            self.focusable_elements.push(id);
        }
    }

    /// Move focus to next element (Tab key)
    pub fn focus_next(&mut self) -> Option<u32> {
        if self.focusable_elements.is_empty() {
            return None;
        }

        let next_index = match self.current_index {
            None => 0,
            Some(idx) => (idx + 1) % self.focusable_elements.len(),
        };
        
        self.current_index = Some(next_index);
        self.focused_element = Some(self.focusable_elements[next_index]);
        self.focused_element
    }

    /// Move focus to previous element (Shift+Tab)
    pub fn focus_previous(&mut self) -> Option<u32> {
        if self.focusable_elements.is_empty() {
            return None;
        }

        let prev_index = match self.current_index {
            None => self.focusable_elements.len() - 1,
            Some(0) => self.focusable_elements.len() - 1,
            Some(idx) => idx - 1,
        };
        
        self.current_index = Some(prev_index);
        self.focused_element = Some(self.focusable_elements[prev_index]);
        self.focused_element
    }

    /// Get currently focused element
    pub fn get_focused(&self) -> Option<u32> {
        self.focused_element
    }

    /// Check if element has focus
    pub fn has_focus(&self, id: u32) -> bool {
        self.focused_element == Some(id)
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard shortcut definition (T408)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    /// Ctrl+F - Open filter box
    Find,
    /// Delete - End selected process
    Delete,
    /// F5 - Refresh data
    Refresh,
    /// Ctrl+Tab - Switch to next tab
    NextTab,
    /// Ctrl+Shift+Tab - Switch to previous tab
    PreviousTab,
    /// Escape - Close dialog/cancel operation
    Escape,
    /// Enter - Activate focused element
    Enter,
    /// Ctrl+1 through Ctrl+6 - Switch to specific tab
    SwitchToTab(u8),
}

impl Shortcut {
    /// Try to parse a keyboard event into a shortcut
    pub fn from_keyboard_event(event: &KeyboardEvent) -> Option<Self> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        
        match event {
            KeyboardEvent::KeyDown { vkey, modifiers, .. } => {
                let vkey = *vkey;
                
                // Escape key
                if vkey == VK_ESCAPE.0 {
                    return Some(Shortcut::Escape);
                }
                
                // Enter key
                if vkey == VK_RETURN.0 {
                    return Some(Shortcut::Enter);
                }
                
                // Delete key
                if vkey == VK_DELETE.0 {
                    return Some(Shortcut::Delete);
                }
                
                // F5 key (refresh)
                if vkey == VK_F5.0 {
                    return Some(Shortcut::Refresh);
                }
                
                // Ctrl+F (find)
                if vkey == 'F' as u16 && modifiers.ctrl && !modifiers.shift {
                    return Some(Shortcut::Find);
                }
                
                // Ctrl+Tab (next tab)
                if vkey == VK_TAB.0 && modifiers.ctrl && !modifiers.shift {
                    return Some(Shortcut::NextTab);
                }
                
                // Ctrl+Shift+Tab (previous tab)
                if vkey == VK_TAB.0 && modifiers.ctrl && modifiers.shift {
                    return Some(Shortcut::PreviousTab);
                }
                
                // Ctrl+1 through Ctrl+6 (switch to specific tab)
                if modifiers.ctrl && !modifiers.shift {
                    if vkey >= '1' as u16 && vkey <= '6' as u16 {
                        let tab_index = (vkey - '1' as u16) as u8;
                        return Some(Shortcut::SwitchToTab(tab_index));
                    }
                }
                
                None
            }
            _ => None,
        }
    }
    
    /// Get human-readable shortcut string
    pub fn to_string(&self) -> &'static str {
        match self {
            Shortcut::Find => "Ctrl+F",
            Shortcut::Delete => "Delete",
            Shortcut::Refresh => "F5",
            Shortcut::NextTab => "Ctrl+Tab",
            Shortcut::PreviousTab => "Ctrl+Shift+Tab",
            Shortcut::Escape => "Escape",
            Shortcut::Enter => "Enter",
            Shortcut::SwitchToTab(n) => match n {
                0 => "Ctrl+1",
                1 => "Ctrl+2",
                2 => "Ctrl+3",
                3 => "Ctrl+4",
                4 => "Ctrl+5",
                5 => "Ctrl+6",
                _ => "Unknown",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager() {
        let mut focus = FocusManager::new();
        focus.register(1);
        focus.register(2);
        focus.register(3);
        
        // First call to focus_next goes to index 0 (element 1)
        assert_eq!(focus.focus_next(), Some(1));
        // Second call goes to index 1 (element 2)
        assert_eq!(focus.focus_next(), Some(2));
        // focus_previous goes back to index 0 (element 1)
        assert_eq!(focus.focus_previous(), Some(1));
        // Verify we can cycle through all elements
        assert_eq!(focus.focus_next(), Some(2));
        assert_eq!(focus.focus_next(), Some(3));
        // Wraps around to element 1
        assert_eq!(focus.focus_next(), Some(1));
    }

    #[test]
    fn test_shortcut_strings() {
        assert_eq!(Shortcut::Find.to_string(), "Ctrl+F");
        assert_eq!(Shortcut::Delete.to_string(), "Delete");
        assert_eq!(Shortcut::Refresh.to_string(), "F5");
        assert_eq!(Shortcut::SwitchToTab(0).to_string(), "Ctrl+1");
    }
}
