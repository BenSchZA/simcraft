use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessRepr {
    #[serde(rename = "type")]
    pub process_type: String,
    #[serde(flatten)]
    pub extra: serde_yaml::Value,
}
