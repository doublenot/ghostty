//! Process-wide state, startup, and the libghostty runtime callbacks.

use std::cell::{Cell, RefCell};
use std::ffi::{c_char, c_void, CString};
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostMessageW,
    PostQuitMessage, RegisterClassExW, HWND_MESSAGE, MB_OK, MSG, SW_SHOWNORMAL, WM_APP,
    WNDCLASSEXW,
};

use crate::ffi::*;
use crate::util::{confirm, cstr, error_box, wide};
use crate::{clipboard, window};

const WM_APP_WAKEUP: u32 = WM_APP + 1;
const WM_APP_QUIT: u32 = WM_APP + 2;
const WM_APP_NEW_WINDOW: u32 = WM_APP + 3;

/// State shared by every window. Only touched on the main thread.
pub struct App {
    pub app: Cell<ghostty_app_t>,
    pub windows: RefCell<Vec<HWND>>,
}

static APP: AtomicPtr<App> = AtomicPtr::new(null_mut());

/// The message-only window that wakeups are posted to. Kept apart from
/// `App` since wakeups arrive on arbitrary threads. Window messages are
/// used rather than thread messages because they are still dispatched
/// inside modal loops, e.g. while the user drags a window border.
static WAKEUP_HWND: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

/// Coalesces wakeups so that a burst of them only ticks the app once.
static WAKEUP_PENDING: AtomicBool = AtomicBool::new(false);

pub fn get() -> &'static App {
    unsafe { &*APP.load(Ordering::Acquire) }
}

pub fn run() -> i32 {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        // libghostty parses CLI arguments (e.g. --font-size=14) from argv.
        let args: Vec<CString> = std::env::args()
            .map(|arg| CString::new(arg).unwrap_or_default())
            .collect();
        let mut argv: Vec<*mut c_char> =
            args.iter().map(|arg| arg.as_ptr() as *mut c_char).collect();
        if ghostty_init(argv.len(), argv.as_mut_ptr()) != GHOSTTY_SUCCESS {
            error_box("Failed to initialize libghostty.");
            return 1;
        }

        let config = ghostty_config_new();
        ghostty_config_load_default_files(config);
        ghostty_config_load_cli_args(config);
        ghostty_config_load_recursive_files(config);
        ghostty_config_finalize(config);
        for i in 0..ghostty_config_diagnostics_count(config) {
            let diagnostic = ghostty_config_get_diagnostic(config, i);
            eprintln!(
                "config: {}",
                cstr(diagnostic.message).unwrap_or("(invalid)")
            );
        }

        window::register_class();
        WAKEUP_HWND.store(create_message_window(), Ordering::Release);

        let app: &'static App = Box::leak(Box::new(App {
            app: Cell::new(null_mut()),
            windows: RefCell::new(Vec::new()),
        }));
        APP.store(app as *const App as *mut App, Ordering::Release);

        let runtime = ghostty_runtime_config_s {
            userdata: null_mut(),
            supports_selection_clipboard: false,
            wakeup_cb: Some(wakeup),
            action_cb: Some(action),
            read_clipboard_cb: Some(clipboard::read),
            confirm_read_clipboard_cb: Some(clipboard::confirm_read),
            write_clipboard_cb: Some(clipboard::write),
            close_surface_cb: Some(window::close_surface),
        };
        let ghostty_app = ghostty_app_new(&runtime, config);
        if ghostty_app.is_null() {
            error_box("Failed to create the libghostty app.");
            return 1;
        }
        app.app.set(ghostty_app);

        if !window::Window::create(None) {
            error_box("Failed to create a terminal window.");
            return 1;
        }

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            // We deliberately skip TranslateMessage: key events carry
            // their own text (see input.rs), so WM_CHAR is not needed.
            DispatchMessageW(&msg);
        }

        ghostty_app_free(ghostty_app);
        ghostty_config_free(config);
        msg.wParam as i32
    }
}

unsafe fn create_message_window() -> HWND {
    let class_name = wide("GhosttyApp");
    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(message_wndproc),
        hInstance: GetModuleHandleW(null()),
        lpszClassName: class_name.as_ptr(),
        ..zeroed()
    };
    RegisterClassExW(&wc);
    CreateWindowExW(
        0,
        class_name.as_ptr(),
        null(),
        0,
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        null_mut(),
        wc.hInstance,
        null(),
    )
}

