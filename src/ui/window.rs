//! Win32 window management

use std::cell::RefCell;
use std::rc::Rc;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HBRUSH, PAINTSTRUCT, InvalidateRect};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::app::state::AppState;
use crate::util::strings::to_wide_string;

/// Window class name
const WINDOW_CLASS_NAME: &str = "RustTaskManagerWindow";

/// Main application window with integrated rendering and monitoring
pub struct Window {
    hwnd: HWND,
    title: String,
    width: i32,
    height: i32,
    state: Option<Rc<RefCell<AppState>>>,
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
            state: None, // Initialized after window creation
        })
    }
    
    /// Initialize the application state (must be called after window creation)
    pub fn initialize_state(&mut self) -> Result<()> {
        let app_state = AppState::new(self.hwnd, self.width as u32, self.height as u32)?;
        let state_rc = Rc::new(RefCell::new(app_state));
        
        // Store state pointer in window user data
        let state_ptr = Rc::into_raw(state_rc.clone()) as isize;
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, state_ptr);
        }
        
        self.state = Some(state_rc);
        
        // Initial data collection
        if let Some(state) = &self.state {
            state.borrow_mut().update().ok();
        }
        
        // Set up timer for periodic updates (1 second)
        unsafe {
            SetTimer(Some(self.hwnd), 1, 1000, None);
        }
        
        // Force initial paint
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
        
        Ok(())
    }
    
    /// Get app state from window user data
    unsafe fn get_state(hwnd: HWND) -> Option<Rc<RefCell<AppState>>> {
        unsafe {
            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if ptr == 0 {
                return None;
            }
            
            let state_rc = Rc::from_raw(ptr as *const RefCell<AppState>);
            let clone = state_rc.clone();
            // Put it back so we don't drop it (intentionally "leak" to keep alive)
            let _ = Rc::into_raw(state_rc);
            Some(clone)
        }
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
                unsafe {
                    let mut ps = PAINTSTRUCT::default();
                    let _hdc = BeginPaint(hwnd, &mut ps);
                    
                    // Render with Direct2D if state is initialized
                    if let Some(state) = Self::get_state(hwnd) {
                        if let Ok(mut state_mut) = state.try_borrow_mut() {
                            state_mut.render().ok();
                        }
                    }
                    
                    let _ = EndPaint(hwnd, &ps);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                unsafe {
                    let width = (lparam.0 & 0xFFFF) as u32;
                    let height = ((lparam.0 >> 16) & 0xFFFF) as u32;
                    
                    if let Some(state) = Self::get_state(hwnd) {
                        if let Ok(mut state_mut) = state.try_borrow_mut() {
                            state_mut.resize(width, height).ok();
                        }
                    }
                }
                LRESULT(0)
            }
            WM_TIMER => {
                unsafe {
                    // Update process data
                    if let Some(state) = Self::get_state(hwnd) {
                        if let Ok(mut state_mut) = state.try_borrow_mut() {
                            state_mut.update().ok();
                        }
                    }
                    
                    // Trigger repaint
                    let _ = InvalidateRect(Some(hwnd), None, false);
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                // F5 to manually refresh (VK_F5 = 0x74)
                if wparam.0 == 0x74 {
                    unsafe {
                        if let Some(state) = Self::get_state(hwnd) {
                            if let Ok(mut state_mut) = state.try_borrow_mut() {
                                state_mut.update().ok();
                            }
                        }
                        let _ = InvalidateRect(Some(hwnd), None, false);
                    }
                }
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
