//! The controller trait.
use cubism_core::Model;

/// The controller trait. A controller is an object that modifies a models
/// parameter and part values in a defined fashion.
pub trait Controller {
    /// Update the passed in models parameters according to this controller.
    fn update_parameters(&mut self, model: &mut Model, delta: f32);
}
