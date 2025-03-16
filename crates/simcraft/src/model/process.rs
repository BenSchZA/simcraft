use log::{debug, error, info};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Debug;

use super::process_trait::SerializableProcess;
use super::{process_state::ProcessState, process_trait::Processor};
use crate::{
    simulator::{Event, SimulationContext},
    utils::errors::SimulationError,
};

#[derive(Clone, Debug)]
pub struct Process {
    inner: Box<dyn Processor + Send>,
}

impl Process {
    pub fn new(inner: Box<dyn Processor + Send>) -> Self {
        Self { inner }
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        // TODO Expand equality check to include configuration
        self.inner.id() == other.inner.id() && self.inner.get_type() == other.inner.get_type()
    }
}

impl Serialize for Process {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let extra_fields: serde_yaml::Value = self.inner.serialize();
        let mut process = serializer.serialize_map(None)?;
        process.serialize_entry("id", &self.inner.id())?;
        process.serialize_entry("type", self.inner.get_type())?;
        if let serde_yaml::Value::Mapping(map) = extra_fields {
            for (key, value) in map.iter() {
                process.serialize_entry(&key, &value)?;
            }
        }
        process.end()
    }
}

impl<'de> Deserialize<'de> for Process {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let process_repr = super::ProcessRepr::deserialize(deserializer)?;
        let process = super::process_factory::create_process::<D>(
            &process_repr.process_type[..],
            process_repr.extra,
        )?;
        Ok(Process::new(process))
    }
}

impl SerializableProcess for Process {}

impl Processor for Process {
    fn id(&self) -> &str {
        // TODO Enforce that id is set
        self.inner.id()
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &mut SimulationContext,
    ) -> Result<Vec<Event>, SimulationError> {
        info!(
            "Process '{}' received event at time {}",
            self.id(),
            event.time
        );
        debug!("Event details: {:?}", event);

        let result = self.inner.on_event(event, context);

        match &result {
            Ok(new_events) => {
                info!(
                    "Process '{}' generated {} new events",
                    self.id(),
                    new_events.len()
                );
                debug!("Generated events: {:?}", new_events);
            }
            Err(e) => {
                error!("Process '{}' failed to handle event: {}", self.id(), e);
            }
        }

        result
    }

    fn get_state(&self) -> ProcessState {
        self.inner.get_state()
    }

    fn get_input_ports(&self) -> Vec<String> {
        self.inner.get_input_ports()
    }

    fn get_output_ports(&self) -> Vec<String> {
        self.inner.get_output_ports()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::nodes::Source;

    #[test]
    fn test_serialization_with_defaults() {
        let process = Process::new(Box::new(Source::new("source-1")));
        let serialized = serde_json::to_string(&process).unwrap();

        let expected = r#"
            {
                "id": "source-1",
                "type": "Source"
            }
        "#;

        let expected_json = serde_json::from_str::<Process>(expected).unwrap();
        let serialized_json = serde_json::from_str::<Process>(serialized.as_str()).unwrap();

        assert_eq!(expected_json.id(), serialized_json.id());
    }
}
