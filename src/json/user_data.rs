use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserData3 {
    pub version: usize,
    pub meta: Meta,
    #[serde(default, rename = "UserData")]
    pub user_data: Vec<UserData>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    #[serde(rename = "UserDataCount")]
    pub user_data_count: usize,
    #[serde(rename = "TotalUserDataSize")]
    pub total_user_data_size: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserData {
    pub target: String,
    pub id: String,
    pub value: String,
}
