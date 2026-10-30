//! A top-level Win32 window hosting one libghostty surface.

use std::cell::{Cell, RefCell};
use std::ffi::{c_int, c_void};
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, FillRect, GetMonitorInfoW, GetStockObject, InvalidateRect,
    MonitorFromWindow, SetDIBitsToDevice, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACK_BRUSH,
    DIB_RGB_COLORS, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_F4;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::app;
use crate::clipboard;
use crate::ffi::*;
use crate::input;
use crate::util::{confirm, wide};

/// Posted by `close_surface` with whether the process is still alive.
const WM_APP_CLOSE: u32 = WM_APP + 10;

/// Default client size for new windows, in logical pixels.
const DEFAULT_SIZE: (u32, u32) = (960, 600);

#[derive(Default)]
struct Frame {
    width: u32,
    height: u32,
    /// Bottom-up BGRA rows, ready for GDI.
    bgra: Vec<u8>,
}

pub struct Window {
    hwnd: Cell<HWND>,
    surface: Cell<ghostty_surface_t>,
    scale: Cell<f64>,
    /// The last frame we were handed, kept so we can repaint on expose.
    frame: RefCell<Frame>,
    cursor: Cell<HCURSOR>,
    cursor_visible: Cell<bool>,
    preedit: Cell<bool>,
    /// The placement to restore when leaving fullscreen.
    restore: Cell<Option<WINDOWPLACEMENT>>,
}

pub fn register_class() {
    let class_name = wide("GhosttyWindow");
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wndproc),
            hInstance: GetModuleHandleW(null()),
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        RegisterClassExW(&wc);
    }
}

impl Window {
    /// Create a window and its surface. `config` is an inherited config
    /// from another surface, or None for a fresh one.
    pub fn create(config: Option<ghostty_surface_config_s>) -> bool {
        let class_name = wide("GhosttyWindow");
        let title = wide("Ghostty");
        unsafe {
            let window = Box::into_raw(Box::new(Window {
                hwnd: Cell::new(null_mut()),
                surface: Cell::new(null_mut()),
                scale: Cell::new(1.0),
                frame: RefCell::new(Frame::default()),
                cursor: Cell::new(LoadCursorW(null_mut(), IDC_IBEAM)),
                cursor_visible: Cell::new(true),
                preedit: Cell::new(false),
                restore: Cell::new(None),
            }));

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null()),
                window as *const c_void,
            );
            if hwnd.is_null() {
                drop(Box::from_raw(window));
                return false;
            }

            // From here on the HWND owns `window` and frees it in
            // WM_NCDESTROY, so failures must go through DestroyWindow.
            let w = &*window;
            let scale = GetDpiForWindow(hwnd) as f64 / 96.0;
            w.scale.set(scale);
            w.set_client_size(DEFAULT_SIZE.0, DEFAULT_SIZE.1);

            let mut config = config.unwrap_or_else(|| ghostty_surface_config_new());
            config.platform_tag = GHOSTTY_PLATFORM_WINDOWS;
            config.platform = ghostty_platform_u {
                windows: ghostty_platform_windows_s { hwnd },
            };
            config.userdata = window as *mut c_void;
            config.scale_factor = scale;
            config.context = GHOSTTY_SURFACE_CONTEXT_WINDOW;

            let surface = ghostty_surface_new(app::get().app.get(), &config);
            if surface.is_null() {
                DestroyWindow(hwnd);
                return false;
            }
            w.surface.set(surface);
            app::get().windows.borrow_mut().push(hwnd);

            let mut client: RECT = zeroed();
            GetClientRect(hwnd, &mut client);
            ghostty_surface_set_content_scale(surface, scale, scale);
            ghostty_surface_set_size(surface, client.right as u32, client.bottom as u32);

