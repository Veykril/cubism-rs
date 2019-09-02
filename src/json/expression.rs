use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Expression3 {
    #[serde(rename = "Type")]
    pub file_type: String,
    pub parameters: Vec<ExpressionParameter>,
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

impl Expression3 {
    /// Reads .exp3.json data from a reader and returns a Expression3 structure.
    #[inline]
    pub fn from_reader<R: std::io::Read>(r: R) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

impl FromStr for Expression3 {
    type Err = serde_json::Error;

    // Parses a .exp3.json file as a string and returns a Expression3 structure.
    #[inline]
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[test]
fn json_samples_exp3() {
    use std::iter::FromIterator;
    let path = std::path::PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
    for model in &["Haru/expressions", "Natori/exp"] {
        let exp_path = path.join(model);
        let expressions = std::fs::read_dir(exp_path).unwrap();

        for exp in expressions {
            let exp = exp.unwrap().path();

            if !exp.is_file() {
                continue;
            }

            serde_json::from_str::<Expression3>(
                &std::fs::read_to_string(&exp)
                    .unwrap_or_else(|_| panic!("error while reading: {:?}", exp)),
            )
            .unwrap_or_else(|e| panic!("error while parsing: {:?},  with error:\n{:#?}", exp, e));
        }
    }
}
