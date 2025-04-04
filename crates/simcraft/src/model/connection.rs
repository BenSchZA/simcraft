use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    #[serde(rename = "sourceID")]
    pub source_id: String,
    pub source_port: Option<String>,
    #[serde(rename = "targetID")]
    pub target_id: String,
    pub target_port: Option<String>,
    pub flow_rate: Option<f64>,
    #[serde(default)]
    pub sequence_number: u64,
}

impl Connection {
    pub fn new(
        id: String,
        source_id: String,
        source_port: Option<String>,
        target_id: String,
        target_port: Option<String>,
        flow_rate: Option<f64>,
    ) -> Self {
        Self {
            id,
            source_id,
            source_port,
            target_id,
            target_port,
            flow_rate,
            sequence_number: 0,
        }
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn source_port(&self) -> Option<&str> {
        self.source_port.as_deref()
    }

    pub fn target_id(&self) -> &str {
        &self.target_id
    }

    pub fn target_port(&self) -> Option<&str> {
        self.target_port.as_deref()
    }
}
