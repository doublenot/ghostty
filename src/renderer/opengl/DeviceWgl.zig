//! The OpenGL device context on Windows.
//!
//! Windows has no EGL, so surface contexts are created with WGL
//! instead (see `gl.wgl`). WGL has no display or config shared between
//! contexts, so the device only ensures that WGL is loaded before any
//! surface creates its context.
const Device = @This();

const std = @import("std");
const Allocator = std.mem.Allocator;
const gl = @import("opengl");

const log = std.log.scoped(.opengl);

/// Options used when creating the OpenGL device. WGL has none.
pub const Options = struct {};

pub fn init(alloc: Allocator, options: Options) !*Device {
    _ = options;

    gl.wgl.load() catch |err| {
        log.warn("failed to load WGL err={}", .{err});
        return err;
    };

    return try alloc.create(Device);
}

pub fn deinit(self: *Device, alloc: Allocator) void {
    alloc.destroy(self);
}
