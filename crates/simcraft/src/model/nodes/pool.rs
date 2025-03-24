use derive_builder::Builder;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::{Action, Overflow, TriggerMode};
use crate::{
    model::{
        process_state::{PoolState, ProcessState},
        Processor, SerializableProcess,
    },
    simulator::{
        event::{Event, EventPayload},
        SimulationContext,
    },
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

    fn handle_automatic_action(
        &mut self,
        context: &SimulationContext,
        new_events: &mut Vec<Event>,
    ) -> Result<(), SimulationError> {
        match self.action {
            Action::PushAny => {
                // Push up to available resources through each connection
                for conn in context.get_outputs(self.id(), Some("out")) {
                    let flow_rate = conn.flow_rate.unwrap_or(1.0);
                    let push_amount = self.state.resources.min(flow_rate);

                    if push_amount > 0.0 {
                        info!(
                            "Pool '{}' pushing {} resources to '{}'",
                            self.id(),
                            push_amount,
                            conn.target_id
                        );
                        new_events.push(Event {
                            time: context.current_time(),
                            source_id: self.id().to_string(),
                            source_port: Some("out".to_string()),
                            target_id: conn.target_id.clone(),
                            target_port: Some(conn.target_port.clone().unwrap_or("in".to_string())),
                            payload: EventPayload::Resource(push_amount),
                        });
                        self.state.resources -= push_amount;
                    }
                }
            }
            Action::PushAll => {
                // Calculate total required resources
                let outputs = context.get_outputs(self.id(), Some("out"));
                let total_required: f64 = outputs
                    .iter()
                    .map(|conn| conn.flow_rate.unwrap_or(1.0))
                    .sum();

                // Push only if we have enough for all outputs
                if self.state.resources >= total_required {
                    for conn in outputs {
                        info!(
                            "Pool '{}' pushing {} resources to '{}'",
                            self.id(),
                            total_required,
                            conn.target_id
                        );
                        let flow_rate = conn.flow_rate.unwrap_or(1.0);
                        new_events.push(Event {
                            time: context.current_time(),
                            source_id: self.id().to_string(),
                            source_port: Some("out".to_string()),
                            target_id: conn.target_id.clone(),
                            target_port: Some(conn.target_port.clone().unwrap_or("in".to_string())),
                            payload: EventPayload::Resource(flow_rate),
                        });
                        self.state.resources -= flow_rate;
                    }
                }
            }
            Action::PullAny => {
                // Pull whatever is available up to flow rates
                for conn in context.get_inputs(self.id(), Some("in")) {
                    let flow_rate = conn.flow_rate.unwrap_or(1.0);
                    // Request resources - actual amount will be determined by source
                    new_events.push(Event {
                        time: context.current_time(),
                        // TODO Decide what the best representation of this is e.g. requesting resources from/to specific ports vs. from/to the node
                        // source_id: conn.source_id.clone(),
                        // source_port: Some(conn.source_port.clone().unwrap_or("out".to_string())),
                        // target_id: self.id().to_string(),
                        // target_port: Some("in".to_string()),
                        source_id: self.id().to_string(),
                        source_port: None,
                        target_id: conn.source_id.clone(),
                        target_port: None,
                        payload: EventPayload::PullRequest(flow_rate),
                    });
                }
            }
            Action::PullAll => {
                // Calculate total requested resources
                let inputs = context.get_inputs(self.id(), Some("in"));
                let total_requested: f64 = inputs
                    .iter()
                    .map(|conn| conn.flow_rate.unwrap_or(1.0))
                    .sum();

                // TODO This should only pull if all sources can provide resources equal to the flow rate, otherwise pull none
                // Request all - will only receive if all are available
                for conn in inputs {
                    let flow_rate = conn.flow_rate.unwrap_or(1.0);
                    new_events.push(Event {
                        time: context.current_time(),
                        source_id: conn.source_id.clone(),
                        // source_port: Some(conn.source_port.clone().unwrap_or("out".to_string())),
                        source_port: None,
                        target_id: self.id().to_string(),
                        // target_port: Some("in".to_string()),
                        target_port: None,
                        payload: EventPayload::PullAllRequest {
                            amount: flow_rate,
                            total_required: total_requested,
                        },
                    });
                }
            }
        }
        Ok(())
    }

    fn handle_pull_request(
        &mut self,
        event: &Event,
        context: &SimulationContext,
        amount: f64,
    ) -> Event {
        debug!("Pool '{}' handling pull request for {}", self.id(), amount);
        let push_amount = self.state.resources.min(amount);
        self.state.resources -= push_amount;

        Event {
            time: context.current_time(),
            source_id: self.id().to_string(),
            source_port: Some("out".to_string()),
            target_id: event.source_id.clone(),
            target_port: Some("in".to_string()),
            payload: EventPayload::Resource(push_amount),
        }
    }
}

impl Processor for Pool {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_event(
        &mut self,
        event: &Event,
        context: &SimulationContext,
    ) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();

