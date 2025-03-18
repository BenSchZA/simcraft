use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use crate::model::ProcessState;

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub step: usize,
    pub time: f64,
    pub process_states: HashMap<String, ProcessState>,
}

pub type SimulationResults = Vec<SimulationState>;
