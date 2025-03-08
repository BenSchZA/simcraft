use log::{debug, error, trace};
use std::fmt::Debug;

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
        debug!(
            "Process '{}' received event at time {}",
            self.id(),
            event.time
        );
        trace!("Event details: {:?}", event);

        let result = self.inner.on_event(event, context);

        match &result {
            Ok(new_events) => {
                debug!(
                    "Process '{}' generated {} new events",
                    self.id(),
                    new_events.len()
                );
                trace!("Generated events: {:?}", new_events);
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
