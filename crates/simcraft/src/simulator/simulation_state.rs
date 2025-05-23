use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use crate::model::ProcessState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub step: u64,
    pub time: f64,
    pub process_states: HashMap<String, ProcessState>,
}
