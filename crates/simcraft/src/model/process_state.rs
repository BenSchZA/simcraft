use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct StepperState {
    pub current_step: usize,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    pub resources: f64,
    pub pending_outgoing_resources: f64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SourceState {
    pub resources_produced: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DrainState {
    pub resources_consumed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessState {
    Source(SourceState),
    Pool(PoolState),
    Drain(DrainState),
    Stepper(StepperState),
    Custom(Value),
}
