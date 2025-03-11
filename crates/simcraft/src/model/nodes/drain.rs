use derive_builder::Builder;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::{Action, TriggerMode};
use crate::{
    model::{
        process_state::{DrainState, ProcessState},
        process_trait::Processor,
    },
    simulator::{
        event::{Event, EventPayload},
        SimulationContext,
    },
    utils::errors::SimulationError,
};

#[derive(Builder, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Drain {
    #[builder(setter(into))]
    id: String,
    state: DrainState,
    trigger_mode: TriggerMode,
    action: Action,
}

impl Default for Drain {
    fn default() -> Self {
        Self {
            id: String::new(),
            state: DrainState::default(),
            trigger_mode: TriggerMode::Automatic,
            action: Action::PullAny,
        }
    }
}

impl Drain {
    pub fn new(id: &str) -> Drain {
        Drain::builder().id(id.to_string()).build().unwrap()
    }

    pub fn builder() -> DrainBuilder {
        DrainBuilder::default()
    }

    fn handle_step(&mut self, context: &mut SimulationContext) -> Vec<Event> {
        debug!("Drain '{}' handling step", self.id());
        let mut new_events = Vec::new();

        if self.trigger_mode == TriggerMode::Automatic {
            match self.action {
                Action::PullAny => self.handle_pull_any(context, &mut new_events),
                Action::PullAll => self.handle_pull_all(context, &mut new_events),
                // TODO Handle incorrect action at compile and run time
                _ => debug!(
                    "Drain '{}' has unsupported action: {:?}",
                    self.id(),
                    self.action
                ),
            }
        }

        new_events
    }

    fn handle_pull_any(&mut self, context: &mut SimulationContext, new_events: &mut Vec<Event>) {
        // Pull whatever is available up to flow rates from each input
        for conn in context.get_inputs(self.id(), Some("in")) {
            let flow_rate = conn.flow_rate.unwrap_or(1.0);
            new_events.push(Event {
                time: context.current_time(),
                source_id: self.id().to_string(),
                source_port: None,
                target_id: conn.source_id.clone(),
                target_port: None,
                payload: EventPayload::PullRequest(flow_rate),
            });
        }
    }

    fn handle_pull_all(&mut self, context: &mut SimulationContext, new_events: &mut Vec<Event>) {
        // Calculate total requested resources
        let inputs = context.get_inputs(self.id(), Some("in"));
        let total_requested: f64 = inputs
            .iter()
            .map(|conn| conn.flow_rate.unwrap_or(1.0))
            .sum();

        // Request all resources - will only receive if all are available
        for conn in inputs {
            let flow_rate = conn.flow_rate.unwrap_or(1.0);
            new_events.push(Event {
                time: context.current_time(),
                source_id: self.id().to_string(),
                source_port: None,
                target_id: conn.source_id.clone(),
                target_port: None,
                payload: EventPayload::PullAllRequest {
                    amount: flow_rate,
                    total_required: total_requested,
                },
            });
        }
    }
}

impl Processor for Drain {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &mut SimulationContext,
    ) -> Result<Vec<Event>, SimulationError> {
        debug!("Drain '{}' handling event: {:?}", self.id(), event.payload);
        let mut new_events = Vec::new();

        match &event.payload {
            EventPayload::Step => {
                new_events.extend(self.handle_step(context));
            }
            EventPayload::Resource(amount) => {
                info!(
                    "{}: Drain '{}' consuming {} resources from '{}'",
                    context.current_time(),
                    self.id(),
                    amount,
                    event.source_id
                );
                self.state.resources_consumed += amount;
            }
            _ => {
                debug!("Drain '{}' ignoring unhandled event type", self.id());
            }
        }

        Ok(new_events)
    }

    fn get_state(&self) -> ProcessState {
        ProcessState::Drain(self.state.clone())
    }

    fn get_input_ports(&self) -> Vec<String> {
        vec!["in".to_string()]
    }

    fn get_output_ports(&self) -> Vec<String> {
        vec![] // Drain has no outputs
    }
}