        match &event.payload {
            EventPayload::Step => match self.trigger_mode {
                TriggerMode::Passive => {}
                TriggerMode::Interactive => {}
                TriggerMode::Automatic => {
                    self.handle_automatic_action(context, &mut new_events)?;
                }
                TriggerMode::Enabling => {}
            },
            EventPayload::Trigger => {
                // TODO Temporarily act as if automatic
            }
            EventPayload::Resource(amount) => {
                info!(
                    "{}: Pool '{}' receiving {} resources from '{}'",
                    context.current_time(),
                    self.id(),
                    amount,
                    event.source_id
                );

                // Handle incoming resource transfer
                let new_amount = if self.capacity < 0.0 {
                    self.state.resources + amount
                } else {
                    match self.overflow {
                        Overflow::Block => {
                            if self.state.resources + amount <= self.capacity {
                                self.state.resources + amount
                            } else {
                                // TODO Block should ensure no resources are pulled from source
                                self.state.resources
                            }
                        }
                        Overflow::Drain => (self.state.resources + amount).min(self.capacity),
                    }
                };
                self.state.resources = new_amount;
            }
            EventPayload::PullRequest(amount) => {
                new_events.push(self.handle_pull_request(event, context, *amount));
            }
            // TODO Implement Pull All
            // EventPayload::PullAllRequest { amount, .. } => {
            //     new_events.push(self.handle_pull_request(event, context, *amount));
            // }
            _ => {}
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::connection::Connection;

    #[test]
    fn test_automatic_push_any() -> Result<(), SimulationError> {
        let trigger_mode = TriggerMode::Automatic;
        let action = Action::PushAny;

        let flow_rate = 5.0;
        let resources = 2.0;

        let mut context = SimulationContext::default();

        context.add_connection(Connection {
            source_port: Some("out".to_string()),
            flow_rate: Some(flow_rate),
            ..Default::default()
        });

        let mut event = Event {
            time: context.current_time(),
            source_id: "".to_string(),
            source_port: Some("step".to_string()),
            target_id: "broadcast".to_string(),
            target_port: None,
            payload: EventPayload::Step,
        };

        let mut pool = Pool::builder()
            .state(PoolState { resources })
            .trigger_mode(trigger_mode)
            .action(action)
            .build()
            .unwrap();

        pool.on_event(&mut event, &mut context)?;

        assert_eq!(
            pool.state.resources,
            resources - resources.min(flow_rate),
            "Should push up to available resources through each connection"
        );

        Ok(())
    }

    #[test]
    fn test_automatic_push_all() -> Result<(), SimulationError> {
        let trigger_mode = TriggerMode::Automatic;
        let action = Action::PushAll;

        let pool_id = "1";

        let mut context = SimulationContext::default();

        context.add_connection(Connection {
            source_id: pool_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: "2".to_string(),
            ..Default::default()
        });

        context.add_connection(Connection {
            source_id: pool_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: "3".to_string(),
            ..Default::default()
        });

        let mut event = Event {
            time: context.current_time(),
            source_id: "".to_string(),
            source_port: Some("step".to_string()),
            target_id: "broadcast".to_string(),
            target_port: None,
            payload: EventPayload::Step,
        };

        let mut pool = Pool::builder()
            .id(pool_id)
            .state(PoolState { resources: 1.0 })
            .trigger_mode(trigger_mode)
            .action(action)
            .build()
            .unwrap();

        pool.on_event(&mut event, &mut context)?;

        assert_eq!(
            pool.state.resources, 1.0,
            "Should only push if enough resources for all outputs"
        );

        pool.state.resources = 2.0;
        pool.on_event(&mut event, &mut context)?;

        assert_eq!(pool.state.resources, 0.0, "Should push all resources");

        Ok(())
    }

    #[test]
    fn test_automatic_pull_any() -> Result<(), SimulationError> {
        let trigger_mode = TriggerMode::Automatic;
        let action = Action::PullAny;

        let flow_rate = 2.0;
        let from_resources = 1.0;
        let to_resources = 0.0;

        let from_pool_id = "1";
        let to_pool_id = "2";

        let mut context = SimulationContext::default();

        context.add_connection(Connection {
            source_id: from_pool_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: to_pool_id.to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(flow_rate),
            ..Default::default()
        });

        let mut event = Event {
            time: context.current_time(),
            source_id: from_pool_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: to_pool_id.to_string(),
            target_port: Some("in".to_string()),
            payload: EventPayload::PullRequest(flow_rate),
        };

        let mut from_pool = Pool::builder()
            .id(from_pool_id.to_string())
            .state(PoolState {
                resources: from_resources,
            })
            .trigger_mode(trigger_mode)
            .action(action)
            .build()
            .unwrap();

        let mut to_pool = Pool::builder()
            .id(to_pool_id.to_string())
            .state(PoolState {
                resources: to_resources,
            })
            .trigger_mode(trigger_mode)
            .action(action)
            .build()
            .unwrap();

        // from_pool receives Pull Request event from to_pool, decreasing resources
        let mut new_events = from_pool.on_event(&mut event, &mut context).unwrap();
        if let Some(resource_event) = new_events.pop() {
            assert_eq!(
                resource_event.payload,
                EventPayload::Resource(from_resources)
            );
            // from_pool sends Resource event to to_pool, increasing resources
            to_pool.on_event(&resource_event, &mut context)?;
        };

        assert_eq!(
            to_pool.state.resources, from_resources,
            "Should receive any available resources up to flow rate"
        );
        assert_eq!(
            from_pool.state.resources,
            from_resources - from_resources.min(flow_rate),
            "Should send any available resources up to flow rate"
        );

        Ok(())
    }
}
