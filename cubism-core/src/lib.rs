#![deny(missing_docs, rust_2018_idioms)]
#![warn(
    clippy::all,
    missing_copy_implementations,
    missing_debug_implementations
)]

//! Rust bindings for Live2D's cubism sdk

mod error;
mod log;
mod mem;
mod moc;
mod model;

pub use crate::{error::*, log::*, moc::*, model::*};

/// Returns the linked library version in a (major, minor, patch) tuple
pub fn version() -> (u8, u8, u16) {
    let version = unsafe { ffi::csmGetVersion() };
    let major = (version & 0xFF00_0000) >> 24;
    let minor = (version & 0x00FF_0000) >> 16;
    let patch = version & 0xFFFF;
    (major as u8, minor as u8, patch as u16)
}

bitflags::bitflags! {
    /// The constant flags of a [Model](model/struct.Model.html)'s drawable.
    pub struct ConstantFlags: u8 {
        /// The drawable should be blended additively.
        const BLEND_ADDITIVE = ffi::csmBlendAdditive;
        /// The drawable should be blended multiplicatively.
        const BLEND_MULTIPLICATIVE = ffi::csmBlendMultiplicative;
        /// The drawable is double sided and therefore shouldn't be culled.
        const IS_DOUBLE_SIDED = ffi::csmIsDoubleSided;
        /// Whether the clipping mask is inverted or not.
        const IS_INVERTED_MASK = ffi::csmIsInvertedMask;
    }
}

bitflags::bitflags! {
    /// The dynamic flags of a [Model](model/struct.Model.html)'s drawable.
    pub struct DynamicFlags: u8 {
        /// The drawable is visible.
        const IS_VISIBLE = ffi::csmIsVisible;
        /// The drawable's visibility changed since the last update.
        const VISIBILITY_CHANGED = ffi::csmVisibilityDidChange;
        /// The drawable's opacity changed since the last update.
        const OPACITY_CHANGED = ffi::csmOpacityDidChange;
        /// The drawable's drawing order changed since the last update.
        const DRAW_ORDER_CHANGED = ffi::csmDrawOrderDidChange;
        /// The drawable's render order changed since the last update.
        const RENDER_ORDER_CHANGED = ffi::csmRenderOrderDidChange;
        /// The drawable's vertex positions changed since the last update.
        const VERTEX_POSITIONS_CHANGED = ffi::csmVertexPositionsDidChange;
    }
}
