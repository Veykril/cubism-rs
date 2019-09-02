#![warn(
    bare_trait_objects,
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::all
)]

//! A framework for Live2D's cubism sdk
pub use cubism_core as core;

pub mod controller;
pub mod effect;
pub mod error;
pub mod expression;
pub mod id;
pub mod json;
pub mod model;
pub mod motion;
