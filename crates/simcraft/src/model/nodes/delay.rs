use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{process_events_with_priority, DelayAction, TriggerMode};
use crate::{
    model::{
        process_state::{DelayState, ProcessState},
        ProcessContext, Processor, SerializableProcess,
    },
    simulator::event::{Event, EventPayload},
    utils::errors::SimulationError,
};

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
    #[builder(setter(skip))]
    next_release_time: f64, // When the next release is allowed
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            id: String::new(),
            state: DelayState::default(),
            trigger_mode: TriggerMode::Automatic,
            action: DelayAction::Delay,
            release_amount: 1.0,
            next_release_time: 0.0,
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

    fn can_release_from_queue(&self, current_time: f64) -> bool {
        self.state.pending_outgoing_resources < self.release_amount
            && self.state.available_resources() >= self.release_amount
            && current_time >= self.next_release_time
    }

    fn create_transfer_event(
        &self,
        target_id: String,
        target_port: Option<String>,
        amount: f64,
        time: f64,
    ) -> Event {
        Event::new(&self.id, &target_id, time, EventPayload::Resource(amount))
            .with_source_port("out")
            .with_target_port(target_port.unwrap_or("in".to_string()))
    }

    fn handle_resource(
        &mut self,
        event: &Event,
        context: &ProcessContext,
        amount: f64,
    ) -> Result<Vec<Event>, SimulationError> {
        assert!(amount >= 0.0);

        let mut outputs = context.outputs_for_port(Some("out"));

        // Validate output connections
        let Some(conn) = outputs.next() else {
            warn!("No output connection - rejecting resources");
            return Ok(vec![Event::new(
                &self.id,
                &event.source_id,
                context.current_time(),
                EventPayload::ResourceRejected(amount),
            )]);
        };

        if outputs.next().is_some() {
            warn!("Multiple output connections - rejecting resources");
            return Ok(vec![Event::new(
                &self.id,
                &event.source_id,
                context.current_time(),
                EventPayload::ResourceRejected(amount),
            )]);
        };

        // Accept the resources
        self.state.resources_received += amount;
        let mut new_events = vec![Event::new(
            &self.id,
            &event.source_id,
            context.current_time(),
            EventPayload::ResourceAccepted(amount),
        )];

        let delay = conn.flow_rate.unwrap_or(1.0);

        match self.action {
            DelayAction::Delay => {
                // In Delay mode, schedule resource transfer after delay
                self.state.pending_outgoing_resources += amount;
                new_events.push(self.create_transfer_event(
                    conn.target_id.clone(),
                    conn.target_port.clone(),
                    amount,
                    context.current_time() + delay,
                ));
            }
            DelayAction::Queue => {
                // If queue was empty, resources need to wait the full delay
                if self.state.available_resources() == amount {
                    self.next_release_time = context.current_time() + delay;
                }

                // Check if we can release immediately
                if self.can_release_from_queue(context.current_time()) {
                    self.state.pending_outgoing_resources += self.release_amount;
                    self.next_release_time = context.current_time() + delay;
                    new_events.push(self.create_transfer_event(
                        conn.target_id.clone(),
                        conn.target_port.clone(),
                        self.release_amount,
                        context.current_time(),
                    ));
                }
            }
        }

        Ok(new_events)
    }

    fn handle_pull_request(
        &mut self,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        Ok(context
            .inputs_for_port(Some("in"))
            .map(|conn| {
                Event::new(
                    self.id(),
                    &conn.source_id,
                    context.current_time(),
                    EventPayload::PullRequest,
                )
            })
            .collect())
    }
}

impl Processor for Delay {
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
            EventPayload::Step => match self.action {
                DelayAction::Delay => vec![], // Delays in Delay mode don't respond to Step events
                DelayAction::Queue => {
                    // In Queue mode, check if we can release on every step
                    let mut outputs = context.outputs_for_port(Some("out"));
                    if let Some(conn) = outputs.next() {
                        let delay = conn.flow_rate.unwrap_or(1.0);
                        if outputs.next().is_none()
                            && self.can_release_from_queue(context.current_time())
                        {
                            self.state.pending_outgoing_resources += self.release_amount;
                            self.next_release_time = context.current_time() + delay;
                            vec![self.create_transfer_event(
                                conn.target_id.clone(),
                                conn.target_port.clone(),
                                self.release_amount,
                                context.current_time(),
                            )]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
            },
            EventPayload::PullRequest | EventPayload::PullAllRequest => {
                self.handle_pull_request(context)?
            }
            EventPayload::Resource(amount) => self.handle_resource(event, context, *amount)?,
            EventPayload::ResourceAccepted(amount) => {
                self.state.pending_outgoing_resources -= amount;
                self.state.resources_released += amount;
                vec![]
            }
            EventPayload::ResourceRejected(amount) => {
                self.state.pending_outgoing_resources -= amount;
                vec![]
            }
            event_payload => {
                warn!("Unhandled event payload: {:?}", event_payload);
                vec![]
            }
        };

        assert!(self.state.resources_received >= 0.0);
        assert!(self.state.resources_released >= 0.0);
        assert!(self.state.pending_outgoing_resources >= 0.0);
        assert!(self.state.current_resources() >= 0.0);
        assert!(self.state.available_resources() >= 0.0);

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
        self.next_release_time = 0.0;
    }
}
