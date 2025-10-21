//! Layout system with DPI-aware rectangle calculations

use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::HiDpi::GetDpiForWindow;

/// Layout rectangle with logical pixels
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    /// X coordinate (left)
    pub x: f32,
    /// Y coordinate (top)
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create from Direct2D rectangle
    pub fn from_d2d(rect: D2D_RECT_F) -> Self {
        Self {
            x: rect.left,
            y: rect.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
        }
    }

    /// Convert to Direct2D rectangle
    pub fn to_d2d(&self) -> D2D_RECT_F {
        D2D_RECT_F {
            left: self.x,
            top: self.y,
            right: self.x + self.width,
            bottom: self.y + self.height,
        }
    }

    /// Check if point is inside rectangle
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Apply padding inward
    pub fn inset(&self, padding: f32) -> Self {
        Self {
            x: self.x + padding,
            y: self.y + padding,
            width: (self.width - 2.0 * padding).max(0.0),
            height: (self.height - 2.0 * padding).max(0.0),
        }
    }

    /// Apply padding with separate values
    pub fn inset_by(&self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            x: self.x + left,
            y: self.y + top,
            width: (self.width - left - right).max(0.0),
            height: (self.height - top - bottom).max(0.0),
        }
    }
}

/// Layout constraints for sizing
#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    /// Minimum width
    pub min_width: f32,
    /// Maximum width
    pub max_width: f32,
    /// Minimum height
    pub min_height: f32,
    /// Maximum height
    pub max_height: f32,
}

impl Constraints {
    /// Create new constraints
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self { min_width, max_width, min_height, max_height }
    }

    /// Create loose constraints (min = 0, max = given)
    pub fn loose(width: f32, height: f32) -> Self {
        Self {
            min_width: 0.0,
            max_width: width,
            min_height: 0.0,
            max_height: height,
        }
    }

    /// Create tight constraints (min = max = given)
    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    /// Constrain width to bounds
    pub fn constrain_width(&self, width: f32) -> f32 {
        width.max(self.min_width).min(self.max_width)
    }

    /// Constrain height to bounds
    pub fn constrain_height(&self, height: f32) -> f32 {
        height.max(self.min_height).min(self.max_height)
    }
}

/// Layout direction for flex layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// Horizontal layout (left to right)
    Horizontal,
    /// Vertical layout (top to bottom)
    Vertical,
}

/// Flexible box layout (similar to CSS flexbox)
pub struct FlexLayout {
    direction: FlexDirection,
    gap: f32,
    padding: f32,
}

impl FlexLayout {
    /// Create a new flex layout with the given direction
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            gap: 0.0,
            padding: 0.0,
        }
    }

    /// Set the gap between children
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set padding around the container
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Layout children within container
    pub fn layout(&self, container: Rect, children: &[(f32, f32)]) -> Vec<Rect> {
        let inner = container.inset(self.padding);
        let mut result = Vec::with_capacity(children.len());

        match self.direction {
            FlexDirection::Horizontal => {
                let mut x = inner.x;
                for &(width, height) in children {
                    result.push(Rect::new(x, inner.y, width, height));
                    x += width + self.gap;
                }
            }
            FlexDirection::Vertical => {
                let mut y = inner.y;
                for &(width, height) in children {
                    result.push(Rect::new(inner.x, y, width, height));
                    y += height + self.gap;
                }
            }
        }

        result
    }
}

/// DPI scaling utilities
pub struct DpiScale {
    dpi: u32,
    scale: f32,
}

impl DpiScale {
    /// Get DPI for window
    pub fn from_hwnd(hwnd: HWND) -> Self {
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        Self {
            dpi,
            scale: dpi as f32 / 96.0, // 96 DPI = 100% scale
        }
    }

    /// Convert logical pixels to physical pixels
    pub fn to_physical(&self, logical: f32) -> f32 {
        logical * self.scale
    }

    /// Convert physical pixels to logical pixels
    pub fn to_logical(&self, physical: f32) -> f32 {
        physical / self.scale
    }

    /// Scale rectangle to physical pixels
    pub fn scale_rect(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x * self.scale,
            y: rect.y * self.scale,
            width: rect.width * self.scale,
            height: rect.height * self.scale,
        }
    }

    /// Get current DPI
    pub fn dpi(&self) -> u32 {
        self.dpi
    }

    /// Get scale factor (1.0 = 96 DPI, 1.5 = 144 DPI, etc.)
    pub fn scale_factor(&self) -> f32 {
        self.scale
    }
}

/// Layout cache to avoid recalculation every frame
pub struct LayoutCache {
    cached_layout: Option<Vec<Rect>>,
    last_container: Option<Rect>,
    last_dpi: u32,
}

impl LayoutCache {
    /// Create a new empty layout cache
    pub fn new() -> Self {
        Self {
            cached_layout: None,
            last_container: None,
            last_dpi: 0,
        }
    }

    /// Get cached layout or compute if invalidated
    pub fn get_or_compute<F>(&mut self, container: Rect, dpi: u32, compute: F) -> &[Rect]
    where
        F: FnOnce() -> Vec<Rect>,
    {
        let needs_update = self.cached_layout.is_none()
            || self.last_container.map_or(true, |c| {
                (c.x - container.x).abs() > 0.1
                    || (c.y - container.y).abs() > 0.1
                    || (c.width - container.width).abs() > 0.1
                    || (c.height - container.height).abs() > 0.1
            })
            || self.last_dpi != dpi;

        if needs_update {
            self.cached_layout = Some(compute());
            self.last_container = Some(container);
            self.last_dpi = dpi;
        }

        self.cached_layout.as_ref().unwrap()
    }

    /// Invalidate cache
    pub fn invalidate(&mut self) {
        self.cached_layout = None;
    }
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(50.0, 30.0));
        assert!(!rect.contains(5.0, 30.0));
        assert!(!rect.contains(50.0, 70.0));
    }

    #[test]
    fn test_rect_inset() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inset = rect.inset(10.0);
        assert_eq!(inset.x, 10.0);
        assert_eq!(inset.y, 10.0);
        assert_eq!(inset.width, 80.0);
        assert_eq!(inset.height, 80.0);
    }

    #[test]
    fn test_flex_layout_horizontal() {
        let flex = FlexLayout::new(FlexDirection::Horizontal).with_gap(10.0);
        let container = Rect::new(0.0, 0.0, 200.0, 50.0);
        let children = vec![(50.0, 30.0), (50.0, 30.0), (50.0, 30.0)];
        
        let result = flex.layout(container, &children);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 60.0); // 50 + 10 gap
        assert_eq!(result[2].x, 120.0); // 60 + 50 + 10 gap
    }
}
