use derive_builder::Builder;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::{Action, TriggerMode};
use crate::{
    model::{
        process_state::{ProcessState, SourceState},
        ProcessContext, Processor, SerializableProcess,
    },
    simulator::event::{Event, EventPayload},
    utils::errors::SimulationError,
};

#[derive(Builder, Debug, Clone, Serialize, Deserialize, SerializableProcess)]
#[serde(default, rename_all = "camelCase")]
#[builder(default)]
pub struct Source {
    #[builder(setter(into))]
    id: String,
    state: SourceState,
    trigger_mode: TriggerMode,
    action: Action,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            id: String::new(),
            state: SourceState::default(),
            trigger_mode: TriggerMode::Automatic,
            action: Action::PushAny,
        }
    }
}

impl Source {
    pub fn new(id: &str) -> Source {
        Source::builder().id(id.to_string()).build().unwrap()
    }

    pub fn builder() -> SourceBuilder {
        SourceBuilder::default()
    }

    fn handle_step(&mut self, context: &ProcessContext) -> Vec<Event> {
        debug!("Source '{}' handling step", self.id());
        let mut new_events = Vec::new();

        if self.trigger_mode == TriggerMode::Automatic {
            match self.action {
                Action::PushAny => self.handle_push_any(context, &mut new_events),
                _ => debug!(
                    "Source '{}' has unsupported action: {:?}",
                    self.id(),
                    self.action
                ),
            }
        }

        new_events
    }

    fn handle_push_any(&mut self, context: &ProcessContext, new_events: &mut Vec<Event>) {
        let outputs = context.outputs_for_port(Some("out"));

        for conn in outputs {
            let amount = conn.flow_rate.unwrap_or(1.0);

            info!(
                "{}: Source '{}' pushing {} resources to '{}'",
                context.current_time(),
                self.id(),
                amount,
                conn.target_id
            );

            new_events.push(Event {
                time: context.current_time(),
                source_id: self.id().to_string(),
                source_port: Some("out".to_string()),
                target_id: conn.target_id.clone(),
                target_port: conn.target_port.clone(),
                payload: EventPayload::Resource(amount),
            });
            self.state.resources_produced += amount;
        }
    }

    fn handle_pull_request(
        &mut self,
        event: &Event,
        context: &ProcessContext,
        amount: f64,
    ) -> Event {
        info!(
            "{}: Source '{}' pushing {} resources to '{}'",
            context.current_time(),
            self.id(),
            amount,
            event.source_id
        );

        debug!(
            "Source '{}' handling pull request for {}",
            self.id(),
            amount
        );

        self.state.resources_produced += amount;
        Event {
            time: context.current_time(),
            source_id: self.id().to_string(),
            source_port: Some("out".to_string()),
            target_id: event.source_id.clone(),
            target_port: event.source_port.clone(),
            payload: EventPayload::Resource(amount),
        }
    }
}

impl Processor for Source {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        debug!("Source '{}' handling event: {:?}", self.id(), event.payload);
        let mut new_events = Vec::new();

        match &event.payload {
            EventPayload::Step => {
                new_events.extend(self.handle_step(context));
            }
            EventPayload::PullRequest(amount) => {
                new_events.push(self.handle_pull_request(event, context, *amount));
            }
            EventPayload::PullAllRequest { amount, .. } => {
                new_events.push(self.handle_pull_request(event, context, *amount));
            }
            _ => {
                debug!("Source '{}' ignoring unhandled event type", self.id());
            }
        }

        debug!(
            "Source '{}' generated {} new events",
            self.id(),
            new_events.len()
        );
        Ok(new_events)
    }

    fn get_state(&self) -> ProcessState {
        ProcessState::Source(self.state.clone())
    }

    fn get_input_ports(&self) -> Vec<String> {
        vec![] // Source has no inputs
    }

    fn get_output_ports(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}
