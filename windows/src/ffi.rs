//! Bindings for the subset of `include/ghostty.h` this app uses.
//!
//! These must be kept in sync with the header by hand. The tests at the
//! bottom pin the sizes and offsets the header produces with MSVC.
#![allow(non_camel_case_types, dead_code)]

use std::ffi::{c_char, c_int, c_void};

pub type ghostty_app_t = *mut c_void;
pub type ghostty_config_t = *mut c_void;
pub type ghostty_surface_t = *mut c_void;

pub const GHOSTTY_SUCCESS: c_int = 0;

// ghostty_platform_e
pub const GHOSTTY_PLATFORM_WINDOWS: c_int = 3;

// ghostty_clipboard_e
pub const GHOSTTY_CLIPBOARD_STANDARD: c_int = 0;

// ghostty_clipboard_request_e
pub const GHOSTTY_CLIPBOARD_REQUEST_PASTE: c_int = 0;
pub const GHOSTTY_CLIPBOARD_REQUEST_OSC_52_READ: c_int = 1;
pub const GHOSTTY_CLIPBOARD_REQUEST_OSC_52_WRITE: c_int = 2;

// ghostty_clipboard_read_result_e
pub const GHOSTTY_CLIPBOARD_READ_STARTED: c_int = 0;
pub const GHOSTTY_CLIPBOARD_READ_UNAVAILABLE: c_int = 1;
pub const GHOSTTY_CLIPBOARD_READ_UNSUPPORTED: c_int = 2;

// ghostty_input_mouse_state_e
pub const GHOSTTY_MOUSE_RELEASE: c_int = 0;
pub const GHOSTTY_MOUSE_PRESS: c_int = 1;

// ghostty_input_mouse_button_e
pub const GHOSTTY_MOUSE_LEFT: c_int = 1;
pub const GHOSTTY_MOUSE_RIGHT: c_int = 2;
pub const GHOSTTY_MOUSE_MIDDLE: c_int = 3;
pub const GHOSTTY_MOUSE_FOUR: c_int = 4;
pub const GHOSTTY_MOUSE_FIVE: c_int = 5;

// ghostty_input_mods_e
pub const GHOSTTY_MODS_NONE: c_int = 0;
pub const GHOSTTY_MODS_SHIFT: c_int = 1 << 0;
pub const GHOSTTY_MODS_CTRL: c_int = 1 << 1;
pub const GHOSTTY_MODS_ALT: c_int = 1 << 2;
pub const GHOSTTY_MODS_SUPER: c_int = 1 << 3;
pub const GHOSTTY_MODS_CAPS: c_int = 1 << 4;
pub const GHOSTTY_MODS_NUM: c_int = 1 << 5;
pub const GHOSTTY_MODS_SHIFT_RIGHT: c_int = 1 << 6;
pub const GHOSTTY_MODS_CTRL_RIGHT: c_int = 1 << 7;
pub const GHOSTTY_MODS_ALT_RIGHT: c_int = 1 << 8;
pub const GHOSTTY_MODS_SUPER_RIGHT: c_int = 1 << 9;

// ghostty_input_action_e
pub const GHOSTTY_ACTION_RELEASE: c_int = 0;
pub const GHOSTTY_ACTION_PRESS: c_int = 1;
pub const GHOSTTY_ACTION_REPEAT: c_int = 2;

// ghostty_surface_context_e
pub const GHOSTTY_SURFACE_CONTEXT_WINDOW: c_int = 0;

// ghostty_target_tag_e
pub const GHOSTTY_TARGET_APP: c_int = 0;
pub const GHOSTTY_TARGET_SURFACE: c_int = 1;

// ghostty_action_tag_e
pub const GHOSTTY_ACTION_QUIT: c_int = 0;
pub const GHOSTTY_ACTION_NEW_WINDOW: c_int = 1;
pub const GHOSTTY_ACTION_CLOSE_ALL_WINDOWS: c_int = 5;
pub const GHOSTTY_ACTION_TOGGLE_MAXIMIZE: c_int = 6;
pub const GHOSTTY_ACTION_TOGGLE_FULLSCREEN: c_int = 7;
pub const GHOSTTY_ACTION_INITIAL_SIZE: c_int = 24;
pub const GHOSTTY_ACTION_CELL_SIZE: c_int = 25;
pub const GHOSTTY_ACTION_RENDER: c_int = 27;
pub const GHOSTTY_ACTION_SET_TITLE: c_int = 33;
pub const GHOSTTY_ACTION_MOUSE_SHAPE: c_int = 38;
pub const GHOSTTY_ACTION_MOUSE_VISIBILITY: c_int = 39;
pub const GHOSTTY_ACTION_RENDERER_HEALTH: c_int = 41;
pub const GHOSTTY_ACTION_CLOSE_WINDOW: c_int = 51;
pub const GHOSTTY_ACTION_RING_BELL: c_int = 52;
pub const GHOSTTY_ACTION_OPEN_URL: c_int = 57;

