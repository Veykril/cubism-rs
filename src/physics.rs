use crate::core::Model;
use crate::json::physics::Physics3;

pub struct Physics {
    wind: (f32, f32),
    gravity: (f32, f32),
    rig: PhysicsRig,
}

impl Physics {
    pub fn from_physics3(phys3: Physics3) -> Self {
        Physics {
            wind: (0.0, 0.0),
            gravity: (0.0, -1.0),
            rig: PhysicsRig::from_physics3(phys3),
        }
    }

    pub fn update(&self, model: &Model, delta: f32) {}
}
