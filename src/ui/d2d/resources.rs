//! Direct2D resource management (brushes, colors, text formats)

use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;

/// Resource pool for D2D rendering
pub struct ResourcePool {
    // Solid color brushes
    white_brush: ID2D1SolidColorBrush,
    black_brush: ID2D1SolidColorBrush,
    gray_brush: ID2D1SolidColorBrush,

    // Text formats
    default_text_format: IDWriteTextFormat,
}

impl ResourcePool {
    /// Create a new resource pool
    pub fn new(
        device_context: &ID2D1DeviceContext,
        dwrite_factory: &IDWriteFactory,
    ) -> Result<Self> {
        // Create brushes using render target methods
        // Note: ID2D1DeviceContext inherits from ID2D1RenderTarget which has CreateSolidColorBrush
        
        // Cast to render target
        let render_target: &ID2D1RenderTarget = unsafe { std::mem::transmute(device_context) };
        
        let white_brush = unsafe {
            render_target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                None,
            )?
        };

        let black_brush = unsafe {
            render_target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                None,
            )?
        };

        let gray_brush = unsafe {
            render_target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
                None,
            )?
        };

        // Create default text format (Segoe UI, 12pt)
        let default_text_format = unsafe {
            dwrite_factory.CreateTextFormat(
                windows::core::w!("Segoe UI"),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                12.0,
                windows::core::w!("en-us"),
            )?
        };

        Ok(Self {
            white_brush,
            black_brush,
            gray_brush,
            default_text_format,
        })
    }

    /// Get white brush
    pub fn white_brush(&self) -> &ID2D1SolidColorBrush {
        &self.white_brush
    }

    /// Get black brush
    pub fn black_brush(&self) -> &ID2D1SolidColorBrush {
        &self.black_brush
    }

    /// Get gray brush
    pub fn gray_brush(&self) -> &ID2D1SolidColorBrush {
        &self.gray_brush
    }

    /// Get default text format
    pub fn default_text_format(&self) -> &IDWriteTextFormat {
        &self.default_text_format
    }
}
