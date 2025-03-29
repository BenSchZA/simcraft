use derive_builder::Builder;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::{Action, TriggerMode};
use crate::{
    model::{
        process_state::{DrainState, ProcessState},
        Connection, ProcessContext, Processor, SerializableProcess,
    },
    simulator::event::{Event, EventPayload},
    utils::errors::SimulationError,
};

#[derive(Builder, Debug, Clone, Serialize, Deserialize, SerializableProcess)]
#[serde(default, rename_all = "camelCase")]
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

    fn handle_step(&mut self, context: &ProcessContext) -> Result<Vec<Event>, SimulationError> {
        debug!("Drain '{}' handling step", self.id());

        let new_events = match (self.trigger_mode, self.action) {
            (TriggerMode::Automatic, Action::PullAny) => self.handle_pull_any(context)?,
            (TriggerMode::Automatic, Action::PullAll) => self.handle_pull_all(context)?,
            (TriggerMode::Automatic, other_action) => {
                debug!(
                    "Drain '{}' has unsupported automatic action: {:?}",
                    self.id(),
                    other_action
                );
                vec![]
            }
            // TODO Handle incorrect action at compile and run time
            _ => vec![], // Passive, Interactive, etc.
        };

        Ok(new_events)
    }

    fn handle_pull_any(&mut self, context: &ProcessContext) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        // Pull whatever is available up to flow rates from each input
        for conn in context.inputs_for_port(Some("in")) {
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

        Ok(new_events)
    }

    fn handle_pull_all(&mut self, context: &ProcessContext) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        // Calculate total requested resources
        let inputs: Vec<&Connection> = context.outputs_for_port(Some("in")).collect();
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

        Ok(new_events)
    }
}

impl Processor for Drain {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        debug!("Drain '{}' handling event: {:?}", self.id(), event.payload);

        let new_events: Vec<Event> = match &event.payload {
            EventPayload::Step => self.handle_step(context)?,
            EventPayload::Resource(amount) => {
                info!(
                    "{}: Drain '{}' consuming {} resources from '{}'",
                    context.current_time(),
                    self.id(),
                    amount,
                    event.source_id
                );
                self.state.resources_consumed += amount;

                vec![Event {
                    time: context.current_time(),
                    source_id: self.id().to_string(),
                    source_port: None,
                    target_id: event.source_id.clone(),
                    target_port: None,
                    payload: EventPayload::ResourceAccepted(*amount),
                }]
            }
            _ => {
                debug!("Drain '{}' ignoring unhandled event type", self.id());
                vec![]
            }
        };

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
