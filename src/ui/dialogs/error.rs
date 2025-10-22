// Error dialog implementation (T472)
//
// Uses MessageBoxW for critical error notifications

use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONWARNING, MB_ICONINFORMATION, MB_OK, MB_OKCANCEL, MB_YESNO,
    IDOK, IDCANCEL, IDYES, IDNO,
};

/// Error severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Critical error that requires application termination
    Critical,
    /// Warning that user should be aware of
    Warning,
    /// Informational message
    Info,
}

/// User response to dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResponse {
    Ok,
    Cancel,
    Yes,
    No,
}

/// Show error dialog to user (T472)
pub fn show_error(hwnd: HWND, title: &str, message: &str, severity: ErrorSeverity) -> DialogResponse {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    
    let icon = match severity {
        ErrorSeverity::Critical => MB_ICONERROR,
        ErrorSeverity::Warning => MB_ICONWARNING,
        ErrorSeverity::Info => MB_ICONINFORMATION,
    };
    
    unsafe {
        let result = MessageBoxW(
            Some(hwnd),
            PCWSTR(message_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            icon | MB_OK,
        );
        
        if result == IDOK {
            DialogResponse::Ok
        } else {
            DialogResponse::Ok
        }
    }
}

/// Show confirmation dialog (T472)
pub fn show_confirmation(hwnd: HWND, title: &str, message: &str) -> DialogResponse {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    
    unsafe {
        let result = MessageBoxW(
            Some(hwnd),
            PCWSTR(message_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OKCANCEL | MB_ICONWARNING,
        );
        
        if result == IDOK {
            DialogResponse::Ok
        } else if result == IDCANCEL {
            DialogResponse::Cancel
        } else {
            DialogResponse::Cancel
        }
    }
}

/// Show yes/no dialog (T472)
pub fn show_yes_no(hwnd: HWND, title: &str, message: &str) -> DialogResponse {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    
    unsafe {
        let result = MessageBoxW(
            Some(hwnd),
            PCWSTR(message_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_YESNO | MB_ICONWARNING,
        );
        
        if result == IDYES {
            DialogResponse::Yes
        } else if result == IDNO {
            DialogResponse::No
        } else {
            DialogResponse::No
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_variants() {
        let severities = [
            ErrorSeverity::Critical,
            ErrorSeverity::Warning,
            ErrorSeverity::Info,
        ];
        assert_eq!(severities.len(), 3);
    }

    #[test]
    fn test_dialog_response_variants() {
        let responses = [
            DialogResponse::Ok,
            DialogResponse::Cancel,
            DialogResponse::Yes,
            DialogResponse::No,
        ];
        assert_eq!(responses.len(), 4);
    }
}
