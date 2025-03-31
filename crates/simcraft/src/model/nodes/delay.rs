use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    model::{
        process_state::{DelayState, ProcessState},
        ProcessContext, Processor, SerializableProcess,
    },
    simulator::event::{Event, EventPayload},
    utils::errors::SimulationError,
};

use super::{DelayAction, TriggerMode};

#[derive(Builder, Debug, Clone, Serialize, Deserialize, SerializableProcess)]
#[serde(default, rename_all = "camelCase")]
#[builder(default)]
pub struct Delay {
    #[builder(setter(into))]
    id: String,
    state: DelayState,
    trigger_mode: TriggerMode,
    action: DelayAction,
    release_amount: f64, // Only used in Queue mode
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            id: String::new(),
            state: DelayState::default(),
            trigger_mode: TriggerMode::Automatic,
            action: DelayAction::Delay,
            release_amount: 1.0,
        }
    }
}

impl Delay {
    pub fn new(id: &str) -> Delay {
        Delay::builder().id(id.to_string()).build().unwrap()
    }

    pub fn builder() -> DelayBuilder {
        DelayBuilder::default()
    }

    fn handle_resource(
        &mut self,
        event: &Event,
        context: &ProcessContext,
        amount: f64,
    ) -> Result<Vec<Event>, SimulationError> {
        assert!(amount >= 0.0);

        let mut outputs = context.outputs_for_port(Some("out"));

        let Some(conn) = outputs.next() else {
            warn!("No output connection - rejecting resources");
            return Ok(vec![Event {
                time: context.current_time(),
                source_id: self.id.clone(),
                source_port: None,
                target_id: event.source_id.clone(),
                target_port: None,
                payload: EventPayload::ResourceRejected(amount),
            }]);
        };

        if outputs.next().is_some() {
            warn!("Multiple output connections - rejecting resources");
            return Ok(vec![Event {
                time: context.current_time(),
                source_id: self.id.clone(),
                source_port: None,
                target_id: event.source_id.clone(),
                target_port: None,
                payload: EventPayload::ResourceRejected(amount),
            }]);
        }

        self.state.resources_received += amount;

        let mut new_events = vec![Event {
            time: context.current_time(),
            source_id: self.id.clone(),
            source_port: None,
            target_id: event.source_id.clone(),
            target_port: None,
            payload: EventPayload::ResourceAccepted(amount),
        }];

        let delay = conn.flow_rate.unwrap_or(0.0);
        let out_event = |amt, time| Event {
            time,
            source_id: self.id.clone(),
            source_port: Some("out".to_string()),
            target_id: conn.target_id.clone(),
            target_port: conn.target_port.clone(),
            payload: EventPayload::Resource(amt),
        };

        match self.action {
            DelayAction::Delay => {
                new_events.push(out_event(amount, context.current_time() + delay));
            }
            DelayAction::Queue => {
                let mut remaining = amount;
                let mut release_time = context.current_time();
                while remaining > 0.0 {
                    let release_amount = remaining.min(self.release_amount);
                    release_time += delay;
                    new_events.push(out_event(release_amount, release_time));
                    remaining -= release_amount;
                }
            }
        }

        Ok(new_events)
    }

    fn handle_pull_request(
        &mut self,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        for conn in context.inputs_for_port(Some("in")) {
            new_events.push(Event {
                time: context.current_time(),
                source_id: self.id().to_string(),
                source_port: None,
                target_id: conn.source_id.clone(),
                target_port: None,
                payload: EventPayload::PullRequest,
            });
        }

        Ok(new_events)
    }
}

impl Processor for Delay {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let new_events: Vec<Event> = match &event.payload {
            EventPayload::Step => vec![],
            EventPayload::PullRequest | EventPayload::PullAllRequest => {
                self.handle_pull_request(context)?
            }
            EventPayload::Resource(amount) => self.handle_resource(event, context, *amount)?,
            EventPayload::ResourceAccepted(amount) => {
                self.state.resources_released += amount;
                vec![]
            }
            EventPayload::ResourceRejected(_amount) => vec![],
            event_payload => {
                warn!("Unhandled event payload: {:?}", event_payload);
                vec![]
            }
        };

        assert!(self.state.resources_received >= 0.0);
        assert!(self.state.resources_released >= 0.0);
        assert!(self.state.resources_received - self.state.resources_released >= 0.0);

        Ok(new_events)
    }

    fn get_state(&self) -> ProcessState {
        ProcessState::Delay(self.state.clone())
    }

    fn get_input_ports(&self) -> Vec<String> {
        vec!["in".to_string()]
    }

    fn get_output_ports(&self) -> Vec<String> {
        vec!["out".to_string()]
    }

    fn reset(&mut self) {
        self.state = DelayState::default();
    }
}
