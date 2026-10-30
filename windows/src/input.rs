//! Translates Win32 keyboard and mouse messages into libghostty input.
//!
//! Key events carry the hardware scan code as libghostty's native
//! keycode (see src/input/keycodes.zig, the "Win" column) along with
//! the text the key produces, computed here with ToUnicode. That is why
//! the message loop doesn't call TranslateMessage.

use std::ffi::{c_int, CString};
use std::ptr::null;

use windows_sys::Win32::Foundation::{LPARAM, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::ffi::*;
use crate::window::Window;

/// ToUnicode flag to leave the keyboard state (e.g. a pending dead key)
/// untouched. Available since Windows 10 1607.
const TOUNICODE_NO_STATE_CHANGE: u32 = 0x4;

/// The modifiers currently held, in libghostty's representation.
pub fn mods() -> c_int {
    let down = |vk: VIRTUAL_KEY| unsafe { GetKeyState(vk as i32) } < 0;
    let toggled = |vk: VIRTUAL_KEY| unsafe { GetKeyState(vk as i32) } & 1 != 0;

    let mut mods = GHOSTTY_MODS_NONE;
    let mut side = |left: VIRTUAL_KEY, right: VIRTUAL_KEY, mod_: c_int, right_mod: c_int| {
        if down(left) || down(right) {
            mods |= mod_;
            if down(right) && !down(left) {
                mods |= right_mod;
            }
        }
    };
    side(
        VK_LSHIFT,
        VK_RSHIFT,
        GHOSTTY_MODS_SHIFT,
        GHOSTTY_MODS_SHIFT_RIGHT,
    );
    side(
        VK_LCONTROL,
        VK_RCONTROL,
        GHOSTTY_MODS_CTRL,
        GHOSTTY_MODS_CTRL_RIGHT,
    );
    side(VK_LMENU, VK_RMENU, GHOSTTY_MODS_ALT, GHOSTTY_MODS_ALT_RIGHT);
    side(
        VK_LWIN,
        VK_RWIN,
        GHOSTTY_MODS_SUPER,
        GHOSTTY_MODS_SUPER_RIGHT,
    );
    if toggled(VK_CAPITAL) {
        mods |= GHOSTTY_MODS_CAPS;
    }
    if toggled(VK_NUMLOCK) {
        mods |= GHOSTTY_MODS_NUM;
    }
    mods
}

enum Translated {
    None,
    Text(String),
    Dead(String),
}

/// Handle a WM_(SYS)KEY(DOWN|UP). Returns true if libghostty consumed it.
pub unsafe fn key(window: &Window, msg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
    let surface = window.surface();
    if surface.is_null() {
        return false;
    }

    // The IME is handling this key and delivers its result separately.
    let vk = wparam as u32;
    if vk == VK_PROCESSKEY as u32 {
        return false;
    }

    let scancode = ((lparam >> 16) & 0xff) as u32;
    let extended = (lparam >> 24) & 1 != 0;
    let released = msg == WM_KEYUP || msg == WM_SYSKEYUP;
    let repeat = !released && (lparam >> 30) & 1 != 0;

    let mods = mods();
    // Windows reports AltGr as LeftCtrl + RightAlt.
    let altgr = mods & GHOSTTY_MODS_CTRL != 0 && GetKeyState(VK_RMENU as i32) < 0;
    let unshifted = unshifted_codepoint(vk, scancode);

    let mut text: Option<CString> = None;
    let mut composing = false;
    let mut consumed_mods = GHOSTTY_MODS_NONE;
    if !released {
        // Any new key ends a dead key's preedit.
        if window.preedit().replace(false) {
            ghostty_surface_preedit(surface, null(), 0);
        }

        match translate(vk, scancode, altgr) {
            Translated::None => {}
            Translated::Dead(s) => {
                composing = true;
                ghostty_surface_preedit(surface, s.as_ptr().cast(), s.len());
                window.preedit().set(true);
            }
            Translated::Text(s) => {
                let first = s.chars().next().map_or(0, |c| c as u32);
                if mods & GHOSTTY_MODS_SHIFT != 0 && first != unshifted {
                    consumed_mods |= GHOSTTY_MODS_SHIFT;
                }
                if altgr {
                    consumed_mods |= GHOSTTY_MODS_CTRL | GHOSTTY_MODS_ALT;
                }
                text = CString::new(s).ok();
            }
        }
    }

    let event = ghostty_input_key_s {
        action: if released {
            GHOSTTY_ACTION_RELEASE
        } else if repeat {
            GHOSTTY_ACTION_REPEAT
        } else {
            GHOSTTY_ACTION_PRESS
        },
        mods,
        consumed_mods,
        keycode: if extended {
            0xe000 | scancode
        } else {
            scancode
        },
        text: text.as_ref().map_or(null(), |t| t.as_ptr()),
        unshifted_codepoint: unshifted,
        composing,
    };
    ghostty_surface_key(surface, event)
}

/// The text a key press produces. Ctrl and Alt are removed first (unless
/// they form AltGr) so that e.g. Ctrl+C reports "c" and libghostty does
/// the control encoding, like the other apprts.
unsafe fn translate(vk: u32, scancode: u32, altgr: bool) -> Translated {
    let mut state = [0u8; 256];
    if GetKeyboardState(state.as_mut_ptr()) == 0 {
        return Translated::None;
    }
    if !altgr {
        for vk in [
            VK_CONTROL,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_MENU,
            VK_LMENU,
            VK_RMENU,
        ] {
            state[vk as usize] = 0;
        }
    }

    let mut buf = [0u16; 16];
    let n = ToUnicode(
        vk,
        scancode,
        state.as_ptr(),
        buf.as_mut_ptr(),
        buf.len() as i32,
        0,
    );
    if n < 0 {
        return Translated::Dead(String::from_utf16_lossy(&buf[..1]));
    }

    // Control characters (Enter, Tab, Backspace, Escape...) are encoded
    // by libghostty from the physical key instead.
    let text: String = String::from_utf16_lossy(&buf[..n as usize])
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    if text.is_empty() {
        Translated::None
    } else {
        Translated::Text(text)
    }
}

/// The codepoint the key produces with no modifiers held.
unsafe fn unshifted_codepoint(vk: u32, scancode: u32) -> u32 {
    let state = [0u8; 256];
    let mut buf = [0u16; 4];
    let n = ToUnicode(
        vk,
        scancode,
        state.as_ptr(),
        buf.as_mut_ptr(),
        buf.len() as i32,
        TOUNICODE_NO_STATE_CHANGE,
    );
    match char::from_u32(buf[0] as u32) {
        Some(c) if n != 0 && !c.is_control() => c as u32,
        _ => 0,
    }
}

fn point(lparam: LPARAM) -> (f64, f64) {
    (
        (lparam & 0xffff) as i16 as f64,
        ((lparam >> 16) & 0xffff) as i16 as f64,
    )
}

/// libghostty expects positions in logical pixels; Windows gives us
/// physical ones since we are per-monitor DPI aware.
pub unsafe fn mouse_pos(window: &Window, lparam: LPARAM) {
    let surface = window.surface();
    if surface.is_null() {
        return;
    }
    let (x, y) = point(lparam);
    let scale = window.scale();
    ghostty_surface_mouse_pos(surface, x / scale, y / scale, mods());
}

pub unsafe fn mouse_button(window: &Window, msg: u32, wparam: WPARAM, lparam: LPARAM) {
    let surface = window.surface();
    if surface.is_null() {
        return;
    }

    let (state, button) = match msg {
        WM_LBUTTONDOWN => (GHOSTTY_MOUSE_PRESS, GHOSTTY_MOUSE_LEFT),
        WM_LBUTTONUP => (GHOSTTY_MOUSE_RELEASE, GHOSTTY_MOUSE_LEFT),
        WM_RBUTTONDOWN => (GHOSTTY_MOUSE_PRESS, GHOSTTY_MOUSE_RIGHT),
        WM_RBUTTONUP => (GHOSTTY_MOUSE_RELEASE, GHOSTTY_MOUSE_RIGHT),
        WM_MBUTTONDOWN => (GHOSTTY_MOUSE_PRESS, GHOSTTY_MOUSE_MIDDLE),
        WM_MBUTTONUP => (GHOSTTY_MOUSE_RELEASE, GHOSTTY_MOUSE_MIDDLE),
        WM_XBUTTONDOWN | WM_XBUTTONUP => {
            let state = if msg == WM_XBUTTONDOWN {
                GHOSTTY_MOUSE_PRESS
            } else {
                GHOSTTY_MOUSE_RELEASE
            };
            let button = if (wparam >> 16) & 0xffff == XBUTTON1 as usize {
                GHOSTTY_MOUSE_FOUR
            } else {
                GHOSTTY_MOUSE_FIVE
            };
            (state, button)
        }
        _ => return,
    };

    // Keep receiving mouse events while a button is held outside the window.
    const ANY_BUTTON: WPARAM = 0x0001 | 0x0002 | 0x0010 | 0x0020 | 0x0040;
    if state == GHOSTTY_MOUSE_PRESS {
        SetCapture(window.hwnd());
    } else if wparam & ANY_BUTTON == 0 {
        ReleaseCapture();
    }

    mouse_pos(window, lparam);
    ghostty_surface_mouse_button(surface, state, button, mods());
}

/// Positive y scrolls up and positive x scrolls left, as in the GTK apprt.
pub unsafe fn scroll(window: &Window, msg: u32, wparam: WPARAM) {
    let surface = window.surface();
    if surface.is_null() {
        return;
    }
    let delta = ((wparam >> 16) & 0xffff) as i16 as f64 / WHEEL_DELTA as f64;
    let (x, y) = if msg == WM_MOUSEHWHEEL {
        (-delta, 0.0)
    } else {
        (0.0, delta)
    };
    ghostty_surface_mouse_scroll(surface, x, y, 0);
}
