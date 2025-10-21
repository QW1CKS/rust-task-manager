//! UTF-16 string utilities for Windows API interop

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

/// Convert UTF-16 null-terminated string to Rust String
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is valid and points to a null-terminated UTF-16 string
/// - `ptr` remains valid for the duration of this function
/// - The string data at `ptr` is properly aligned for `u16`
pub unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    // SAFETY: Caller guarantees ptr is valid and null-terminated
    unsafe {
        let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

/// Convert Rust string to UTF-16 with null terminator
pub fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Convert UTF-16 buffer to OsString
pub fn from_wide_buf(buf: &[u16]) -> OsString {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    OsString::from_wide(&buf[..end])
}
