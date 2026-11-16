//! Ghostty for Windows.
//!
//! A Win32 host for libghostty, in the same way the macOS app hosts it
//! from Swift: this crate owns windows, input and the clipboard, while
//! libghostty owns the terminal, config, fonts and rendering. libghostty
//! renders each surface offscreen and signals GHOSTTY_ACTION_RENDER when
//! a frame is ready, which we pull with `ghostty_surface_take_frame`.

// No console window, even in debug builds: blocking writes to a console
// stall libghostty's threads. Run from a terminal, stderr is still
// inherited; debug builds started without one log to a file instead.
#![windows_subsystem = "windows"]

mod app;
mod clipboard;
mod ffi;
mod input;
mod util;
mod window;

fn main() {
    std::process::exit(app::run());
}
