use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tracing::warn;

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

    fn handle_automatic_action(
        &mut self,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let new_events = match self.action {
            Action::PullAny => self.handle_pull_any(context)?,
            Action::PullAll => self.handle_pull_all(context)?,
            // TODO Handle invalid actions at compile time
            Action::PushAny => unimplemented!(),
            Action::PushAll => unimplemented!(),
        };

        Ok(new_events)
    }

    fn handle_pull_any(&mut self, context: &ProcessContext) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        // Pull whatever is available up to flow rates from each input
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

    fn handle_pull_all(&mut self, context: &ProcessContext) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        // Request all resources - will only receive if all are available
        let inputs: Vec<&Connection> = context.outputs_for_port(Some("in")).collect();
        for conn in inputs {
            new_events.push(Event {
                time: context.current_time(),
                source_id: self.id().to_string(),
                source_port: None,
                target_id: conn.source_id.clone(),
                target_port: None,
                payload: EventPayload::PullAllRequest,
            });
        }

        Ok(new_events)
    }

    fn handle_resource(
        &mut self,
        event: &Event,
        context: &ProcessContext,
        amount: f64,
    ) -> Result<Vec<Event>, SimulationError> {
        assert!(amount >= 0.0);

        self.state.resources_consumed += amount;

        Ok(vec![Event {
            time: context.current_time(),
            source_id: self.id().to_string(),
            source_port: None,
            target_id: event.source_id.clone(),
            target_port: None,
            payload: EventPayload::ResourceAccepted(amount),
        }])
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
        let new_events: Vec<Event> = match &event.payload {
            EventPayload::Step => match self.trigger_mode {
                TriggerMode::Passive => vec![],
                TriggerMode::Interactive => unimplemented!(),
                TriggerMode::Automatic => self.handle_automatic_action(context)?,
                TriggerMode::Enabling => {
                    if context.current_step() == 1 {
                        self.handle_automatic_action(context)?
                    } else {
                        vec![]
                    }
                }
            },
            EventPayload::Resource(amount) => self.handle_resource(event, context, *amount)?,
            event_payload => {
                warn!("Unhandled event payload: {:?}", event_payload);
                vec![]
            }
        };

        assert!(self.state.resources_consumed >= 0.0);

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

    fn reset(&mut self) {
        self.state = DrainState::default();
    }
}