            ShowWindow(hwnd, SW_SHOWDEFAULT);
            true
        }
    }

    pub fn from_hwnd(hwnd: HWND) -> Option<&'static Window> {
        unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Window).as_ref() }
    }

    pub fn from_surface(surface: ghostty_surface_t) -> Option<&'static Window> {
        unsafe { Self::from_userdata(ghostty_surface_userdata(surface)) }
    }

    pub fn from_userdata(userdata: *mut c_void) -> Option<&'static Window> {
        unsafe { (userdata as *const Window).as_ref() }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd.get()
    }

    /// The surface, or null before it is created and after it is freed.
    pub fn surface(&self) -> ghostty_surface_t {
        self.surface.get()
    }

    pub fn scale(&self) -> f64 {
        self.scale.get()
    }

    /// Whether a dead key is showing as preedit text.
    pub fn preedit(&self) -> &Cell<bool> {
        &self.preedit
    }

    pub fn invalidate(&self) {
        unsafe { InvalidateRect(self.hwnd(), null(), 0) };
    }

    pub fn set_title(&self, title: &str) {
        let title = wide(title);
        unsafe { SetWindowTextW(self.hwnd(), title.as_ptr()) };
    }

    pub fn set_mouse_shape(&self, shape: c_int) {
        let id = match shape {
            GHOSTTY_MOUSE_SHAPE_TEXT | GHOSTTY_MOUSE_SHAPE_VERTICAL_TEXT => IDC_IBEAM,
            GHOSTTY_MOUSE_SHAPE_POINTER | GHOSTTY_MOUSE_SHAPE_ALIAS => IDC_HAND,
            GHOSTTY_MOUSE_SHAPE_HELP => IDC_HELP,
            GHOSTTY_MOUSE_SHAPE_PROGRESS => IDC_APPSTARTING,
            GHOSTTY_MOUSE_SHAPE_WAIT => IDC_WAIT,
            GHOSTTY_MOUSE_SHAPE_CELL | GHOSTTY_MOUSE_SHAPE_CROSSHAIR => IDC_CROSS,
            GHOSTTY_MOUSE_SHAPE_MOVE
            | GHOSTTY_MOUSE_SHAPE_ALL_SCROLL
            | GHOSTTY_MOUSE_SHAPE_GRAB
            | GHOSTTY_MOUSE_SHAPE_GRABBING => IDC_SIZEALL,
            GHOSTTY_MOUSE_SHAPE_NO_DROP | GHOSTTY_MOUSE_SHAPE_NOT_ALLOWED => IDC_NO,
            GHOSTTY_MOUSE_SHAPE_COL_RESIZE
            | GHOSTTY_MOUSE_SHAPE_E_RESIZE
            | GHOSTTY_MOUSE_SHAPE_W_RESIZE
            | GHOSTTY_MOUSE_SHAPE_EW_RESIZE => IDC_SIZEWE,
            GHOSTTY_MOUSE_SHAPE_ROW_RESIZE
            | GHOSTTY_MOUSE_SHAPE_N_RESIZE
            | GHOSTTY_MOUSE_SHAPE_S_RESIZE
            | GHOSTTY_MOUSE_SHAPE_NS_RESIZE => IDC_SIZENS,
            GHOSTTY_MOUSE_SHAPE_NE_RESIZE
            | GHOSTTY_MOUSE_SHAPE_SW_RESIZE
            | GHOSTTY_MOUSE_SHAPE_NESW_RESIZE => IDC_SIZENESW,
            GHOSTTY_MOUSE_SHAPE_NW_RESIZE
            | GHOSTTY_MOUSE_SHAPE_SE_RESIZE
            | GHOSTTY_MOUSE_SHAPE_NWSE_RESIZE => IDC_SIZENWSE,
            _ => IDC_ARROW,
        };
        unsafe {
            self.cursor.set(LoadCursorW(null_mut(), id));
            self.apply_cursor();
        }
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.cursor_visible.set(visible);
        unsafe { self.apply_cursor() };
    }

    unsafe fn apply_cursor(&self) {
        SetCursor(if self.cursor_visible.get() {
            self.cursor.get()
        } else {
            null_mut()
        });
    }

    /// Resize so the client area is `width` by `height` logical pixels.
    pub fn set_client_size(&self, width: u32, height: u32) {
        let hwnd = self.hwnd();
        let scale = self.scale();
        unsafe {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: (width as f64 * scale).round() as i32,
                bottom: (height as f64 * scale).round() as i32,
            };
            AdjustWindowRectExForDpi(
                &mut rect,
                GetWindowLongW(hwnd, GWL_STYLE) as u32,
                0,
                GetWindowLongW(hwnd, GWL_EXSTYLE) as u32,
                GetDpiForWindow(hwnd),
            );
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    pub fn request_close(&self) {
        unsafe { PostMessageW(self.hwnd(), WM_CLOSE, 0, 0) };
    }

    pub fn toggle_maximize(&self) {
        unsafe {
            let hwnd = self.hwnd();
            ShowWindow(
                hwnd,
                if IsZoomed(hwnd) != 0 {
                    SW_RESTORE
                } else {
                    SW_MAXIMIZE
                },
            );
        }
    }

    /// Borderless fullscreen on the window's current monitor.
    pub fn toggle_fullscreen(&self) {
        unsafe {
            let hwnd = self.hwnd();
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            match self.restore.take() {
                None => {
                    let mut placement: WINDOWPLACEMENT = zeroed();
                    placement.length = size_of::<WINDOWPLACEMENT>() as u32;
                    let mut info: MONITORINFO = zeroed();
                    info.cbSize = size_of::<MONITORINFO>() as u32;
                    let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                    if GetWindowPlacement(hwnd, &mut placement) == 0
                        || GetMonitorInfoW(monitor, &mut info) == 0
                    {
                        return;
                    }

                    SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_OVERLAPPEDWINDOW as i32));
                    let r = info.rcMonitor;
                    SetWindowPos(
                        hwnd,
                        HWND_TOP,
                        r.left,
                        r.top,
                        r.right - r.left,
                        r.bottom - r.top,
                        SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                    );
                    self.restore.set(Some(placement));
                }
                Some(placement) => {
                    SetWindowLongW(hwnd, GWL_STYLE, style | WS_OVERLAPPEDWINDOW as i32);
                    SetWindowPlacement(hwnd, &placement);
                    SetWindowPos(
                        hwnd,
                        null_mut(),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE
                            | SWP_NOSIZE
                            | SWP_NOZORDER
                            | SWP_NOOWNERZORDER
                            | SWP_FRAMECHANGED,
                    );
                }
            }
        }
    }

    /// Close the window, confirming first if the process is still running.
    fn close(&self, process_alive: bool) {
        let surface = self.surface();
        let needs_confirm = process_alive
            && !surface.is_null()
            && unsafe { ghostty_surface_needs_confirm_quit(surface) };
        if needs_confirm
            && !confirm(
                self.hwnd(),
                "Close this terminal? A process is still running.",
            )
        {
            return;
        }
        unsafe { DestroyWindow(self.hwnd()) };
    }

    unsafe fn paint(&self) {
        let hwnd = self.hwnd();
        let surface = self.surface();

        if !surface.is_null() {
            let mut taken: ghostty_surface_frame_s = zeroed();
            if ghostty_surface_take_frame(surface, &mut taken) {
                if !taken.pixels.is_null() {
                    let len = taken.width as usize * taken.height as usize * 4;
                    let pixels = std::slice::from_raw_parts(taken.pixels, len);
                    let mut frame = self.frame.borrow_mut();
                    frame.width = taken.width;
                    frame.height = taken.height;
                    frame.bgra.clear();
                    frame.bgra.extend_from_slice(pixels);

                    // libghostty hands us RGBA; GDI wants BGRA.
                    for px in frame.bgra.as_chunks_mut::<4>().0 {
                        px.swap(0, 2);
                    }
                }
                ghostty_surface_free_frame(surface, &mut taken);
            }
        }

        let mut ps: PAINTSTRUCT = zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let frame = self.frame.borrow();
        let (width, height) = (frame.width as i32, frame.height as i32);
        if width > 0 && height > 0 {
            let mut bmi: BITMAPINFO = zeroed();
            bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = width;
            // A positive height means bottom-up rows, which is the order
            // OpenGL reads them back in.
            bmi.bmiHeader.biHeight = height;
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;
            SetDIBitsToDevice(
                hdc,
                0,
                0,
                width as u32,
                height as u32,
                0,
                0,
                0,
                height as u32,
                frame.bgra.as_ptr().cast(),
                &bmi,
                DIB_RGB_COLORS,
            );
        }

        // Fill whatever the frame doesn't cover, e.g. mid-resize.
        let mut client: RECT = zeroed();
        GetClientRect(hwnd, &mut client);
        let brush = GetStockObject(BLACK_BRUSH);
        if client.right > width {
            let rect = RECT {
                left: width,
                top: 0,
                right: client.right,
                bottom: client.bottom,
            };
            FillRect(hdc, &rect, brush);
        }
        if client.bottom > height {
            let rect = RECT {
                left: 0,
                top: height,
                right: width.min(client.right),
                bottom: client.bottom,
            };
            FillRect(hdc, &rect, brush);
        }
        EndPaint(hwnd, &ps);
    }
}