// ghostty_action_mouse_shape_e
pub const GHOSTTY_MOUSE_SHAPE_DEFAULT: c_int = 0;
pub const GHOSTTY_MOUSE_SHAPE_CONTEXT_MENU: c_int = 1;
pub const GHOSTTY_MOUSE_SHAPE_HELP: c_int = 2;
pub const GHOSTTY_MOUSE_SHAPE_POINTER: c_int = 3;
pub const GHOSTTY_MOUSE_SHAPE_PROGRESS: c_int = 4;
pub const GHOSTTY_MOUSE_SHAPE_WAIT: c_int = 5;
pub const GHOSTTY_MOUSE_SHAPE_CELL: c_int = 6;
pub const GHOSTTY_MOUSE_SHAPE_CROSSHAIR: c_int = 7;
pub const GHOSTTY_MOUSE_SHAPE_TEXT: c_int = 8;
pub const GHOSTTY_MOUSE_SHAPE_VERTICAL_TEXT: c_int = 9;
pub const GHOSTTY_MOUSE_SHAPE_ALIAS: c_int = 10;
pub const GHOSTTY_MOUSE_SHAPE_COPY: c_int = 11;
pub const GHOSTTY_MOUSE_SHAPE_MOVE: c_int = 12;
pub const GHOSTTY_MOUSE_SHAPE_NO_DROP: c_int = 13;
pub const GHOSTTY_MOUSE_SHAPE_NOT_ALLOWED: c_int = 14;
pub const GHOSTTY_MOUSE_SHAPE_GRAB: c_int = 15;
pub const GHOSTTY_MOUSE_SHAPE_GRABBING: c_int = 16;
pub const GHOSTTY_MOUSE_SHAPE_ALL_SCROLL: c_int = 17;
pub const GHOSTTY_MOUSE_SHAPE_COL_RESIZE: c_int = 18;
pub const GHOSTTY_MOUSE_SHAPE_ROW_RESIZE: c_int = 19;
pub const GHOSTTY_MOUSE_SHAPE_N_RESIZE: c_int = 20;
pub const GHOSTTY_MOUSE_SHAPE_E_RESIZE: c_int = 21;
pub const GHOSTTY_MOUSE_SHAPE_S_RESIZE: c_int = 22;
pub const GHOSTTY_MOUSE_SHAPE_W_RESIZE: c_int = 23;
pub const GHOSTTY_MOUSE_SHAPE_NE_RESIZE: c_int = 24;
pub const GHOSTTY_MOUSE_SHAPE_NW_RESIZE: c_int = 25;
pub const GHOSTTY_MOUSE_SHAPE_SE_RESIZE: c_int = 26;
pub const GHOSTTY_MOUSE_SHAPE_SW_RESIZE: c_int = 27;
pub const GHOSTTY_MOUSE_SHAPE_EW_RESIZE: c_int = 28;
pub const GHOSTTY_MOUSE_SHAPE_NS_RESIZE: c_int = 29;
pub const GHOSTTY_MOUSE_SHAPE_NESW_RESIZE: c_int = 30;
pub const GHOSTTY_MOUSE_SHAPE_NWSE_RESIZE: c_int = 31;
pub const GHOSTTY_MOUSE_SHAPE_ZOOM_IN: c_int = 32;
pub const GHOSTTY_MOUSE_SHAPE_ZOOM_OUT: c_int = 33;

// ghostty_action_mouse_visibility_e
pub const GHOSTTY_MOUSE_VISIBLE: c_int = 0;
pub const GHOSTTY_MOUSE_HIDDEN: c_int = 1;

