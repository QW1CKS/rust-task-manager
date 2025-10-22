//! Fluent Design System implementation (T384-T389)
//!
//! Implements Windows 11 Fluent Design visual effects:
//! - Reveal effect on hover (subtle highlight)
//! - Rounded corners (4px radius)
//! - Drop shadows for elevated elements
//! - Connected animations (element transitions between views)
//! - Parallax effect on scroll

use windows::Win32::Graphics::Direct2D::{
    ID2D1DeviceContext, ID2D1SolidColorBrush,
    Common::{D2D1_COLOR_F, D2D_RECT_F},
    D2D1_ROUNDED_RECT,
};
use windows::core::Result;

/// Fluent Design constants
pub mod constants {
    /// Corner radius for controls (T385)
    pub const CORNER_RADIUS: f32 = 4.0;
    
    /// Corner radius for cards/panels
    pub const CORNER_RADIUS_LARGE: f32 = 8.0;
    
    /// Shadow offset for elevated elements (T386)
    pub const SHADOW_OFFSET_X: f32 = 0.0;
    pub const SHADOW_OFFSET_Y: f32 = 2.0;
    
    /// Shadow blur radius
    pub const SHADOW_BLUR: f32 = 8.0;
    
    /// Shadow color (black with alpha)
    pub const SHADOW_COLOR: u32 = 0x40000000; // 25% black
    
    /// Reveal effect intensity (T384)
    pub const REVEAL_INTENSITY: f32 = 0.15; // 15% white overlay
    
    /// Reveal gradient size (pixels from cursor)
    pub const REVEAL_GRADIENT_SIZE: f32 = 100.0;
    
    /// Parallax offset multiplier (T388)
    pub const PARALLAX_MULTIPLIER: f32 = 0.05; // 5% of scroll distance
}

/// Reveal effect state (T384)
#[derive(Debug, Clone, Copy)]
pub struct RevealEffect {
    /// Mouse position (relative to control)
    pub mouse_x: f32,
    pub mouse_y: f32,
    /// Reveal intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Is mouse over control?
    pub active: bool,
}

impl RevealEffect {
    /// Create new reveal effect
    pub fn new() -> Self {
        Self {
            mouse_x: 0.0,
            mouse_y: 0.0,
            intensity: 0.0,
            active: false,
        }
    }

    /// Update mouse position
    pub fn update_mouse(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
        self.active = true;
    }

    /// Mouse left control
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Animate intensity (call each frame)
    pub fn animate(&mut self, delta_time: f32) {
        if self.active {
            // Fade in
            self.intensity = (self.intensity + delta_time * 5.0).min(1.0);
        } else {
            // Fade out
            self.intensity = (self.intensity - delta_time * 5.0).max(0.0);
        }
    }

    /// Render reveal effect at cursor position
    pub fn render(
        &self,
        context: &ID2D1DeviceContext,
        brush: &ID2D1SolidColorBrush,
        rect: &D2D_RECT_F,
    ) -> Result<()> {
        if self.intensity <= 0.0 {
            return Ok(());
        }

        unsafe {
            // Calculate reveal gradient center
            let center_x = rect.left + self.mouse_x;
            let center_y = rect.top + self.mouse_y;
            
            // Create radial gradient brush centered at mouse
            // Note: This is a simplified version. Full implementation would use
            // ID2D1RadialGradientBrush with proper gradient stops.
            
            // For now, just draw a subtle circular highlight at cursor position
            let highlight_color = D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: self.intensity * constants::REVEAL_INTENSITY,
            };
            
            brush.SetColor(&highlight_color);
            
            // TODO: Draw ellipse requires D2D1_ELLIPSE which is not available
            // in current windows-rs version. Will implement when API is available.
            // For now, draw a simple rectangle highlight at mouse position
            
            let highlight_rect = D2D_RECT_F {
                left: center_x - constants::REVEAL_GRADIENT_SIZE / 2.0,
                top: center_y - constants::REVEAL_GRADIENT_SIZE / 2.0,
                right: center_x + constants::REVEAL_GRADIENT_SIZE / 2.0,
                bottom: center_y + constants::REVEAL_GRADIENT_SIZE / 2.0,
            };
            
            context.FillRectangle(&highlight_rect, brush);
        }

        Ok(())
    }
}

impl Default for RevealEffect {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw rounded rectangle with Fluent corners (T385)
pub fn draw_rounded_rect(
    context: &ID2D1DeviceContext,
    brush: &ID2D1SolidColorBrush,
    rect: &D2D_RECT_F,
    radius: f32,
) -> Result<()> {
    unsafe {
        let rounded_rect = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: radius,
            radiusY: radius,
        };
        
        context.FillRoundedRectangle(&rounded_rect, brush);
    }
    
    Ok(())
}

