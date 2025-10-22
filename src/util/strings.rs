//! UTF-16 string utilities for Windows API interop
//!
//! Provides efficient string conversion and pooling for Windows APIs (T317, T324).

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::Arc;

/// Convert UTF-16 null-terminated string to Rust String
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is valid and points to a null-terminated UTF-16 string
/// - `ptr` remains valid for the duration of this function
/// - The string data at `ptr` is properly aligned for `u16`
pub unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    // T359: Validate pointer preconditions
    debug_assert!(!ptr.is_null(), "from_wide_ptr called with null pointer");
    debug_assert_eq!(ptr as usize % 2, 0, "pointer must be 2-byte aligned for u16");
    
    if ptr.is_null() {
        return String::new();
    }

    // SAFETY (T358): String traversal is safe because:
    // - Caller guarantees ptr is valid and null-terminated
    // - We iterate until null terminator (0) is found
    // - from_raw_parts creates slice with computed length
    // - String::from_utf16_lossy handles invalid UTF-16 gracefully
    unsafe {
        let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
        
        // T359: Validate reasonable string length
        debug_assert!(len < 32768, "suspiciously long string length {}", len);
        
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

/// String pool for interning process names (T317)
///
/// Most processes have common names (e.g., "svchost.exe", "System"),
/// so we can save memory by sharing string allocations.
pub struct StringPool {
    pool: RefCell<HashMap<String, Arc<str>>>,
    conversion_buffer: RefCell<Vec<u16>>,
}

impl StringPool {
    /// Creates a new string pool with common process names pre-allocated
    pub fn new() -> Self {
        let mut pool = HashMap::with_capacity(256);
        
        // Pre-populate with common Windows process names
        let common_names = [
            "System",
            "Registry",
            "smss.exe",
            "csrss.exe",
            "wininit.exe",
            "services.exe",
            "lsass.exe",
            "svchost.exe",
            "explorer.exe",
            "dwm.exe",
            "taskmgr.exe",
            "chrome.exe",
            "firefox.exe",
            "msedge.exe",
            "code.exe",
            "SearchApp.exe",
            "StartMenuExperienceHost.exe",
            "RuntimeBroker.exe",
            "ApplicationFrameHost.exe",
            "SystemSettings.exe",
        ];

        for name in &common_names {
            let arc: Arc<str> = Arc::from(*name);
            pool.insert(name.to_string(), arc);
        }

        Self {
            pool: RefCell::new(pool),
            conversion_buffer: RefCell::new(Vec::with_capacity(512)),
        }
    }

    /// Interns a string, returning a shared reference
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut pool = self.pool.borrow_mut();
        
        if let Some(existing) = pool.get(s) {
            return Arc::clone(existing);
        }

        let arc: Arc<str> = Arc::from(s);
        pool.insert(s.to_string(), Arc::clone(&arc));
        arc
    }

    /// Interns a UTF-16 string, reusing conversion buffer (T324)
    ///
    /// # Safety
    ///
    /// The caller must ensure `ptr` is a valid null-terminated UTF-16 string.
    pub unsafe fn intern_wide(&self, ptr: *const u16) -> Arc<str> {
        if ptr.is_null() {
            return Arc::from("");
        }

        let mut buffer = self.conversion_buffer.borrow_mut();
        buffer.clear();

        // Read UTF-16 string into reusable buffer
        let mut offset = 0;
        loop {
            let ch = unsafe { *ptr.add(offset) };
            if ch == 0 {
                break;
            }
            buffer.push(ch);
            offset += 1;
        }

        // Convert to String
        let string = String::from_utf16_lossy(&buffer);
        
        // Release buffer borrow before calling intern
        drop(buffer);
        
        self.intern(&string)
    }

    /// Returns the number of unique strings in the pool
    pub fn len(&self) -> usize {
        self.pool.borrow().len()
    }

    /// Returns true if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.borrow().is_empty()
    }

    /// Clears all entries except common names
    pub fn clear_except_common(&mut self) {
        let mut pool = self.pool.borrow_mut();
        pool.retain(|_, v| Arc::strong_count(v) > 1);
    }
}

impl Default for StringPool {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    /// Thread-local string pool for zero-contention access
    static STRING_POOL: StringPool = StringPool::new();
}

/// Interns a string using the thread-local pool
pub fn intern(s: &str) -> Arc<str> {
    STRING_POOL.with(|pool| pool.intern(s))
}

/// Interns a UTF-16 string using the thread-local pool
///
/// # Safety
///
/// The caller must ensure `ptr` is a valid null-terminated UTF-16 string.
pub unsafe fn intern_wide(ptr: *const u16) -> Arc<str> {
    STRING_POOL.with(|pool| unsafe { pool.intern_wide(ptr) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool_deduplication() {
        let pool = StringPool::new();
        
        let s1 = pool.intern("test.exe");
        let s2 = pool.intern("test.exe");
        let s3 = pool.intern("other.exe");

        assert!(Arc::ptr_eq(&s1, &s2));
        assert!(!Arc::ptr_eq(&s1, &s3));
        assert_eq!(pool.len(), 22); // 20 common + test.exe + other.exe
    }

    #[test]
    fn test_common_names_prepopulated() {
        let pool = StringPool::new();
        
        let svchost = pool.intern("svchost.exe");
        assert_eq!(pool.len(), 20); // Should not add new entry
        assert_eq!(svchost.as_ref(), "svchost.exe");
    }

    #[test]
    fn test_intern_wide() {
        let pool = StringPool::new();
        
        let wide_str: Vec<u16> = "test.exe".encode_utf16().chain(std::iter::once(0)).collect();
        let result = unsafe { pool.intern_wide(wide_str.as_ptr()) };
        
        assert_eq!(result.as_ref(), "test.exe");
    }
}
