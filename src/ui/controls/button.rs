//! Button control with Fluent Design styling

use super::Control;
use crate::ui::input::{MouseEvent, KeyboardEvent, MouseButton};
use crate::ui::layout::Rect;
use windows::Win32::Graphics::Direct2D::{
    ID2D1DeviceContext,
    Common::D2D1_COLOR_F,
};
use windows::core::Result;

/// Button states for Fluent Design
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Normal state (default)
    Normal,
    /// Hover state (mouse over)
    Hover,
    /// Pressed state (mouse down)
    Pressed,
    /// Disabled state (not interactive)
    Disabled,
}

/// Button control with Fluent Design styling
pub struct Button {
    #[allow(dead_code)]
    text: String,
    state: ButtonState,
    enabled: bool,
    focused: bool,
    on_click: Option<Box<dyn Fn()>>,
}

impl Button {
    /// Create a new button with the given text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            state: ButtonState::Normal,
            enabled: true,
            focused: false,
            on_click: None,
        }
    }

    /// Add a callback function that will be called when the button is clicked
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn() + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    /// Enable or disable the button
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.state = if enabled {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        };
    }

    /// Get Fluent Design colors based on state
    fn get_colors(&self) -> (D2D1_COLOR_F, D2D1_COLOR_F, D2D1_COLOR_F) {
        // (background, border, text)
        match self.state {
            ButtonState::Normal => (
                D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, // Transparent
                D2D1_COLOR_F { r: 0.6, g: 0.6, b: 0.6, a: 1.0 }, // Gray border
                D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White text
            ),
            ButtonState::Hover => (
                D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.1 }, // 10% white fill
                D2D1_COLOR_F { r: 0.8, g: 0.8, b: 0.8, a: 1.0 }, // Lighter border
                D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White text
            ),
            ButtonState::Pressed => (
                D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.2 }, // 20% white fill
                D2D1_COLOR_F { r: 0.9, g: 0.9, b: 0.9, a: 1.0 }, // Even lighter border
                D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.9 }, // Slightly dimmed text
            ),
            ButtonState::Disabled => (
                D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, // Transparent
                D2D1_COLOR_F { r: 0.3, g: 0.3, b: 0.3, a: 1.0 }, // Dark border
                D2D1_COLOR_F { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }, // Gray text
            ),
        }
    }
}

impl Control for Button {
    fn render(&self, context: &ID2D1DeviceContext, rect: Rect) -> Result<()> {
        unsafe {
            let d2d_rect = rect.to_d2d();

            let (bg_color, border_color, text_color) = self.get_colors();

            // Create brushes for this render (TODO: cache these)
            let render_target: &windows::Win32::Graphics::Direct2D::ID2D1RenderTarget = 
                std::mem::transmute(context);

            let bg_brush = render_target.CreateSolidColorBrush(&bg_color, None)?;
            let border_brush = render_target.CreateSolidColorBrush(&border_color, None)?;
            let _text_brush = render_target.CreateSolidColorBrush(&text_color, None)?;

            // Draw background
            context.FillRectangle(&d2d_rect, &bg_brush);

            // Draw border (rounded rectangle would be better, but this is MVP)
            context.DrawRectangle(&d2d_rect, &border_brush, 1.0, None);

            // Draw focus indicator if focused
            if self.focused {
                let focus_rect = Rect::new(
                    rect.x + 2.0,
                    rect.y + 2.0,
                    rect.width - 4.0,
                    rect.height - 4.0,
                ).to_d2d();
                let focus_color = D2D1_COLOR_F { r: 0.0, g: 0.5, b: 1.0, a: 1.0 }; // Blue
                let focus_brush = render_target.CreateSolidColorBrush(&focus_color, None)?;
                context.DrawRectangle(&focus_rect, &focus_brush, 2.0, None);
            }

            // Draw text (TODO: use cached text format)
            // For now, skip text rendering until we wire up resources properly
            // In full implementation, would use IDWriteTextFormat + DrawText

            Ok(())
        }
    }

    fn handle_mouse(&mut self, event: MouseEvent, rect: Rect) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            MouseEvent::Move { x, y, .. } => {
                let inside = rect.contains(x as f32, y as f32);
                if inside && self.state != ButtonState::Pressed {
                    self.state = ButtonState::Hover;
                } else if !inside && self.state == ButtonState::Hover {
                    self.state = ButtonState::Normal;
                }
                inside
            }
            MouseEvent::ButtonDown { button: MouseButton::Left, x, y, .. } => {
                if rect.contains(x as f32, y as f32) {
                    self.state = ButtonState::Pressed;
                    true
                } else {
                    false
                }
            }
            MouseEvent::ButtonUp { button: MouseButton::Left, x, y, .. } => {
                if self.state == ButtonState::Pressed && rect.contains(x as f32, y as f32) {
                    // Click!
                    if let Some(ref callback) = self.on_click {
                        callback();
                    }
                    self.state = ButtonState::Hover;
                    true
                } else {
                    self.state = ButtonState::Normal;
                    false
                }
            }
            _ => false,
        }
    }

    fn handle_keyboard(&mut self, event: KeyboardEvent) -> bool {
        if !self.enabled || !self.focused {
            return false;
        }

        use windows::Win32::UI::Input::KeyboardAndMouse::{VK_RETURN, VK_SPACE};

        match event {
            KeyboardEvent::KeyDown { vkey, .. } => {
                if vkey == VK_RETURN.0 || vkey == VK_SPACE.0 {
                    // Activate button
                    if let Some(ref callback) = self.on_click {
                        callback();
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn can_focus(&self) -> bool {
        self.enabled
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}