/// Draw rounded rectangle border
pub fn draw_rounded_rect_border(
    context: &ID2D1DeviceContext,
    brush: &ID2D1SolidColorBrush,
    rect: &D2D_RECT_F,
    radius: f32,
    stroke_width: f32,
) -> Result<()> {
    unsafe {
        let rounded_rect = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: radius,
            radiusY: radius,
        };
        
        context.DrawRoundedRectangle(&rounded_rect, brush, stroke_width, None);
    }
    
    Ok(())
}

/// Draw drop shadow for elevated element (T386)
pub fn draw_drop_shadow(
    context: &ID2D1DeviceContext,
    brush: &ID2D1SolidColorBrush,
    rect: &D2D_RECT_F,
    radius: f32,
) -> Result<()> {
    unsafe {
        // Shadow is drawn below and slightly offset from the element
        let shadow_rect = D2D_RECT_F {
            left: rect.left + constants::SHADOW_OFFSET_X,
            top: rect.top + constants::SHADOW_OFFSET_Y,
            right: rect.right + constants::SHADOW_OFFSET_X,
            bottom: rect.bottom + constants::SHADOW_OFFSET_Y,
        };
        
        let shadow_color = D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.25, // 25% opacity
        };
        
        brush.SetColor(&shadow_color);
        
        let rounded_shadow = D2D1_ROUNDED_RECT {
            rect: shadow_rect,
            radiusX: radius,
            radiusY: radius,
        };
        
        // Draw shadow (note: real shadow would use blur effect)
        context.FillRoundedRectangle(&rounded_shadow, brush);
    }
    
    Ok(())
}

/// Connected animation state (T389)
#[derive(Debug, Clone, Copy)]
pub struct ConnectedAnimation {
    /// Starting position
    pub start_x: f32,
    pub start_y: f32,
    /// Ending position
    pub end_x: f32,
    pub end_y: f32,
    /// Animation progress (0.0 - 1.0)
    pub progress: f32,
    /// Animation duration (seconds)
    pub duration: f32,
    /// Is animation active?
    pub active: bool,
}

impl ConnectedAnimation {
    /// Create new connected animation
    pub fn new(start_x: f32, start_y: f32, end_x: f32, end_y: f32, duration: f32) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
            progress: 0.0,
            duration,
            active: true,
        }
    }

    /// Update animation (call each frame)
    pub fn update(&mut self, delta_time: f32) -> bool {
        if !self.active {
            return false;
        }

        self.progress += delta_time / self.duration;
        
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.active = false;
        }

        true
    }

    /// Get current position with easing
    pub fn current_position(&self) -> (f32, f32) {
        // Ease out cubic for smooth deceleration
        let t = self.progress;
        let eased = 1.0 - (1.0 - t).powi(3);
        
        let x = self.start_x + (self.end_x - self.start_x) * eased;
        let y = self.start_y + (self.end_y - self.start_y) * eased;
        
        (x, y)
    }
}

/// Parallax effect for scrolling (T388)
#[derive(Debug, Clone, Copy)]
pub struct ParallaxEffect {
    /// Scroll offset
    pub scroll_offset: f32,
    /// Parallax layer depth (0.0 = no parallax, 1.0 = full)
    pub depth: f32,
}

impl ParallaxEffect {
    /// Create new parallax effect
    pub fn new(depth: f32) -> Self {
        Self {
            scroll_offset: 0.0,
            depth: depth.clamp(0.0, 1.0),
        }
    }

    /// Update scroll offset
    pub fn set_scroll(&mut self, offset: f32) {
        self.scroll_offset = offset;
    }

    /// Get parallax offset for this layer
    pub fn get_offset(&self) -> f32 {
        self.scroll_offset * self.depth * constants::PARALLAX_MULTIPLIER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reveal_effect_animation() {
        let mut reveal = RevealEffect::new();
        assert_eq!(reveal.intensity, 0.0);
        
        reveal.update_mouse(50.0, 50.0);
        assert!(reveal.active);
        
        reveal.animate(0.1); // 100ms
        assert!(reveal.intensity > 0.0);
        assert!(reveal.intensity <= 1.0);
    }

    #[test]
    fn test_connected_animation() {
        let mut anim = ConnectedAnimation::new(0.0, 0.0, 100.0, 100.0, 0.5);
        
        anim.update(0.25); // Half duration
        let (x, y) = anim.current_position();
        assert!(x > 0.0 && x < 100.0);
        assert!(y > 0.0 && y < 100.0);
        
        anim.update(0.25); // Complete
        assert!(!anim.active);
        let (x, y) = anim.current_position();
        assert_eq!(x, 100.0);
        assert_eq!(y, 100.0);
    }

    #[test]
    fn test_parallax_effect() {
        let mut parallax = ParallaxEffect::new(0.5);
        parallax.set_scroll(100.0);
        
        let offset = parallax.get_offset();
        assert!(offset > 0.0);
        assert!(offset < 100.0); // Should be less than full scroll
    }
}
