use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{process_events_with_priority, Action, Overflow, TriggerMode};
use crate::{
    model::{
        process_state::{PoolState, ProcessState},
        Connection, ProcessContext, Processor, SerializableProcess,
    },
    simulator::event::{Event, EventPayload},
    utils::errors::SimulationError,
};

#[derive(Builder, Debug, Clone, Serialize, Deserialize, SerializableProcess)]
#[serde(default, rename_all = "camelCase")]
#[builder(default)]
pub struct Pool {
    #[builder(setter(into))]
    id: String,
    state: PoolState,
    trigger_mode: TriggerMode,
    action: Action,
    overflow: Overflow,
    capacity: f64,
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            id: String::new(),
            state: PoolState::default(),
            trigger_mode: TriggerMode::Passive,
            action: Action::PullAny,
            overflow: Overflow::Block,
            capacity: -1.0,
        }
    }
}

impl Pool {
    pub fn new(id: &str) -> Pool {
        Pool::builder().id(id.to_string()).build().unwrap()
    }

    pub fn builder() -> PoolBuilder {
        PoolBuilder::default()
    }

    fn available_resources(&self) -> f64 {
        self.state.available_resources()
    }

    fn handle_automatic_action(
        &mut self,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        match self.action {
            Action::PushAny => {
                // Push up to available resources through each connection
                for conn in context.outputs_for_port(Some("out")) {
                    let flow_rate = conn.flow_rate.unwrap_or(1.0);
                    let available_resources = self.available_resources();
                    let push_amount = available_resources.min(flow_rate);

                    if push_amount > 0.0 {
                        new_events.push(
                            Event::new(
                                self.id(),
                                &conn.target_id,
                                context.current_time(),
                                EventPayload::Resource(push_amount),
                            )
                            .with_source_port("out")
                            .with_target_port(conn.target_port.clone().unwrap_or("in".to_string())),
                        );

                        self.state.pending_outgoing_resources += push_amount;
                    }
                }
            }
            Action::PushAll => {
                // Calculate total required resources
                let outputs: Vec<&Connection> = context.outputs_for_port(Some("out")).collect();
                let total_required: f64 = outputs
                    .iter()
                    .map(|conn| conn.flow_rate.unwrap_or(1.0))
                    .sum();

                // Push only if we have enough available resources for all outputs
                let available_resources = self.available_resources();
                if available_resources >= total_required {
                    for conn in outputs {
                        let flow_rate = conn.flow_rate.unwrap_or(1.0);
                        new_events.push(
                            Event::new(
                                self.id(),
                                &conn.target_id,
                                context.current_time(),
                                EventPayload::Resource(flow_rate),
                            )
                            .with_source_port("out")
                            .with_target_port(conn.target_port.clone().unwrap_or("in".to_string())),
                        );

                        self.state.pending_outgoing_resources += flow_rate;
                    }
                }
            }
            Action::PullAny => {
                // Pull whatever is available up to flow rates
                for conn in context.inputs_for_port(Some("in")) {
                    // Request resources - actual amount will be determined by Source/Pool
                    new_events.push(Event::new(
                        self.id(),
                        &conn.source_id,
                        context.current_time(),
                        EventPayload::PullRequest,
                    ));
                }
            }
            Action::PullAll => {
                // Request all - will only receive if flow rate resources are available
                let inputs: Vec<&Connection> = context.outputs_for_port(Some("in")).collect();
                for conn in inputs {
                    new_events.push(Event::new(
                        &conn.source_id,
                        self.id(),
                        context.current_time(),
                        EventPayload::PullAllRequest,
                    ));
                }
            }
        }

        Ok(new_events)
    }

    fn handle_pull_request(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let flow_rate = context
            .outputs_for_port(Some("out"))
            .find(|conn| conn.target_id == event.source_id)
            .map(|conn| match conn.flow_rate {
                Some(rate) => rate,
                None => {
                    warn!(
                        "Pool '{}' has no flow_rate set for connection to '{}'. Defaulting to flow rate of 1.0.",
                        self.id(),
                        event.source_id
                    );
                    1.0
                }
            })
            .unwrap_or_else(|| {
                warn!(
                    "Pool '{}' has no output connection to '{}'. Defaulting to flow rate of 0.",
                    self.id(),
                    event.source_id
                );
                0.0
            });

        let available_resources = self.available_resources();
        let amount = available_resources.min(flow_rate);

        if amount > 0.0 {
            self.state.pending_outgoing_resources += amount;

            let event = Event::new(
                self.id(),
                &event.source_id,
                context.current_time(),
                EventPayload::Resource(amount),
            )
            .with_source_port("out")
            .with_target_port("in");
            Ok(vec![event])
        } else {
            Ok(vec![])
        }
    }

