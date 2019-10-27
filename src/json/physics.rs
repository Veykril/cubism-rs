/// Parses .physics3.json.
use serde::{Deserialize, Serialize};

use std::str::FromStr;

/// Rust structure representation for .physics3.json file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Physics3 {
    version: usize,
    meta: Physics3Meta,
    physics_settings: Vec<PhysicsSetting>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsSetting {
    id: String,
    #[serde(default)]
    input: Vec<PhysicsInput>,
    #[serde(default)]
    output: Vec<PhysicsOutput>,
    #[serde(default)]
    vertices: Vec<PhysicsVertex>,
    normalization: Option<PhysicsNormalization>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsInput {
    source: PhysicsTarget,
    weight: f32,
    #[serde(rename = "Type")]
    ty: String,
    reflect: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsOutput {
    destination: PhysicsTarget,
    vertex_index: usize,
    scale: f32,
    weight: f32,
    #[serde(rename = "Type")]
    ty: String,
    reflect: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsVertex {
    position: Vec2D,
    mobility: f32,
    delay: f32,
    acceleration: f32,
    radius: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsNormalization {
    position: PhysicsNormalizationParameter,
    angle: PhysicsNormalizationParameter,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsNormalizationParameter {
    minimum: f32,
    maximum: f32,
    default: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsTarget {
    target: String,
    id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Physics3Meta {
    total_input_count: usize,
    total_output_count: usize,
    vertex_count: usize,
    physics_setting_count: usize,
    effective_forces: EffectiveForces,
    physics_dictionary: Vec<PhysicsIdName>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsIdName {
    id: String,
    name: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EffectiveForces {
    #[serde(default)]
    gravity: Vec2D,
    #[serde(default)]
    wind: Vec2D,
}

#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Vec2D {
    x: f32,
    y: f32,
}

impl Physics3 {
    /// Parses a Physics3 from a .physics3.json reader.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

impl FromStr for Physics3 {
    type Err = serde_json::Error;

    /// Parses a Physics3 from a .physics3.json string.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[test]
fn json_samples_physics3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Rice", "Hiyori", "Mark", "Natori"] {
        let path = path.join([model, "/", model, ".physics3.json"].concat());

        Physics3::from_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("error while reading {:?}: {:?}", &path, e)),
        )
        .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &path, e));
    }
}
