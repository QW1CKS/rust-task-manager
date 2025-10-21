//! UI controls module

pub mod button;
pub mod context_menu;
pub mod filter_box;
pub mod table;

use crate::ui::input::{MouseEvent, KeyboardEvent};
use crate::ui::layout::Rect;
use windows::Win32::Graphics::Direct2D::ID2D1DeviceContext;
use windows::core::Result;

/// Base trait for all UI controls
pub trait Control {
    /// Render the control
    fn render(&self, context: &ID2D1DeviceContext, rect: Rect) -> Result<()>;

    /// Handle mouse event, returns true if handled
    fn handle_mouse(&mut self, event: MouseEvent, rect: Rect) -> bool;

    /// Handle keyboard event, returns true if handled
    fn handle_keyboard(&mut self, event: KeyboardEvent) -> bool;

    /// Hit test - returns true if point is within control bounds
    fn hit_test(&self, x: f32, y: f32, rect: Rect) -> bool {
        rect.contains(x, y)
    }

    /// Set DPI for the control (called on DPI change)
    fn set_dpi(&mut self, _dpi: u32) {
        // Default: no-op, override if needed
    }

    /// Returns true if control can receive keyboard focus
    fn can_focus(&self) -> bool {
        false
    }

    /// Set focus state
    fn set_focus(&mut self, _focused: bool) {
        // Default: no-op, override if needed
    }
}
