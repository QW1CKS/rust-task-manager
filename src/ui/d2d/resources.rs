//! Direct2D resource management (brushes, colors, text formats)
//!
//! T312: Implements lazy resource creation - brushes are created on first use
//! to improve startup time.
//! T320: Memory usage tracking for D2D resources

use std::cell::OnceCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;

/// Global memory tracker for D2D resources (T320)
static D2D_RESOURCE_MEMORY: AtomicUsize = AtomicUsize::new(0);

/// Resource pool for D2D rendering with lazy initialization (T312)
pub struct ResourcePool {
    // Device context reference for creating brushes on-demand
    device_context: ID2D1DeviceContext,
    
    // DirectWrite factory reference
    dwrite_factory: IDWriteFactory,
    
    // Lazily-initialized solid color brushes (T312)
    white_brush: OnceCell<ID2D1SolidColorBrush>,
    black_brush: OnceCell<ID2D1SolidColorBrush>,
    gray_brush: OnceCell<ID2D1SolidColorBrush>,
    blue_brush: OnceCell<ID2D1SolidColorBrush>,
    light_gray_brush: OnceCell<ID2D1SolidColorBrush>,

    // Lazily-initialized text formats (T312)
    default_text_format: OnceCell<IDWriteTextFormat>,
    bold_text_format: OnceCell<IDWriteTextFormat>,
    title_text_format: OnceCell<IDWriteTextFormat>,
    
    // Memory tracking (T320)
    brush_memory: usize,
    text_format_memory: usize,
}

impl ResourcePool {
    /// Estimated memory per brush (T320)
    const BRUSH_MEMORY: usize = 256; // bytes
    
    /// Estimated memory per text format (T320)
    const TEXT_FORMAT_MEMORY: usize = 512; // bytes
    /// Create a new resource pool (T312: resources created lazily on first use)
    pub fn new(
        device_context: &ID2D1DeviceContext,
        dwrite_factory: &IDWriteFactory,
    ) -> Result<Self> {
        Ok(Self {
            device_context: device_context.clone(),
            dwrite_factory: dwrite_factory.clone(),
            white_brush: OnceCell::new(),
            black_brush: OnceCell::new(),
            gray_brush: OnceCell::new(),
            blue_brush: OnceCell::new(),
            light_gray_brush: OnceCell::new(),
            default_text_format: OnceCell::new(),
            bold_text_format: OnceCell::new(),
            title_text_format: OnceCell::new(),
            brush_memory: 0,
            text_format_memory: 0,
        })
    }
    
    /// Get current D2D resource memory usage (T320)
    pub fn memory_usage(&self) -> usize {
        self.brush_memory + self.text_format_memory
    }
    
    /// Get global D2D resource memory (T320)
    pub fn global_memory_usage() -> usize {
        D2D_RESOURCE_MEMORY.load(Ordering::Relaxed)
    }

    /// Get white brush (created on first use - T312, T320)
    pub fn white_brush(&self) -> &ID2D1SolidColorBrush {
        self.white_brush.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::BRUSH_MEMORY, Ordering::Relaxed);
            let render_target: &ID2D1RenderTarget = unsafe { std::mem::transmute(&self.device_context) };
            unsafe {
                render_target.CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    },
                    None,
                ).expect("Failed to create white brush")
            }
        })
    }

    /// Get black brush (created on first use - T312, T320)
    pub fn black_brush(&self) -> &ID2D1SolidColorBrush {
        self.black_brush.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::BRUSH_MEMORY, Ordering::Relaxed);
            let render_target: &ID2D1RenderTarget = unsafe { std::mem::transmute(&self.device_context) };
            unsafe {
                render_target.CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    None,
                ).expect("Failed to create black brush")
            }
        })
    }

    /// Get gray brush (created on first use - T312, T320)
    pub fn gray_brush(&self) -> &ID2D1SolidColorBrush {
        self.gray_brush.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::BRUSH_MEMORY, Ordering::Relaxed);
            let render_target: &ID2D1RenderTarget = unsafe { std::mem::transmute(&self.device_context) };
            unsafe {
                render_target.CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    },
                    None,
                ).expect("Failed to create gray brush")
            }
        })
    }

    /// Get default text format (created on first use - T312, T320)
    pub fn default_text_format(&self) -> &IDWriteTextFormat {
        self.default_text_format.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::TEXT_FORMAT_MEMORY, Ordering::Relaxed);
            unsafe {
                self.dwrite_factory.CreateTextFormat(
                    windows::core::w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    12.0,
                    windows::core::w!("en-us"),
                ).expect("Failed to create default text format")
            }
        })
    }
    
    /// Get blue brush (created on first use)
    pub fn blue_brush(&self) -> &ID2D1SolidColorBrush {
        self.blue_brush.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::BRUSH_MEMORY, Ordering::Relaxed);
            let render_target: &ID2D1RenderTarget = unsafe { std::mem::transmute(&self.device_context) };
            unsafe {
                render_target.CreateSolidColorBrush(
                    &D2D1_COLOR_F { r: 0.0, g: 0.48, b: 1.0, a: 1.0 },
                    None,
                ).expect("Failed to create blue brush")
            }
        })
    }
    
    /// Get light gray brush (created on first use)
    pub fn light_gray_brush(&self) -> &ID2D1SolidColorBrush {
        self.light_gray_brush.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::BRUSH_MEMORY, Ordering::Relaxed);
            let render_target: &ID2D1RenderTarget = unsafe { std::mem::transmute(&self.device_context) };
            unsafe {
                render_target.CreateSolidColorBrush(
                    &D2D1_COLOR_F { r: 0.9, g: 0.9, b: 0.9, a: 1.0 },
                    None,
                ).expect("Failed to create light gray brush")
            }
        })
    }
    
    /// Get bold text format (created on first use)
    pub fn bold_text_format(&self) -> &IDWriteTextFormat {
        self.bold_text_format.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::TEXT_FORMAT_MEMORY, Ordering::Relaxed);
            unsafe {
                self.dwrite_factory.CreateTextFormat(
                    windows::core::w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL,
                    12.0, windows::core::w!("en-us"),
                ).expect("Failed to create bold text format")
            }
        })
    }
    
    /// Get title text format (created on first use)
    pub fn title_text_format(&self) -> &IDWriteTextFormat {
        self.title_text_format.get_or_init(|| {
            D2D_RESOURCE_MEMORY.fetch_add(Self::TEXT_FORMAT_MEMORY, Ordering::Relaxed);
            unsafe {
                self.dwrite_factory.CreateTextFormat(
                    windows::core::w!("Segoe UI"), None, DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL,
                    18.0, windows::core::w!("en-us"),
                ).expect("Failed to create title text format")
            }
        })
    }
    
    // Convenience aliases for common UI elements
    pub fn background_brush(&self) -> &ID2D1SolidColorBrush { self.white_brush() }
    pub fn text_brush(&self) -> &ID2D1SolidColorBrush { self.black_brush() }
    pub fn border_brush(&self) -> &ID2D1SolidColorBrush { self.gray_brush() }
    pub fn header_brush(&self) -> &ID2D1SolidColorBrush { self.light_gray_brush() }
}
