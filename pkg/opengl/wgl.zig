//! Thin WGL bindings for offscreen OpenGL contexts on Windows.
//!
//! Ghostty never draws to a window's default framebuffer: it renders
//! into a framebuffer object and reads the pixels back for the apprt to
//! present. WGL still requires a device context with a pixel format to
//! create and bind a context, so each `Context` owns a hidden window
//! that exists only to provide one.
//!
//! Call `load()` once before creating any contexts.

const std = @import("std");
const windows = std.os.windows;

const log = std.log.scoped(.opengl_wgl);

pub const BOOL = windows.BOOL;
pub const HDC = windows.HDC;
pub const HGLRC = windows.HGLRC;
pub const HWND = windows.HWND;

const BYTE = windows.BYTE;
const WORD = windows.WORD;
const DWORD = windows.DWORD;
const UINT = windows.UINT;
const LPARAM = windows.LPARAM;
const WPARAM = usize;
const LRESULT = isize;

// WGL_ARB_create_context and WGL_ARB_create_context_profile
pub const CONTEXT_MAJOR_VERSION_ARB = 0x2091;
pub const CONTEXT_MINOR_VERSION_ARB = 0x2092;
pub const CONTEXT_PROFILE_MASK_ARB = 0x9126;
pub const CONTEXT_CORE_PROFILE_BIT_ARB = 0x00000001;

const CS_OWNDC = 0x0020;
const WS_POPUP = 0x80000000;
const PFD_DOUBLEBUFFER = 0x00000001;
const PFD_DRAW_TO_WINDOW = 0x00000004;
const PFD_SUPPORT_OPENGL = 0x00000020;
const PFD_TYPE_RGBA = 0;

const class_name = std.unicode.utf8ToUtf16LeStringLiteral("GhosttyWGL");

const WNDPROC = *const fn (HWND, UINT, WPARAM, LPARAM) callconv(.winapi) LRESULT;

const WNDCLASSEXW = extern struct {
    cbSize: UINT = @sizeOf(WNDCLASSEXW),
    style: UINT,
    lpfnWndProc: WNDPROC,
    cbClsExtra: c_int = 0,
    cbWndExtra: c_int = 0,
    hInstance: ?windows.HINSTANCE,
    hIcon: ?windows.HICON = null,
    hCursor: ?windows.HCURSOR = null,
    hbrBackground: ?windows.HBRUSH = null,
    lpszMenuName: ?windows.LPCWSTR = null,
    lpszClassName: windows.LPCWSTR,
    hIconSm: ?windows.HICON = null,
};

const PIXELFORMATDESCRIPTOR = extern struct {
    nSize: WORD = @sizeOf(PIXELFORMATDESCRIPTOR),
    nVersion: WORD = 1,
    dwFlags: DWORD,
    iPixelType: BYTE,
    cColorBits: BYTE,
    cRedBits: BYTE = 0,
    cRedShift: BYTE = 0,
    cGreenBits: BYTE = 0,
    cGreenShift: BYTE = 0,
    cBlueBits: BYTE = 0,
    cBlueShift: BYTE = 0,
    cAlphaBits: BYTE,
    cAlphaShift: BYTE = 0,
    cAccumBits: BYTE = 0,
    cAccumRedBits: BYTE = 0,
    cAccumGreenBits: BYTE = 0,
    cAccumBlueBits: BYTE = 0,
    cAccumAlphaBits: BYTE = 0,
    cDepthBits: BYTE = 0,
    cStencilBits: BYTE = 0,
    cAuxBuffers: BYTE = 0,
    iLayerType: BYTE = 0,
    bReserved: BYTE = 0,
    dwLayerMask: DWORD = 0,
    dwVisibleMask: DWORD = 0,
    dwDamageMask: DWORD = 0,
};

extern "kernel32" fn GetModuleHandleW(
    lpModuleName: ?windows.LPCWSTR,
) callconv(.winapi) ?windows.HINSTANCE;
extern "user32" fn RegisterClassExW(
    lpWndClass: *const WNDCLASSEXW,
) callconv(.winapi) windows.ATOM;
extern "user32" fn CreateWindowExW(
    dwExStyle: DWORD,
    lpClassName: windows.LPCWSTR,
    lpWindowName: ?windows.LPCWSTR,
    dwStyle: DWORD,
    X: c_int,
    Y: c_int,
    nWidth: c_int,
    nHeight: c_int,
    hWndParent: ?HWND,
    hMenu: ?windows.HMENU,
    hInstance: ?windows.HINSTANCE,
    lpParam: ?windows.LPVOID,
) callconv(.winapi) ?HWND;
extern "user32" fn DestroyWindow(hWnd: HWND) callconv(.winapi) BOOL;
extern "user32" fn GetDC(hWnd: ?HWND) callconv(.winapi) ?HDC;
extern "user32" fn DefWindowProcW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) callconv(.winapi) LRESULT;
extern "gdi32" fn ChoosePixelFormat(
    hdc: HDC,
    ppfd: *const PIXELFORMATDESCRIPTOR,
) callconv(.winapi) c_int;
extern "gdi32" fn SetPixelFormat(
    hdc: HDC,
    format: c_int,
    ppfd: *const PIXELFORMATDESCRIPTOR,
) callconv(.winapi) BOOL;
extern "opengl32" fn wglCreateContext(hdc: HDC) callconv(.winapi) ?HGLRC;
extern "opengl32" fn wglDeleteContext(hglrc: HGLRC) callconv(.winapi) BOOL;
extern "opengl32" fn wglMakeCurrent(hdc: ?HDC, hglrc: ?HGLRC) callconv(.winapi) BOOL;
extern "opengl32" fn wglGetProcAddress(name: windows.LPCSTR) callconv(.winapi) ?windows.PROC;

