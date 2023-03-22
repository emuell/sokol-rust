//! To use this module, enable the feature "imgui"
// machine generated, do not edit

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::{app as sapp, gfx as sg};

/// Helper function to convert a C string to a rust string slice
#[inline]
fn c_char_ptr_to_rust_str(c_char_ptr: *const core::ffi::c_char) -> &'static str {
    let c_str = unsafe { core::ffi::CStr::from_ptr(c_char_ptr) };
    c_str.to_str().expect("c_char_ptr contained invalid Utf8 Data")
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Allocator {
    pub alloc: Option<extern "C" fn(usize, *mut core::ffi::c_void) -> *mut core::ffi::c_void>,
    pub free: Option<extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void)>,
    pub user_data: *mut core::ffi::c_void,
}
impl Allocator {
    pub const fn new() -> Self {
        Self { alloc: None, free: None, user_data: core::ptr::null_mut() }
    }
}
impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Desc {
    pub max_vertices: i32,
    pub color_format: sg::PixelFormat,
    pub depth_format: sg::PixelFormat,
    pub sample_count: i32,
    pub ini_filename: *const core::ffi::c_char,
    pub no_default_font: bool,
    pub disable_paste_override: bool,
    pub disable_set_mouse_cursor: bool,
    pub disable_windows_resize_from_edges: bool,
    pub write_alpha_channel: bool,
    pub allocator: Allocator,
}
impl Desc {
    pub const fn new() -> Self {
        Self {
            max_vertices: 0,
            color_format: sg::PixelFormat::new(),
            depth_format: sg::PixelFormat::new(),
            sample_count: 0,
            ini_filename: core::ptr::null(),
            no_default_font: false,
            disable_paste_override: false,
            disable_set_mouse_cursor: false,
            disable_windows_resize_from_edges: false,
            write_alpha_channel: false,
            allocator: Allocator::new(),
        }
    }
}
impl Default for Desc {
    fn default() -> Self {
        Self::new()
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FrameDesc {
    pub width: i32,
    pub height: i32,
    pub delta_time: f64,
    pub dpi_scale: f32,
}
impl FrameDesc {
    pub const fn new() -> Self {
        Self { width: 0, height: 0, delta_time: 0.0, dpi_scale: 0.0 }
    }
}
impl Default for FrameDesc {
    fn default() -> Self {
        Self::new()
    }
}
pub mod ffi {
    #![allow(unused_imports)]
    use super::*;
    extern "C" {
        pub fn simgui_setup(desc: *const Desc);
        pub fn simgui_new_frame(desc: *const FrameDesc);
        pub fn simgui_render();
        pub fn simgui_handle_event(ev: *const sapp::Event) -> bool;
        pub fn simgui_map_keycode(keycode: sapp::Keycode) -> i32;
        pub fn simgui_shutdown();
    }
}
#[inline]
pub fn setup(desc: &Desc) {
    unsafe { ffi::simgui_setup(desc) }
}
#[inline]
pub fn new_frame(desc: &FrameDesc) {
    unsafe { ffi::simgui_new_frame(desc) }
}
#[inline]
pub fn render() {
    unsafe { ffi::simgui_render() }
}
#[inline]
pub fn handle_event(ev: &sapp::Event) -> bool {
    unsafe { ffi::simgui_handle_event(ev) }
}
#[inline]
pub fn map_keycode(keycode: sapp::Keycode) -> i32 {
    unsafe { ffi::simgui_map_keycode(keycode) }
}
#[inline]
pub fn shutdown() {
    unsafe { ffi::simgui_shutdown() }
}
