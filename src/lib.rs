#![deny(missing_docs, bare_trait_objects)]
#![warn(clippy::all)]

//! A framework for Live2D's cubism sdk
pub use cubism_core as core;

pub mod controller;
pub mod error;
pub mod expression;
pub mod id;
pub mod json;
pub mod model;
pub mod motion;
pub(crate) mod util;
