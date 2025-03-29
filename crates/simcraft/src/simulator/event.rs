use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventPayload {
    SimulationStart,
    SimulationEnd,
    Step,
    Trigger, // Triggers a TriggerMode::Passive node to fire
    Resource(f64),
    ResourceAccepted(f64),
    ResourceRejected(f64),
    Custom(String),
    PullRequest(f64),
    PullAllRequest { amount: f64, total_required: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub source_id: String,
    pub source_port: Option<String>,
    pub target_id: String,
    pub target_port: Option<String>,
    pub time: f64,
    pub payload: EventPayload,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.partial_cmp(&self.time).unwrap()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Event {}

impl Event {}
