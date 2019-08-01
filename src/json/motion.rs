//! Parses .motion3.json.
#![deny(missing_docs)]
use serde::{Deserialize, Serialize};

/// Rust structure representation for Motion3 metadata.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Meta {
    /// Duration of a motion.
    pub duration: f32,
    /// Frame per second.
    pub fps: f32,
    #[serde(rename = "Loop")]
    /// True if the motion is looped.
    pub looped: bool,
    /// TODO:
    #[serde(rename = "AreBeziersRestricted")]
    pub restricted_beziers: bool,
    /// A number of curves that the motion3.json file has.
    #[serde(rename = "CurveCount")]
    pub len_curves: usize,
    /// A number of segments that the motion3.json file has.
    #[serde(rename = "TotalSegmentCount")]
    pub len_segments: usize,
    /// A number of points that the motion3.json file has.
    #[serde(rename = "TotalPointCount")]
    pub len_points: usize,
    /// A number of user data fields that the motion3.json file has.
    #[serde(rename = "UserDataCount")]
    pub len_user_data: usize,
    /// A total size of user data.
    #[serde(rename = "TotalUserDataSize")]
    pub size_user_data: usize,
}

/// Rust structure representation for Motion3 curve data.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Curve {
    /// Target.
    pub target: String,
    /// Id.
    pub id: String,
    /// Segments.
    pub segments: Vec<f32>, // TODO: more higher-level parser for motion segments
    /// Fade-in time. 1.0 [sec] as default.
    #[serde(default = "fade_time_default")]
    pub fade_in_time: f32,
    /// Fade-out time. 1.0 [sec] as default.
    #[serde(default = "fade_time_default")]
    pub fade_out_time: f32,
}

fn fade_time_default() -> f32 {
    1.0
}

/// Rust structure representation for Motion3.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Motion3 {
    /// Version.
    pub version: u32,
    /// Metadata.
    pub meta: Meta,
    /// Curves.
    pub curves: Vec<Curve>,
}

impl Motion3 {
    /// Parses a .motion3.json file as a string and returns a Motion3 structure.
    #[inline]
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    /// Reads .motion3.json data from a reader and returns a Motion3 structure.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

#[test]
fn json_samples_motion3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Haru", "Hiyori", "Mark", "Natori"] {
        let motion_path = path.join([model, "/motions/"].concat());
        let motions = std::fs::read_dir(motion_path).unwrap();

        for motion in motions {
            let motion = motion.unwrap().path();

            if !motion.is_file() {
                continue;
            }

            serde_json::from_str::<Motion3>(
                &std::fs::read_to_string(&motion)
                    .expect(&format!("error while reading: {:?}", motion)),
            )
            .expect(&format!("error while parsing: {:?}", motion));
        }
    }
}