unsafe extern "system" fn message_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_APP_WAKEUP => {
            WAKEUP_PENDING.store(false, Ordering::Release);
            ghostty_app_tick(get().app.get());
            0
        }
        WM_APP_QUIT => {
            quit();
            0
        }
        WM_APP_NEW_WINDOW => {
            let config = wparam as *mut ghostty_surface_config_s;
            let config = (!config.is_null()).then(|| *Box::from_raw(config));
            window::Window::create(config);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Close every window, asking first if any has a running process.
fn quit() {
    let windows = get().windows.borrow().clone();
    let needs_confirm = windows.iter().any(|&hwnd| {
        window::Window::from_hwnd(hwnd)
            .is_some_and(|w| unsafe { ghostty_surface_needs_confirm_quit(w.surface()) })
    });
    if needs_confirm && !confirm(null_mut(), "Quit Ghostty? Processes are still running.") {
        return;
    }
    for hwnd in windows {
        unsafe { DestroyWindow(hwnd) };
    }
    if get().windows.borrow().is_empty() {
        unsafe { PostQuitMessage(0) };
    }
}

unsafe extern "C" fn wakeup(_userdata: *mut c_void) {
    if !WAKEUP_PENDING.swap(true, Ordering::AcqRel) {
        PostMessageW(WAKEUP_HWND.load(Ordering::Acquire), WM_APP_WAKEUP, 0, 0);
    }
}

/// Handles actions from libghostty. This is called while libghostty is
/// processing (e.g. inside `ghostty_app_tick`), so anything that would
/// re-enter it, like creating or destroying surfaces, is posted instead.
unsafe extern "C" fn action(
    _app: ghostty_app_t,
    target: ghostty_target_s,
    action: ghostty_action_s,
) -> bool {
    let surface = (target.tag == GHOSTTY_TARGET_SURFACE).then_some(target.target.surface);
    let window = surface.and_then(window::Window::from_surface);
    let wakeup_hwnd = WAKEUP_HWND.load(Ordering::Acquire);

    match action.tag {
        GHOSTTY_ACTION_QUIT | GHOSTTY_ACTION_CLOSE_ALL_WINDOWS => {
            PostMessageW(wakeup_hwnd, WM_APP_QUIT, 0, 0);
            true
        }

        GHOSTTY_ACTION_NEW_WINDOW => {
            let config = match surface {
                Some(surface) => Box::into_raw(Box::new(ghostty_surface_inherited_config(
                    surface,
                    GHOSTTY_SURFACE_CONTEXT_WINDOW,
                ))),
                None => null_mut(),
            };
            PostMessageW(wakeup_hwnd, WM_APP_NEW_WINDOW, config as WPARAM, 0);
            true
        }

        GHOSTTY_ACTION_RENDER => window.map(|w| w.invalidate()).is_some(),

        GHOSTTY_ACTION_SET_TITLE => match (window, cstr(action.action.set_title.title)) {
            (Some(w), Some(title)) => {
                w.set_title(title);
                true
            }
            _ => false,
        },

        GHOSTTY_ACTION_MOUSE_SHAPE => window
            .map(|w| w.set_mouse_shape(action.action.mouse_shape))
            .is_some(),

        GHOSTTY_ACTION_MOUSE_VISIBILITY => window
            .map(|w| w.set_cursor_visible(action.action.mouse_visibility == GHOSTTY_MOUSE_VISIBLE))
            .is_some(),

        GHOSTTY_ACTION_INITIAL_SIZE => {
            let size = action.action.initial_size;
            window
                .map(|w| w.set_client_size(size.width, size.height))
                .is_some()
        }

        GHOSTTY_ACTION_CLOSE_WINDOW => window.map(|w| w.request_close()).is_some(),

        GHOSTTY_ACTION_TOGGLE_MAXIMIZE => window.map(|w| w.toggle_maximize()).is_some(),

        GHOSTTY_ACTION_TOGGLE_FULLSCREEN => window.map(|w| w.toggle_fullscreen()).is_some(),

        GHOSTTY_ACTION_RING_BELL => {
            MessageBeep(MB_OK);
            true
        }

        GHOSTTY_ACTION_OPEN_URL => {
            let url = action.action.open_url;
            if url.url.is_null() {
                return false;
            }
            let bytes = std::slice::from_raw_parts(url.url as *const u8, url.len);
            let url = wide(&String::from_utf8_lossy(bytes));
            let verb = wide("open");
            ShellExecuteW(
                null_mut(),
                verb.as_ptr(),
                url.as_ptr(),
                null(),
                null(),
                SW_SHOWNORMAL,
            );
            true
        }

        GHOSTTY_ACTION_RENDERER_HEALTH => {
            eprintln!("renderer health changed: {}", action.action.renderer_health);
            true
        }

        _ => false,
    }
}
