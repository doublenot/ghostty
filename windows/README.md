# Ghostty for Windows

A native Win32 host for libghostty, written in Rust. It plays the same
role as the macOS app: this crate owns windows, input and the clipboard,
while libghostty owns the terminal, configuration, fonts and rendering.

libghostty renders each surface offscreen with OpenGL (via WGL) and
signals `GHOSTTY_ACTION_RENDER` when a frame is ready. The app pulls it
with `ghostty_surface_take_frame` and blits it to the window.

## Requirements

- Zig (the version in `build.zig.zon`'s `minimum_zig_version`)
- Visual Studio Build Tools with the "Desktop development with C++"
  workload (MSVC and the Windows SDK)
- Rust with the `x86_64-pc-windows-msvc` toolchain
- A GPU driver supporting OpenGL 4.3

## Building

From the repository root, build libghostty:

```powershell
zig build
```

This installs `ghostty-internal.dll`, its import library and
`ghostty.h` into `zig-out`. Then build and run the app:

```powershell
cd windows
cargo run
```

`build.rs` links against `../zig-out/lib` and copies the DLL next to the
executable. Set `GHOSTTY_LIB_DIR` to use a libghostty from elsewhere.

Debug builds keep a console window so libghostty's logs are visible.

## Status

Working: a terminal window per surface, keyboard input (including
AltGr and dead keys), mouse and scrolling, clipboard with paste
protection, titles, cursor shapes, DPI scaling, fullscreen, new windows.

Not yet: tabs and splits, IME composition, a GPU presentation path
(frames are currently copied through the CPU), and an installer.
