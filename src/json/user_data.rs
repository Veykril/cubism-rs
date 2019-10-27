use serde::{Deserialize, Serialize};

use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserData3 {
    pub version: usize,
    pub meta: Meta,
    pub user_data: Vec<UserData>,
}

impl UserData3 {
    /// Parses a UserData3 from a .userdata3.json reader.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

impl FromStr for UserData3 {
    type Err = serde_json::Error;

    /// Parses a UserData3 from a .userdata3.json string.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Meta {
    pub user_data_count: usize,
    pub total_user_data_size: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserData {
    pub target: UserDataTarget,
    pub id: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum UserDataTarget {
    ArtMesh,
}

#[test]
fn json_samples_userdata3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Haru", "Hiyori", "Mark"] {
        let userdata_path = path.join([model, "/", model, ".userdata3.json"].concat());
        UserData3::from_str(
            &std::fs::read_to_string(&userdata_path)
                .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &userdata_path, e)),
        )
        .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &userdata_path, e));
    }
}
