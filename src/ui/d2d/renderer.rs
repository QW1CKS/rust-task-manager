//! Direct2D renderer with hardware acceleration

use windows::core::{Interface, Result};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

/// Direct2D renderer
pub struct Renderer {
    // D2D Factory
    d2d_factory: ID2D1Factory1,

    // D3D11 Device
    d3d_device: ID3D11Device,

    // DXGI Device
    dxgi_device: IDXGIDevice,

    // D2D Device
    d2d_device: ID2D1Device,

    // D2D Device Context
    device_context: ID2D1DeviceContext,

    // DXGI Swap Chain
    swap_chain: IDXGISwapChain1,

    // DirectWrite Factory
    dwrite_factory: IDWriteFactory,

    // Window handle
    hwnd: HWND,

    // Window dimensions
    width: u32,
    height: u32,
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
            hwnd,
            width,
            height,
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
}
