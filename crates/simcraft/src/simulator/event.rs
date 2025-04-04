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
    PullRequest,
    PullAllRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub source_id: String,
    pub source_port: Option<String>,
    pub target_id: String,
    pub target_port: Option<String>,
    pub time: f64,
    pub payload: EventPayload,
    pub sequence_number: u64,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by time (reversed for min-heap)
        match other.time.partial_cmp(&self.time).unwrap() {
            // If the times are equal, compare by sequence number
            Ordering::Equal => other.sequence_number.cmp(&self.sequence_number),
            ord => ord,
        }
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

impl Event {
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        time: f64,
        payload: EventPayload,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
            time,
            payload,
            source_port: None,
            target_port: None,
            sequence_number: 0,
        }
    }

    pub fn with_source_port(mut self, source_port: impl Into<String>) -> Self {
        self.source_port = Some(source_port.into());
        self
    }

    pub fn with_target_port(mut self, target_port: impl Into<String>) -> Self {
        self.target_port = Some(target_port.into());
        self
    }

    pub fn with_ports(
        mut self,
        source_port: impl Into<String>,
        target_port: impl Into<String>,
    ) -> Self {
        self.source_port = Some(source_port.into());
        self.target_port = Some(target_port.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use super::*;

    #[test]
    fn test_event_ordering() {
        let mut events = BinaryHeap::new();

        events.extend([
            Event {
                source_id: "test".into(),
                target_id: "test".into(),
                time: 2.0,
                payload: EventPayload::Step,
                source_port: None,
                target_port: None,
                sequence_number: 1,
            },
            Event {
                source_id: "test".into(),
                target_id: "test".into(),
                time: 1.0,
                payload: EventPayload::Step,
                source_port: None,
                target_port: None,
                sequence_number: 2,
            },
            Event {
                source_id: "test".into(),
                target_id: "test".into(),
                time: 1.0,
                payload: EventPayload::Step,
                source_port: None,
                target_port: None,
                sequence_number: 1,
            },
        ]);


        // Events should be ordered by:
        // 1. Earlier time first (1.0 before 2.0) due to min-heap ordering
        // 2. For same time, lower sequence number first
        let event_1 = events.pop().unwrap();
        let event_2 = events.pop().unwrap();
        let event_3 = events.pop().unwrap();

        assert_eq!(event_1.time, 1.0);
        assert_eq!(event_1.sequence_number, 1);

        assert_eq!(event_2.time, 1.0);
        assert_eq!(event_2.sequence_number, 2);

        assert_eq!(event_3.time, 2.0);
        assert_eq!(event_3.sequence_number, 1);
    }
}
