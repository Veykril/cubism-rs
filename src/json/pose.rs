use serde::{Deserialize, Serialize};

use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pose3 {
    #[serde(rename = "Type")]
    pub ty: String,
    pub groups: Vec<PoseGroup>,
    #[serde(default = "Pose3::fade_in_time_default")]
    pub fade_in_time: f32,
}

impl Pose3 {
    /// Parses a Pose3 from a .pose3.json reader.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }

    fn fade_in_time_default() -> f32 {
        0.5
    }
}

impl FromStr for Pose3 {
    type Err = serde_json::Error;

    /// Parses a Pose3 from a .pose3.json string.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoseItem {
    pub id: String,
    pub link: Vec<String>,
}

type PoseGroup = Vec<PoseItem>;

#[test]
fn json_samples_pose3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Haru", "Hiyori", "Natori"] {
        let pose_path = path.join([model, "/", model, ".pose3.json"].concat());
        Pose3::from_str(
            &std::fs::read_to_string(&pose_path)
                .unwrap_or_else(|e| panic!("error while reading {:?}: {:?}", &pose_path, e)),
        )
        .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &pose_path, e));
    }
}
