//! Raw declarations for the part of libdivecomputer DiveLens uses: listing
//! supported dive computers and parsing a stored dive into samples.
//! See `third_party/libdivecomputer/include/libdivecomputer/*.h`.
#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int, c_uint, c_void};

pub type dc_status_t = c_int;
pub const DC_STATUS_SUCCESS: dc_status_t = 0;
pub const DC_STATUS_DONE: dc_status_t = 1;

/// `dc_sample_type_t` values used by DiveLens.
pub type dc_sample_type_t = c_uint;
pub const DC_SAMPLE_TIME: dc_sample_type_t = 0;
pub const DC_SAMPLE_DEPTH: dc_sample_type_t = 1;
pub const DC_SAMPLE_TEMPERATURE: dc_sample_type_t = 3;

#[repr(C)]
pub struct dc_context_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct dc_descriptor_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct dc_iterator_t {
    _private: [u8; 0],
}
#[repr(C)]
pub struct dc_parser_t {
    _private: [u8; 0],
}

/// `dc_sample_value_t` is a C union. Every member DiveLens reads starts at
/// offset 0: `time` is an `unsigned int` (milliseconds), `depth` (metres) and
/// `temperature` (°C) are doubles. Read them through the pointer with the
/// matching type only.
pub type dc_sample_value_t = c_void;

pub type dc_sample_callback_t = Option<
    unsafe extern "C" fn(
        kind: dc_sample_type_t,
        value: *const dc_sample_value_t,
        userdata: *mut c_void,
    ),
>;

unsafe extern "C" {
    pub fn dc_context_new(context: *mut *mut dc_context_t) -> dc_status_t;
    pub fn dc_context_free(context: *mut dc_context_t) -> dc_status_t;

    pub fn dc_descriptor_iterator_new(
        iterator: *mut *mut dc_iterator_t,
        context: *mut dc_context_t,
    ) -> dc_status_t;
    pub fn dc_iterator_next(iterator: *mut dc_iterator_t, item: *mut c_void) -> dc_status_t;
    pub fn dc_iterator_free(iterator: *mut dc_iterator_t) -> dc_status_t;
    pub fn dc_descriptor_free(descriptor: *mut dc_descriptor_t);
    pub fn dc_descriptor_get_vendor(descriptor: *const dc_descriptor_t) -> *const c_char;
    pub fn dc_descriptor_get_product(descriptor: *const dc_descriptor_t) -> *const c_char;

    pub fn dc_parser_new2(
        parser: *mut *mut dc_parser_t,
        context: *mut dc_context_t,
        descriptor: *mut dc_descriptor_t,
        data: *const u8,
        size: usize,
    ) -> dc_status_t;
    pub fn dc_parser_samples_foreach(
        parser: *mut dc_parser_t,
        callback: dc_sample_callback_t,
        userdata: *mut c_void,
    ) -> dc_status_t;
    pub fn dc_parser_destroy(parser: *mut dc_parser_t) -> dc_status_t;
}
