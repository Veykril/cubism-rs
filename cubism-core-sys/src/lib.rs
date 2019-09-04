#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![allow(clippy::identity_op)]

pub mod moc;
pub mod model;

pub use self::{moc::*, model::*};

use std::os::raw::{c_char, c_float, c_uint};

pub type csmVersion = c_uint;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct csmVector2 {
    pub x: c_float,
    pub y: c_float,
}

extern "C" {
    pub fn csmGetVersion() -> csmVersion;
}

pub type csmLogFunction = Option<unsafe extern "C" fn(message: *const c_char)>;

extern "C" {
    pub fn csmGetLogFunction() -> csmLogFunction;
    pub fn csmSetLogFunction(handler: csmLogFunction);
}