const CreateContextAttribsARB = *const fn (
    hdc: HDC,
    share: ?HGLRC,
    attribs: [*]const c_int,
) callconv(.winapi) ?HGLRC;

/// Loaded by `load()` using a temporary legacy context, since WGL
/// extension functions can only be queried while a context is current.
var create_context_attribs: ?CreateContextAttribsARB = null;

pub const Error = error{
    RegisterClassFailed,
    CreateWindowFailed,
    PixelFormatFailed,
    CreateContextFailed,
    MakeCurrentFailed,
    CreateContextAttribsUnsupported,
};

/// Register our hidden window class and load the WGL extension
/// functions needed to create core profile contexts. Must be called
/// on the thread that will create contexts. Safe to call repeatedly.
pub fn load() Error!void {
    if (create_context_attribs != null) return;

    const wc: WNDCLASSEXW = .{
        .style = CS_OWNDC,
        .lpfnWndProc = DefWindowProcW,
        .hInstance = GetModuleHandleW(null),
        .lpszClassName = class_name,
    };
    if (RegisterClassExW(&wc) == 0) return error.RegisterClassFailed;

    const window: Window = try .create();
    defer window.destroy();

    const legacy = wglCreateContext(window.hdc) orelse
        return error.CreateContextFailed;
    defer _ = wglDeleteContext(legacy);
    if (!wglMakeCurrent(window.hdc, legacy).toBool()) return error.MakeCurrentFailed;
    defer _ = wglMakeCurrent(null, null);

    const proc = wglGetProcAddress("wglCreateContextAttribsARB") orelse {
        log.warn("driver does not support WGL_ARB_create_context", .{});
        return error.CreateContextAttribsUnsupported;
    };
    create_context_attribs = @ptrCast(proc);
}

/// A hidden window with a pixel format set, used only for its DC.
const Window = struct {
    hwnd: HWND,
    hdc: HDC,

    fn create() Error!Window {
        const hwnd = CreateWindowExW(
            0,
            class_name,
            null,
            WS_POPUP,
            0,
            0,
            1,
            1,
            null,
            null,
            GetModuleHandleW(null),
            null,
        ) orelse return error.CreateWindowFailed;
        errdefer _ = DestroyWindow(hwnd);

        // CS_OWNDC gives the window a private DC that stays valid for
        // the window's lifetime, so we never release it.
        const hdc = GetDC(hwnd) orelse return error.CreateWindowFailed;

        const pfd: PIXELFORMATDESCRIPTOR = .{
            .dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            .iPixelType = PFD_TYPE_RGBA,
            .cColorBits = 32,
            .cAlphaBits = 8,
        };
        const format = ChoosePixelFormat(hdc, &pfd);
        if (format == 0) return error.PixelFormatFailed;
        if (!SetPixelFormat(hdc, format, &pfd).toBool()) return error.PixelFormatFailed;

        return .{ .hwnd = hwnd, .hdc = hdc };
    }

    fn destroy(self: Window) void {
        _ = DestroyWindow(self.hwnd);
    }
};

/// An OpenGL context and the hidden window that backs it. The window
/// must be destroyed on the thread that created it, so create and
/// destroy contexts on the same thread; any thread may make it current.
pub const Context = struct {
    window: Window,
    hglrc: HGLRC,

    pub fn create(attribs: []const c_int) Error!Context {
        const createFn = create_context_attribs orelse
            return error.CreateContextAttribsUnsupported;

        const window: Window = try .create();
        errdefer window.destroy();

        // The attribute list must be zero-terminated.
        var list: [16]c_int = @splat(0);
        std.debug.assert(attribs.len < list.len);
        @memcpy(list[0..attribs.len], attribs);

        const hglrc = createFn(window.hdc, null, &list) orelse
            return error.CreateContextFailed;
        return .{ .window = window, .hglrc = hglrc };
    }

    pub fn destroy(self: *const Context) void {
        _ = wglDeleteContext(self.hglrc);
        self.window.destroy();
    }

    pub fn makeCurrent(self: *const Context) Error!void {
        if (!wglMakeCurrent(self.window.hdc, self.hglrc).toBool()) {
            return error.MakeCurrentFailed;
        }
    }
};

/// Unbind whatever context is current on the calling thread.
pub fn releaseCurrent() void {
    _ = wglMakeCurrent(null, null);
}
