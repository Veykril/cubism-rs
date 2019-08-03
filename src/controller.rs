use cubism_core::Model;

pub trait Controller {
    fn update_parameters(&mut self, model: &mut Model, delta: f32);
}
