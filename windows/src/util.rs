//! Small Win32 helpers.

use std::ffi::{c_char, CStr};
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_YESNO,
};

/// Encode a string as a null-terminated UTF-16 buffer.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Read a borrowed C string from libghostty, which is always UTF-8.
pub unsafe fn cstr<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

pub fn error_box(message: &str) {
    let text = wide(message);
    let title = wide("Ghostty");
    unsafe {
        MessageBoxW(
            null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

/// Ask a yes/no question, returning true for yes.
pub fn confirm(owner: HWND, message: &str) -> bool {
    let text = wide(message);
    let title = wide("Ghostty");
    unsafe {
        MessageBoxW(
            owner,
            text.as_ptr(),
            title.as_ptr(),
            MB_YESNO | MB_ICONWARNING,
        ) == IDYES
    }
}
