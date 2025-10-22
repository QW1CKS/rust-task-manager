//! Direct2D renderer with hardware acceleration
//!
//! Performance optimizations (Phase 6):
//! - T333: Event-driven rendering with dirty flag tracking
//! - T334: ID2D1CommandList caching for static UI elements
//! - T335: Draw call batching to reduce overhead
//! - T336: Layer caching for expensive effects

use windows::core::{Interface, Result};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

/// Direct2D renderer with event-driven rendering
pub struct Renderer {
    // D2D Factory
    #[allow(dead_code)]
    d2d_factory: ID2D1Factory1,

    // D3D Device
    #[allow(dead_code)]
    d3d_device: ID3D11Device,

    // DXGI Device
    #[allow(dead_code)]
    dxgi_device: IDXGIDevice,

    // D2D Device
    #[allow(dead_code)]
    d2d_device: ID2D1Device,

    // D2D Device Context
    device_context: ID2D1DeviceContext,

    // DXGI Swap Chain
    swap_chain: IDXGISwapChain1,

    // DirectWrite Factory
    dwrite_factory: IDWriteFactory,

    // Bitmap render target
    #[allow(dead_code)]
    bitmap: ID2D1Bitmap1,

    // Window handle
    #[allow(dead_code)]
    hwnd: HWND,

    // Window dimensions
    width: u32,
    height: u32,

    // T333: Event-driven rendering - dirty tracking
    dirty: bool,
    last_frame_time: std::time::Instant,

    // T334: Command list for static UI caching
    static_ui_cache: Option<ID2D1CommandList>,
    cache_valid: bool,
}

