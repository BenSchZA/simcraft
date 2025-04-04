use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{process_events_with_priority, Action, TriggerMode};
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

    fn handle_automatic_action(
        &mut self,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let new_events = match self.action {
            Action::PushAny => self.handle_push_any(context)?,
            Action::PushAll => self.handle_push_all(context)?,
            // TODO Handle invalid actions at compile time
            Action::PullAny => unimplemented!(),
            Action::PullAll => unimplemented!(),
        };

        Ok(new_events)
    }

    fn handle_push_any(&mut self, context: &ProcessContext) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        let outputs = context.outputs_for_port(Some("out"));
        for conn in outputs {
            let amount = conn.flow_rate.unwrap_or(1.0);
            new_events.push(
                Event::new(
                    self.id().to_string(),
                    conn.target_id.clone(),
                    context.current_time(),
                    EventPayload::Resource(amount),
                )
                .with_source_port("out")
                .with_target_port(conn.target_port.clone().unwrap_or("in".to_string())),
            );
        }

        Ok(new_events)
    }

    pub fn handle_push_all(
        &mut self,
        _context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        // NOTE This is complex due to resource delivery accept/reject logic
        unimplemented!()
    }

    fn handle_pull_request(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let amount = context
            .outputs_for_port(Some("out"))
            .find(|conn| conn.target_id == event.source_id)
            .and_then(|conn| conn.flow_rate)
            .unwrap_or(1.0);

        Ok(vec![Event::new(
            self.id().to_string(),
            event.source_id.clone(),
            context.current_time(),
            EventPayload::Resource(amount),
        )
        .with_source_port("out")
        .with_target_port(
            event.source_port.clone().unwrap_or("in".to_string()),
        )])
    }
}

impl Processor for Source {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_events(
        &mut self,
        events: &[Event],
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        process_events_with_priority(events, context, |event, ctx| self.on_event(event, ctx))
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let new_events: Vec<Event> = match &event.payload {
            EventPayload::SimulationStart | EventPayload::SimulationEnd => vec![],
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
            EventPayload::Trigger => self.handle_automatic_action(context)?,
            EventPayload::PullRequest | EventPayload::PullAllRequest => {
                self.handle_pull_request(event, context)?
            }
            EventPayload::ResourceAccepted(amount) => {
                self.state.resources_produced += amount;
                vec![]
            }
            // Sources have infinite resources, so no need to handle rejection
            EventPayload::ResourceRejected(_) => vec![],
            event_payload => {
                warn!("Unhandled event payload: {:?}", event_payload);
                vec![]
            }
        };

        assert!(self.state.resources_produced >= 0.0);

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

    fn reset(&mut self) {
        self.state = SourceState::default();
    }
}
