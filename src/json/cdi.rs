/// Parses .cdi3.json.
use serde::{Deserialize, Serialize};

use std::str::FromStr;

/// Rust structure representation for .cdi3.json file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cdi3 {
    pub version: usize,
    #[serde(default)]
    pub parameters: Vec<Cdi3Parameter>,
    #[serde(default)]
    pub parameter_groups: Vec<Cdi3ParameterGroup>,
    #[serde(default)]
    pub parts: Vec<Cdi3Part>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cdi3Parameter {
    id: String,
    #[serde(default)]
    group_id: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cdi3ParameterGroup {
    id: String,
    #[serde(default)]
    group_id: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Cdi3Part {
    id: String,
    name: String,
}

impl Cdi3 {
    /// Parses a Cdi3 from a .cdi3.json reader.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

impl FromStr for Cdi3 {
    type Err = serde_json::Error;

    /// Parses a Cdi3 from a .cdi3.json string.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[test]
fn json_samples_cdi3() {
    use std::iter::FromIterator;
    let path =
        std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res/Rice/Rice.cdi3.json"]);

    Cdi3::from_str(
        &std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("error while reading {:?}: {:?}", &path, e)),
    )
    .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &path, e));
}