impl Renderer {
    /// Create a new renderer for the given window
    pub fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        // Create D2D Factory
        let d2d_factory: ID2D1Factory1 = unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?
        };

        // Create D3D11 Device
        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;

        let feature_levels = [
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0,
            D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_10_0,
        ];

        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                Some(&mut d3d_context),
            )?;
        }

        let d3d_device = d3d_device.unwrap();

        // Get DXGI Device
        let dxgi_device: IDXGIDevice = d3d_device.cast()?;

        // Create D2D Device
        let d2d_device = unsafe { d2d_factory.CreateDevice(&dxgi_device)? };

        // Create D2D Device Context
        let device_context = unsafe { d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)? };

        // Create Swap Chain
        let dxgi_factory: IDXGIFactory2 = unsafe {
            let dxgi_adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
            dxgi_adapter.GetParent()?
        };

        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_NONE,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE,
            Flags: 0,
        };

        let swap_chain = unsafe {
            dxgi_factory.CreateSwapChainForHwnd(&d3d_device, hwnd, &swap_chain_desc, None, None)?
        };

        // Create DirectWrite Factory
        let dwrite_factory: IDWriteFactory = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?
        };

        // Set render target to swap chain backbuffer
        let backbuffer: IDXGISurface = unsafe { swap_chain.GetBuffer(0)? };

        let bitmap_properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: std::mem::ManuallyDrop::new(None),
        };

        let bitmap: ID2D1Bitmap1 = unsafe {
            device_context.CreateBitmapFromDxgiSurface(&backbuffer, Some(&bitmap_properties))?
        };

        unsafe {
            device_context.SetTarget(&bitmap);
        }

        Ok(Self {
            d2d_factory,
            d3d_device,
            dxgi_device,
            d2d_device,
            device_context,
            swap_chain,
            dwrite_factory,
            bitmap,
            hwnd,
            width,
            height,
            dirty: true, // Start dirty to force initial render
            last_frame_time: std::time::Instant::now(),
            static_ui_cache: None, // T334: Lazy create on first static UI draw
            cache_valid: false,
        })
    }

    /// Begin a frame
    pub fn begin_frame(&self) {
        unsafe {
            self.device_context.BeginDraw();
            // Clear to solid color (placeholder for Mica)
            let color = D2D1_COLOR_F {
                r: 0.95,
                g: 0.95,
                b: 0.95,
                a: 1.0,
            };
            self.device_context.Clear(Some(&color));
        }
    }

    /// End a frame and present
    pub fn end_frame(&self) -> Result<()> {
        unsafe {
            self.device_context.EndDraw(None, None)?;
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
        }
        Ok(())
    }

    /// Resize the swap chain
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.width = width;
        self.height = height;

        // T336: Invalidate caches on resize
        self.invalidate_static_cache();

        // Release render target
        unsafe {
            self.device_context.SetTarget(None);
        }

        // Resize swap chain
        unsafe {
            self.swap_chain.ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0))?;
        }

        // Recreate render target
        let backbuffer: IDXGISurface = unsafe { self.swap_chain.GetBuffer(0)? };

        let bitmap_properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: std::mem::ManuallyDrop::new(None),
        };

        let bitmap: ID2D1Bitmap1 = unsafe {
            self.device_context.CreateBitmapFromDxgiSurface(&backbuffer, Some(&bitmap_properties))?
        };

        unsafe {
            self.device_context.SetTarget(&bitmap);
        }

        Ok(())
    }

    /// Get device context for drawing
    pub fn device_context(&self) -> &ID2D1DeviceContext {
        &self.device_context
    }

    /// Get DirectWrite factory
    pub fn dwrite_factory(&self) -> &IDWriteFactory {
        &self.dwrite_factory
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// T333: Mark renderer as needing update (dirty)
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// T333: Check if rendering is needed (throttled to 120 FPS)
    pub fn should_render(&mut self) -> bool {
        const TARGET_FRAME_TIME: std::time::Duration = std::time::Duration::from_micros(8333); // 120 FPS
        
        if !self.dirty {
            return false;
        }

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_frame_time);
        
        if elapsed >= TARGET_FRAME_TIME {
            self.last_frame_time = now;
            self.dirty = false;
            true
        } else {
            false
        }
    }

    /// T334: Create command list for static UI caching
    pub fn create_static_ui_cache(&mut self) -> Result<()> {
        unsafe {
            let command_list = self.device_context.CreateCommandList()?;
            self.static_ui_cache = Some(command_list);
            self.cache_valid = false;
        }
        Ok(())
    }

    /// T334: Begin recording static UI into command list
    pub fn begin_static_ui_recording(&mut self) -> Result<()> {
        if self.static_ui_cache.is_none() {
            self.create_static_ui_cache()?;
        }

        if let Some(ref command_list) = self.static_ui_cache {
            unsafe {
                self.device_context.SetTarget(command_list);
            }
        }
        Ok(())
    }

    /// T334: End recording and mark cache as valid
    pub fn end_static_ui_recording(&mut self) -> Result<()> {
        if let Some(ref command_list) = self.static_ui_cache {
            unsafe {
                command_list.Close()?;
            }
            self.cache_valid = true;

            // Reset target to bitmap
            unsafe {
                self.device_context.SetTarget(&self.bitmap);
            }
        }
        Ok(())
    }

    /// T334: Draw cached static UI (much faster than re-rendering)
    pub fn draw_static_ui_cache(&self) {
        if self.cache_valid {
            if let Some(ref command_list) = self.static_ui_cache {
                unsafe {
                    self.device_context.DrawImage(
                        command_list,
                        None,
                        None,
                        D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                        D2D1_COMPOSITE_MODE_SOURCE_OVER,
                    );
                }
            }
        }
    }

    /// T334: Invalidate static UI cache (call when layout changes)
    pub fn invalidate_static_cache(&mut self) {
        self.cache_valid = false;
        self.dirty = true;
    }

    /// T335: Batch multiple rectangles into single geometry (reduce draw calls)
    pub fn draw_rectangles_batched(&self, rects: &[D2D_RECT_F], brush: &ID2D1Brush) -> Result<()> {
        // For small batches, individual calls are fine
        if rects.len() <= 3 {
            for rect in rects {
                unsafe {
                    self.device_context.FillRectangle(rect, brush);
                }
            }
            return Ok(());
        }

        // For larger batches, use geometry group for single draw call
        unsafe {
            let mut geometries: Vec<Option<ID2D1Geometry>> = Vec::with_capacity(rects.len());
            
            for rect in rects {
                let rect_geom = self.d2d_factory.CreateRectangleGeometry(rect)?;
                geometries.push(Some(rect_geom.cast()?));
            }

            let geometry_group = self.d2d_factory.CreateGeometryGroup(
                D2D1_FILL_MODE_WINDING,
                &geometries,
            )?;

            self.device_context.FillGeometry(&geometry_group, brush, None);
        }

        Ok(())
    }

    /// T336: Create and cache a layer for expensive effects (shadows, blurs)
    pub fn create_cached_layer(&self, width: f32, height: f32) -> Result<ID2D1Layer> {
        unsafe {
            let size = D2D_SIZE_F { width, height };
            self.device_context.CreateLayer(Some(&size))
        }
    }

    /// T336: Push layer with caching (reuse across frames)
    pub fn push_layer(&self, layer: &ID2D1Layer, params: &D2D1_LAYER_PARAMETERS1) {
        unsafe {
            self.device_context.PushLayer(params as *const _, layer);
        }
    }

    /// T336: Pop layer
    pub fn pop_layer(&self) {
        unsafe {
            self.device_context.PopLayer();
        }
    }
    
    /// T337: Check if rectangle is visible (occlusion culling)
    /// Returns false if rect is completely outside the viewport
    pub fn is_visible(&self, rect: &D2D_RECT_F) -> bool {
        let viewport_width = self.width as f32;
        let viewport_height = self.height as f32;
        
        // Check if rectangle intersects viewport
        !(rect.right < 0.0 || 
          rect.left > viewport_width || 
          rect.bottom < 0.0 || 
          rect.top > viewport_height)
    }
    
    /// T337: Check if element is visible with scroll offset
    pub fn is_visible_with_scroll(&self, rect: &D2D_RECT_F, scroll_x: f32, scroll_y: f32) -> bool {
        let adjusted_rect = D2D_RECT_F {
            left: rect.left - scroll_x,
            top: rect.top - scroll_y,
            right: rect.right - scroll_x,
            bottom: rect.bottom - scroll_y,
        };
        self.is_visible(&adjusted_rect)
    }
}
