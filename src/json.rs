#![allow(missing_docs)]

pub mod cdi;
pub mod expression;
pub mod model;
pub mod motion;
pub mod physics;
pub mod pose;
pub mod user_data;

/// Utility function to map non-positive floats to 1.0 after deserialization
pub(self) fn de_fade_time<'de, D>(d: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    f32::deserialize(d).map(|val| if val <= 0.0 { 1.0 } else { val })
}

pub(self) const fn float_1() -> f32 {
    1.0
}
