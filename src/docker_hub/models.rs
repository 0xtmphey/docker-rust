use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: u64,
    pub issued_at: String,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub media_type: String,
    pub size: usize,
    pub digest: String,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub media_type: String,
    pub size: usize,
    pub digest: String,
    pub urls: Option<Vec<String>>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: usize,
    pub media_type: String,
    pub config: Config,
    pub layers: Vec<Layer>,
}
