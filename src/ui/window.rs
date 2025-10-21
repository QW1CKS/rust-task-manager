//! Win32 window management

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HBRUSH, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::util::strings::to_wide_string;

/// Window class name
const WINDOW_CLASS_NAME: &str = "RustTaskManagerWindow";

/// Main application window
pub struct Window {
    hwnd: HWND,
    title: String,
    width: i32,
    height: i32,
}

impl Window {
    /// Create a new window
    ///
    /// # Arguments
    /// * `title` - Window title
    /// * `width` - Initial width
    /// * `height` - Initial height
    pub fn new(title: &str, width: i32, height: i32) -> Result<Self> {
        // Register window class
        Self::register_window_class()?;

        let window_title = to_wide_string(title);
        let class_name = to_wide_string(WINDOW_CLASS_NAME);

        // Create window
        // SAFETY: CreateWindowExW is safe to call with valid parameters
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(window_title.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                GetModuleHandleW(None).ok().map(|h| HINSTANCE(h.0)),
                None,
            )?
        };

        Ok(Self {
            hwnd,
            title: title.to_string(),
            width,
            height,
        })
    }

    /// Register the window class
    fn register_window_class() -> Result<()> {
        let class_name = to_wide_string(WINDOW_CLASS_NAME);

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: unsafe { GetModuleHandleW(None).ok().unwrap_or_default() }.into(),
            hIcon: HICON::default(),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: HICON::default(),
        };

        // SAFETY: RegisterClassExW is safe with valid WNDCLASSEXW
        let atom = unsafe { RegisterClassExW(&wc) };

        if atom == 0 {
            // Class might already be registered, which is okay
            Err(Error::from_thread())
        } else {
            Ok(())
        }
    }

    /// Window procedure (message handler)
    ///
    /// # Safety
    /// This is called by Windows with valid HWND and message parameters
    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DESTROY => {
                // SAFETY: PostQuitMessage is safe to call
                unsafe {
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                // SAFETY: DestroyWindow is safe with valid HWND
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                // TODO: Phase 2 - Add Direct2D rendering here
                let mut ps = PAINTSTRUCT::default();
                // SAFETY: BeginPaint/EndPaint are safe with valid HWND
                unsafe {
                    let _hdc = BeginPaint(hwnd, &mut ps);
                    // Placeholder: solid background
                    let _ = EndPaint(hwnd, &ps);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                // TODO: Phase 2 - Handle resize for Direct2D swap chain
                LRESULT(0)
            }
            WM_DPICHANGED => {
                // TODO: Phase 2 - Handle DPI change
                // Get suggested window rect from LPARAM
                // SAFETY: LPARAM contains pointer to RECT
                unsafe {
                    let rect = lparam.0 as *const windows::Win32::Foundation::RECT;
                    if !rect.is_null() {
                        let r = *rect;
                        SetWindowPos(
                            hwnd,
                            None,
                            r.left,
                            r.top,
                            r.right - r.left,
                            r.bottom - r.top,
                            SWP_NOZORDER | SWP_NOACTIVATE,
                        )
                        .ok();
                    }
                }
                LRESULT(0)
            }
            _ => {
                // SAFETY: DefWindowProcW is safe to call
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
        }
    }

    /// Run the message loop
    ///
    /// Returns when WM_QUIT is received
    pub fn run_message_loop(&self) -> Result<()> {
        let mut msg = MSG::default();

        // SAFETY: GetMessageW is safe to call
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }

    /// Get window handle
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Get window title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get window dimensions
    pub fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Show the window
    pub fn show(&self) {
        // SAFETY: ShowWindow is safe with valid HWND
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
        }
    }
}