    fn handle_pull_all_request(
        &mut self,
        event: &Event,
        context: &ProcessContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let amount = context
            .outputs_for_port(Some("out"))
            .find(|conn| conn.target_id == event.source_id)
            .map(|conn| {
                let required = conn.flow_rate.unwrap_or(0.0);

                if self.state.resources < required {
                    warn!(
                        "Pool '{}' has insufficient resources ({}) for full transfer ({} required) to '{}'.",
                        self.id(),
                        self.state.resources,
                        required,
                        event.source_id
                    );
                    0.0
                } else {
                    required
                }
            })
            .unwrap_or_else(|| {
                // TODO Emit warning event and decide how best to handle flow_rate default
                warn!(
                    "Pool '{}' has no output connection to '{}'. Declining pull-all request.",
                    self.id(),
                    event.source_id
                );
                0.0
            });

        if amount > 0.0 {
            self.state.pending_outgoing_resources += amount;

            let mut event = Event::new(
                self.id(),
                &event.source_id,
                context.current_time(),
                EventPayload::Resource(amount),
            );
            event = event.with_source_port("out");
            event = event.with_target_port("in");
            Ok(vec![event])
        } else {
            Ok(vec![])
        }
    }

    fn handle_resource(
        &mut self,
        event: &Event,
        context: &ProcessContext,
        amount: f64,
    ) -> Result<Vec<Event>, SimulationError> {
        assert!(amount >= 0.0);

        let future_resources = self.state.resources + amount;
        let (accepted, rejected) = if self.capacity < 0.0 || future_resources <= self.capacity {
            self.state.resources += amount;
            (amount, 0.0)
        } else {
            match self.overflow {
                Overflow::Block => (0.0, amount),
                Overflow::Drain => {
                    let accepted = (self.capacity - self.state.resources).max(0.0);
                    self.state.resources += accepted;
                    let rejected = amount - accepted;
                    (accepted, rejected)
                }
            }
        };

        let mut new_events = Vec::new();

        if accepted > 0.0 {
            new_events.push(Event::new(
                self.id(),
                &event.source_id,
                context.current_time(),
                EventPayload::ResourceAccepted(accepted),
            ));
        }

        if rejected > 0.0 {
            new_events.push(Event::new(
                self.id(),
                &event.source_id,
                context.current_time(),
                EventPayload::ResourceRejected(rejected),
            ));
        }

        Ok(new_events)
    }
}

impl Processor for Pool {
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
            EventPayload::PullRequest => self.handle_pull_request(event, context)?,
            EventPayload::PullAllRequest => self.handle_pull_all_request(event, context)?,
            EventPayload::Resource(amount) => self.handle_resource(event, context, *amount)?,
            EventPayload::ResourceAccepted(amount) => {
                self.state.pending_outgoing_resources -= amount;
                self.state.resources -= amount;
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

        assert!(
            self.state.pending_outgoing_resources >= 0.0,
            "Resource Underflow: Pending outgoing resources for pool {} = {}",
            self.id,
            self.state.pending_outgoing_resources
        );
        assert!(
            self.state.resources >= 0.0,
            "Resource Underflow: Resources for pool {} = {}",
            self.id,
            self.state.resources
        );
        if self.capacity >= 0.0 {
            assert!(
                self.state.resources <= self.capacity + f64::EPSILON,
                "Resource Overflow: resources = {}, capacity = {}",
                self.state.resources,
                self.capacity
            );
        }

        Ok(new_events)
    }

    fn get_state(&self) -> ProcessState {
        ProcessState::Pool(self.state.clone())
    }

    fn get_input_ports(&self) -> Vec<String> {
        vec!["in".to_string()]
    }

    fn get_output_ports(&self) -> Vec<String> {
        vec!["out".to_string()]
    }

    fn reset(&mut self) {
        self.state = PoolState::default();
    }
}
