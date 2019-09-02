use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pose3 {
    #[serde(rename = "Type")]
    pub file_type: String,
    pub groups: Vec<PoseGroup>,
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
        serde_json::from_str::<Pose3>(
            &std::fs::read_to_string(&path.join([model, "/", model, ".pose3.json"].concat()))
                .unwrap(),
        )
        .unwrap();
    }
}
