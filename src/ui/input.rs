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
    ((wp.0 >> 16) as i16)
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
    
    // Convert to char (handles UTF-16 surrogates)
    if let Some(character) = char::from_u32(char_code) {
        Some(KeyboardEvent::Char { character, repeat_count })
    } else {
        None
    }
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

/// Focus management for keyboard navigation
pub struct FocusManager {
    focused_element: Option<u32>,
    focusable_elements: Vec<u32>,
    current_index: usize,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused_element: None,
            focusable_elements: Vec::new(),
            current_index: 0,
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

        self.current_index = (self.current_index + 1) % self.focusable_elements.len();
        self.focused_element = Some(self.focusable_elements[self.current_index]);
        self.focused_element
    }

    /// Move focus to previous element (Shift+Tab)
    pub fn focus_previous(&mut self) -> Option<u32> {
        if self.focusable_elements.is_empty() {
            return None;
        }

        if self.current_index == 0 {
            self.current_index = self.focusable_elements.len() - 1;
        } else {
            self.current_index -= 1;
        }
        self.focused_element = Some(self.focusable_elements[self.current_index]);
        self.focused_element
    }

    /// Set focus to specific element
    pub fn set_focus(&mut self, id: u32) -> bool {
        if let Some(index) = self.focusable_elements.iter().position(|&x| x == id) {
            self.current_index = index;
            self.focused_element = Some(id);
            true
        } else {
            false
        }
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
