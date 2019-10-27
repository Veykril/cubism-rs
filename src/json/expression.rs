// Parses .exp3.json.

use serde::{Deserialize, Serialize};

use std::str::FromStr;

/// Rust structure representation for .exp3.json file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Expression3 {
    #[serde(rename = "Type")]
    pub ty: String,
    #[serde(deserialize_with = "super::de_fade_time", default = "super::float_1")]
    pub fade_in_time: f32,
    #[serde(deserialize_with = "super::de_fade_time", default = "super::float_1")]
    pub fade_out_time: f32,
    pub parameters: Vec<ExpressionParameter>,
}

impl Expression3 {
    /// Parses a Expression3 from a .expression3.json reader.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

impl FromStr for Expression3 {
    type Err = serde_json::Error;

    /// Parses a Expression3 from a .expression3.json string.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ExpressionBlendType {
    Add = 0x00,
    Multiply = 0x01,
    Overwrite = 0x02,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExpressionParameter {
    pub id: String,
    #[serde(rename = "Blend")]
    pub blend_type: ExpressionBlendType,
    pub value: f32,
}

#[test]
fn json_samples_exp3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Haru/expressions", "Natori/exp"] {
        let exp_path = path.join(model);
        let expressions = std::fs::read_dir(exp_path).unwrap();

        for exp in expressions {
            let exp_path = exp.unwrap().path();

            if !exp_path.is_file() {
                continue;
            }

            Expression3::from_str(
                &std::fs::read_to_string(&exp_path)
                    .unwrap_or_else(|e| panic!("error while reading {:?}: {:?}", &exp_path, e)),
            )
            .unwrap_or_else(|e| panic!("error while parsing {:?}: {:?}", &exp_path, e));
        }
    }
}