/// libghostty asks us to close a surface, e.g. when its process exits.
pub unsafe extern "C" fn close_surface(userdata: *mut c_void, process_alive: bool) {
    if let Some(window) = Window::from_userdata(userdata) {
        PostMessageW(window.hwnd(), WM_APP_CLOSE, process_alive as WPARAM, 0);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lparam as *const CREATESTRUCTW);
        let window = create.lpCreateParams as *const Window;
        (*window).hwnd.set(hwnd);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, window as isize);
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let Some(window) = Window::from_hwnd(hwnd) else {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    };
    let surface = window.surface();

    match msg {
        WM_PAINT => {
            window.paint();
            0
        }

        // Every pixel is covered by WM_PAINT, so skip erasing to avoid flicker.
        WM_ERASEBKGND => 1,

        WM_SIZE => {
            if !surface.is_null() {
                let visible = wparam != SIZE_MINIMIZED as WPARAM;
                ghostty_surface_set_occlusion(surface, visible);
                let (width, height) = ((lparam & 0xffff) as u32, ((lparam >> 16) & 0xffff) as u32);
                if visible && width > 0 && height > 0 {
                    ghostty_surface_set_size(surface, width, height);
                }
            }
            0
        }

        WM_DPICHANGED => {
            let scale = (wparam & 0xffff) as f64 / 96.0;
            window.scale.set(scale);
            if !surface.is_null() {
                ghostty_surface_set_content_scale(surface, scale, scale);
            }
            let r = &*(lparam as *const RECT);
            SetWindowPos(
                hwnd,
                null_mut(),
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            0
        }

        WM_SETFOCUS | WM_KILLFOCUS => {
            let focused = msg == WM_SETFOCUS;
            if !surface.is_null() {
                ghostty_surface_set_focus(surface, focused);
            }
            ghostty_app_set_focus(app::get().app.get(), focused);
            0
        }

        // Let Windows handle Alt+F4 so it closes the window as usual.
        WM_SYSKEYDOWN if wparam == VK_F4 as WPARAM => DefWindowProcW(hwnd, msg, wparam, lparam),

        WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            if input::key(window, msg, wparam, lparam) {
                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        WM_MOUSEMOVE => {
            input::mouse_pos(window, lparam);
            0
        }

        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
        | WM_MBUTTONUP => {
            input::mouse_button(window, msg, wparam, lparam);
            0
        }

        // XBUTTON messages are the one case that must return TRUE.
        WM_XBUTTONDOWN | WM_XBUTTONUP => {
            input::mouse_button(window, msg, wparam, lparam);
            1
        }

        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            input::scroll(window, msg, wparam);
            0
        }

        WM_SETCURSOR if (lparam & 0xffff) as u32 == HTCLIENT => {
            window.apply_cursor();
            1
        }

        WM_CLOSE => {
            window.close(true);
            0
        }

        WM_APP_CLOSE => {
            window.close(wparam != 0);
            0
        }

        clipboard::WM_APP_CLIPBOARD_READ => {
            clipboard::complete_read(window, wparam as *mut c_void);
            0
        }

        clipboard::WM_APP_CLIPBOARD_CONFIRM => {
            clipboard::complete_confirm(window, wparam as *mut clipboard::PendingConfirm);
            0
        }

        clipboard::WM_APP_CLIPBOARD_WRITE => {
            clipboard::complete_write(window, wparam as *mut String);
            0
        }

        WM_DESTROY => {
            let surface = window.surface.replace(null_mut());
            if !surface.is_null() {
                ghostty_surface_free(surface);
            }

            let app = app::get();
            let mut windows = app.windows.borrow_mut();
            windows.retain(|&h| h != hwnd);
            if windows.is_empty() {
                PostQuitMessage(0);
            }
            0
        }

        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(window as *const Window as *mut Window));
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
