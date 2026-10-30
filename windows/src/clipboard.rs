//! The Windows clipboard and libghostty's clipboard callbacks.
//!
//! The callbacks run while libghostty is processing, so anything that
//! completes a request or shows a prompt is posted to the window and
//! finished from its message handler instead.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr::{copy_nonoverlapping, null};

use windows_sys::Win32::Foundation::{GlobalFree, HANDLE, HWND};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

use crate::ffi::*;
use crate::util::confirm;
use crate::window::Window;

pub const WM_APP_CLIPBOARD_READ: u32 = WM_APP + 20;
pub const WM_APP_CLIPBOARD_CONFIRM: u32 = WM_APP + 21;
pub const WM_APP_CLIPBOARD_WRITE: u32 = WM_APP + 22;

const TEXT_PLAIN: &CStr = c"text/plain";

/// A confirmation request, copied out of the callback's borrowed memory.
pub struct PendingConfirm {
    state: *mut c_void,
    request: c_int,
    contents: Vec<(CString, Vec<u8>)>,
    available: Vec<CString>,
}

pub unsafe extern "C" fn read(
    userdata: *mut c_void,
    clipboard: c_int,
    state: *mut c_void,
    _mimes: *const *const c_char,
    _mimes_len: usize,
    _list: bool,
) -> c_int {
    let Some(window) = Window::from_userdata(userdata) else {
        return GHOSTTY_CLIPBOARD_READ_UNAVAILABLE;
    };
    if clipboard != GHOSTTY_CLIPBOARD_STANDARD {
        return GHOSTTY_CLIPBOARD_READ_UNSUPPORTED;
    }
    PostMessageW(window.hwnd(), WM_APP_CLIPBOARD_READ, state as usize, 0);
    GHOSTTY_CLIPBOARD_READ_STARTED
}

/// Serve a read request with the clipboard's text, if any.
pub unsafe fn complete_read(window: &Window, state: *mut c_void) {
    let surface = window.surface();
    if surface.is_null() {
        return;
    }

    let text = get_text(window.hwnd());
    let content = text.as_ref().map(|text| ghostty_clipboard_content_s {
        mime: TEXT_PLAIN.as_ptr(),
        data: text.as_ptr().cast(),
        len: text.len(),
    });
    let available = [TEXT_PLAIN.as_ptr()];
    let complete = ghostty_clipboard_complete_s {
        contents: content.as_ref().map_or(null(), |c| c as *const _),
        contents_len: content.is_some() as usize,
        available: available.as_ptr(),
        available_len: content.is_some() as usize,
        confirmed: false,
        remember: false,
    };
    ghostty_surface_complete_clipboard_request(surface, &complete, state);
}

pub unsafe extern "C" fn confirm_read(
    userdata: *mut c_void,
    confirm: *const ghostty_clipboard_confirm_s,
    state: *mut c_void,
    request: c_int,
) {
    let Some(window) = Window::from_userdata(userdata) else {
        return;
    };
    let confirm = &*confirm;

    let contents = (0..confirm.contents_len)
        .map(|i| {
            let c = &*confirm.contents.add(i);
            let data = std::slice::from_raw_parts(c.data as *const u8, c.len).to_vec();
            (CStr::from_ptr(c.mime).to_owned(), data)
        })
        .collect();
    let available = (0..confirm.available_len)
        .map(|i| CStr::from_ptr(*confirm.available.add(i)).to_owned())
        .collect();

    let pending = Box::new(PendingConfirm {
        state,
        request,
        contents,
        available,
    });
    PostMessageW(
        window.hwnd(),
        WM_APP_CLIPBOARD_CONFIRM,
        Box::into_raw(pending) as usize,
        0,
    );
}

/// Ask the user about a read that libghostty flagged, then complete or
/// deny it.
pub unsafe fn complete_confirm(window: &Window, pending: *mut PendingConfirm) {
    let pending = Box::from_raw(pending);
    let surface = window.surface();
    if surface.is_null() {
        return;
    }

    let question = match pending.request {
        GHOSTTY_CLIPBOARD_REQUEST_PASTE => {
            "The text you are pasting may be unsafe (for example, it contains newlines \
             that could run commands). Paste anyway?"
        }
        _ => "A program in this terminal wants to read the clipboard. Allow it?",
    };
    if !confirm(window.hwnd(), question) {
        ghostty_surface_deny_clipboard_request(surface, pending.state);
        return;
    }

    let contents: Vec<ghostty_clipboard_content_s> = pending
        .contents
        .iter()
        .map(|(mime, data)| ghostty_clipboard_content_s {
            mime: mime.as_ptr(),
            data: data.as_ptr().cast(),
            len: data.len(),
        })
        .collect();
    let available: Vec<*const c_char> = pending.available.iter().map(|m| m.as_ptr()).collect();
    let complete = ghostty_clipboard_complete_s {
        contents: contents.as_ptr(),
        contents_len: contents.len(),
        available: available.as_ptr(),
        available_len: available.len(),
        confirmed: true,
        remember: false,
    };
    ghostty_surface_complete_clipboard_request(surface, &complete, pending.state);
}

pub unsafe extern "C" fn write(
    userdata: *mut c_void,
    clipboard: c_int,
    contents: *const ghostty_clipboard_content_s,
    len: usize,
    confirm: bool,
) {
    let Some(window) = Window::from_userdata(userdata) else {
        return;
    };
    if clipboard != GHOSTTY_CLIPBOARD_STANDARD {
        return;
    }

    let text = (0..len).map(|i| &*contents.add(i)).find_map(|c| {
        let mime = CStr::from_ptr(c.mime).to_bytes();
        mime.starts_with(b"text/plain").then(|| {
            let data = std::slice::from_raw_parts(c.data as *const u8, c.len);
            String::from_utf8_lossy(data).into_owned()
        })
    });
    let Some(text) = text else {
        return;
    };

    if confirm {
        let text = Box::into_raw(Box::new(text));
        PostMessageW(window.hwnd(), WM_APP_CLIPBOARD_WRITE, text as usize, 0);
    } else {
        set_text(window.hwnd(), &text);
    }
}

/// Ask before letting a program (e.g. via OSC 52) set the clipboard.
pub unsafe fn complete_write(window: &Window, text: *mut String) {
    let text = Box::from_raw(text);
    if confirm(
        window.hwnd(),
        "A program in this terminal wants to set the clipboard. Allow it?",
    ) {
        set_text(window.hwnd(), &text);
    }
}

unsafe fn get_text(hwnd: HWND) -> Option<String> {
    if OpenClipboard(hwnd) == 0 {
        return None;
    }
    let mut text = None;
    let handle = GetClipboardData(CF_UNICODETEXT as u32);
    if !handle.is_null() {
        let ptr = GlobalLock(handle) as *const u16;
        if !ptr.is_null() {
            let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
            text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(
                ptr, len,
            )));
            GlobalUnlock(handle);
        }
    }
    CloseClipboard();
    text
}

unsafe fn set_text(hwnd: HWND, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    if OpenClipboard(hwnd) == 0 {
        return;
    }
    EmptyClipboard();
    let handle = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);
    if !handle.is_null() {
        let ptr = GlobalLock(handle) as *mut u16;
        if !ptr.is_null() {
            copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            GlobalUnlock(handle);
            // On success the clipboard owns the memory.
            if SetClipboardData(CF_UNICODETEXT as u32, handle as HANDLE).is_null() {
                GlobalFree(handle);
            }
        } else {
            GlobalFree(handle);
        }
    }
    CloseClipboard();
}