#[repr(C)]
pub struct ghostty_input_key_s {
    pub action: c_int,
    pub mods: c_int,
    pub consumed_mods: c_int,
    pub keycode: u32,
    pub text: *const c_char,
    pub unshifted_codepoint: u32,
    pub composing: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ghostty_platform_windows_s {
    pub hwnd: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ghostty_platform_u {
    pub windows: ghostty_platform_windows_s,
}

#[repr(C)]
pub struct ghostty_env_var_s {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct ghostty_surface_config_s {
    pub platform_tag: c_int,
    pub platform: ghostty_platform_u,
    pub userdata: *mut c_void,
    pub scale_factor: f64,
    pub font_size: f32,
    pub working_directory: *const c_char,
    pub command: *const c_char,
    pub env_vars: *mut ghostty_env_var_s,
    pub env_var_count: usize,
    pub initial_input: *const c_char,
    pub wait_after_command: bool,
    pub context: c_int,
}

#[repr(C)]
pub struct ghostty_surface_size_s {
    pub columns: u16,
    pub rows: u16,
    pub width_px: u32,
    pub height_px: u32,
    pub cell_width_px: u32,
    pub cell_height_px: u32,
}

#[repr(C)]
pub struct ghostty_surface_frame_s {
    pub width: u32,
    pub height: u32,
    pub pixels: *const u8,
    pub handle: *mut c_void,
}

#[repr(C)]
pub struct ghostty_clipboard_content_s {
    pub mime: *const c_char,
    pub data: *const c_char,
    pub len: usize,
}

#[repr(C)]
pub struct ghostty_clipboard_complete_s {
    pub contents: *const ghostty_clipboard_content_s,
    pub contents_len: usize,
    pub available: *const *const c_char,
    pub available_len: usize,
    pub confirmed: bool,
    pub remember: bool,
}

#[repr(C)]
pub struct ghostty_clipboard_confirm_s {
    pub contents: *const ghostty_clipboard_content_s,
    pub contents_len: usize,
    pub available: *const *const c_char,
    pub available_len: usize,
    pub name: *const c_char,
    pub can_remember: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ghostty_target_u {
    pub surface: ghostty_surface_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ghostty_target_s {
    pub tag: c_int,
    pub target: ghostty_target_u,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ghostty_action_set_title_s {
    pub title: *const c_char,
}

/// ghostty_action_initial_size_s and ghostty_action_cell_size_s.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ghostty_action_size_s {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ghostty_action_open_url_s {
    pub kind: c_int,
    pub url: *const c_char,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ghostty_action_u {
    pub set_title: ghostty_action_set_title_s,
    pub initial_size: ghostty_action_size_s,
    pub cell_size: ghostty_action_size_s,
    pub mouse_shape: c_int,
    pub mouse_visibility: c_int,
    pub renderer_health: c_int,
    pub open_url: ghostty_action_open_url_s,
    // Members we don't read; the largest is 24 bytes.
    _size: [u64; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ghostty_action_s {
    pub tag: c_int,
    pub action: ghostty_action_u,
}

#[repr(C)]
pub struct ghostty_diagnostic_s {
    pub message: *const c_char,
}

pub type ghostty_runtime_wakeup_cb = Option<unsafe extern "C" fn(*mut c_void)>;
pub type ghostty_runtime_read_clipboard_cb = Option<
    unsafe extern "C" fn(
        *mut c_void,
        c_int,
        *mut c_void,
        *const *const c_char,
        usize,
        bool,
    ) -> c_int,
>;
pub type ghostty_runtime_confirm_read_clipboard_cb = Option<
    unsafe extern "C" fn(*mut c_void, *const ghostty_clipboard_confirm_s, *mut c_void, c_int),
>;
pub type ghostty_runtime_write_clipboard_cb = Option<
    unsafe extern "C" fn(*mut c_void, c_int, *const ghostty_clipboard_content_s, usize, bool),
>;
pub type ghostty_runtime_close_surface_cb = Option<unsafe extern "C" fn(*mut c_void, bool)>;
pub type ghostty_runtime_action_cb =
    Option<unsafe extern "C" fn(ghostty_app_t, ghostty_target_s, ghostty_action_s) -> bool>;

#[repr(C)]
pub struct ghostty_runtime_config_s {
    pub userdata: *mut c_void,
    pub supports_selection_clipboard: bool,
    pub wakeup_cb: ghostty_runtime_wakeup_cb,
    pub action_cb: ghostty_runtime_action_cb,
    pub read_clipboard_cb: ghostty_runtime_read_clipboard_cb,
    pub confirm_read_clipboard_cb: ghostty_runtime_confirm_read_clipboard_cb,
    pub write_clipboard_cb: ghostty_runtime_write_clipboard_cb,
    pub close_surface_cb: ghostty_runtime_close_surface_cb,
}

extern "C" {
    pub fn ghostty_init(argc: usize, argv: *mut *mut c_char) -> c_int;

    pub fn ghostty_config_new() -> ghostty_config_t;
    pub fn ghostty_config_free(config: ghostty_config_t);
    pub fn ghostty_config_load_cli_args(config: ghostty_config_t);
    pub fn ghostty_config_load_default_files(config: ghostty_config_t);
    pub fn ghostty_config_load_recursive_files(config: ghostty_config_t);
    pub fn ghostty_config_finalize(config: ghostty_config_t);
    pub fn ghostty_config_diagnostics_count(config: ghostty_config_t) -> u32;
    pub fn ghostty_config_get_diagnostic(
        config: ghostty_config_t,
        index: u32,
    ) -> ghostty_diagnostic_s;

    pub fn ghostty_app_new(
        runtime: *const ghostty_runtime_config_s,
        config: ghostty_config_t,
    ) -> ghostty_app_t;
    pub fn ghostty_app_free(app: ghostty_app_t);
    pub fn ghostty_app_tick(app: ghostty_app_t);
    pub fn ghostty_app_set_focus(app: ghostty_app_t, focused: bool);

    pub fn ghostty_surface_config_new() -> ghostty_surface_config_s;
    pub fn ghostty_surface_new(
        app: ghostty_app_t,
        config: *const ghostty_surface_config_s,
    ) -> ghostty_surface_t;
    pub fn ghostty_surface_free(surface: ghostty_surface_t);
    pub fn ghostty_surface_userdata(surface: ghostty_surface_t) -> *mut c_void;
    pub fn ghostty_surface_inherited_config(
        surface: ghostty_surface_t,
        context: c_int,
    ) -> ghostty_surface_config_s;
    pub fn ghostty_surface_needs_confirm_quit(surface: ghostty_surface_t) -> bool;
    pub fn ghostty_surface_set_content_scale(surface: ghostty_surface_t, x: f64, y: f64);
    pub fn ghostty_surface_set_focus(surface: ghostty_surface_t, focused: bool);
    pub fn ghostty_surface_set_occlusion(surface: ghostty_surface_t, visible: bool);
    pub fn ghostty_surface_set_size(surface: ghostty_surface_t, width: u32, height: u32);
    pub fn ghostty_surface_take_frame(
        surface: ghostty_surface_t,
        frame: *mut ghostty_surface_frame_s,
    ) -> bool;
    pub fn ghostty_surface_free_frame(
        surface: ghostty_surface_t,
        frame: *mut ghostty_surface_frame_s,
    );
    pub fn ghostty_surface_key(surface: ghostty_surface_t, key: ghostty_input_key_s) -> bool;
    pub fn ghostty_surface_preedit(surface: ghostty_surface_t, text: *const c_char, len: usize);
    pub fn ghostty_surface_mouse_button(
        surface: ghostty_surface_t,
        state: c_int,
        button: c_int,
        mods: c_int,
    ) -> bool;
    pub fn ghostty_surface_mouse_pos(surface: ghostty_surface_t, x: f64, y: f64, mods: c_int);
    pub fn ghostty_surface_mouse_scroll(surface: ghostty_surface_t, x: f64, y: f64, mods: c_int);
    pub fn ghostty_surface_complete_clipboard_request(
        surface: ghostty_surface_t,
        complete: *const ghostty_clipboard_complete_s,
        state: *mut c_void,
    );
    pub fn ghostty_surface_deny_clipboard_request(surface: ghostty_surface_t, state: *mut c_void);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    // Values printed by compiling include/ghostty.h with MSVC's ABI.
    #[test]
    fn layout_matches_header() {
        assert_eq!(size_of::<ghostty_input_key_s>(), 32);
        assert_eq!(size_of::<ghostty_surface_config_s>(), 88);
        assert_eq!(offset_of!(ghostty_surface_config_s, userdata), 16);
        assert_eq!(offset_of!(ghostty_surface_config_s, scale_factor), 24);
        assert_eq!(offset_of!(ghostty_surface_config_s, wait_after_command), 80);
        assert_eq!(offset_of!(ghostty_surface_config_s, context), 84);
        assert_eq!(size_of::<ghostty_runtime_config_s>(), 64);
        assert_eq!(offset_of!(ghostty_runtime_config_s, wakeup_cb), 16);
        assert_eq!(offset_of!(ghostty_runtime_config_s, close_surface_cb), 56);
        assert_eq!(size_of::<ghostty_target_s>(), 16);
        assert_eq!(size_of::<ghostty_action_u>(), 24);
        assert_eq!(size_of::<ghostty_action_s>(), 32);
        assert_eq!(size_of::<ghostty_clipboard_complete_s>(), 40);
        assert_eq!(size_of::<ghostty_clipboard_confirm_s>(), 48);
        assert_eq!(size_of::<ghostty_surface_frame_s>(), 24);
        assert_eq!(size_of::<ghostty_surface_size_s>(), 20);
    }
}
