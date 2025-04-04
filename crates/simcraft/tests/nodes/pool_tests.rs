#[cfg(test)]
mod pool_tests {
    use super::*;
    use crate::model::Connection;
    use crate::prelude::*;

    #[test]
    fn test_automatic_push_any() -> Result<(), SimulationError> {
        let trigger_mode = TriggerMode::Automatic;
        let action = Action::PushAny;

        let flow_rate = 5.0;
        let resources = 2.0;

        let sender_id = "sender";
        let receiver_id = "receiver";

        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id(sender_id)
            .state(PoolState {
                resources,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(trigger_mode)
            .action(action)
            .build()
            .unwrap();

        let mut receiver = Pool::builder()
            .id(receiver_id)
            .state(PoolState::default())
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;
        simulation.add_process(receiver.clone())?;

        simulation.add_connection(Connection {
            source_id: sender_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver_id.to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(flow_rate),
            ..Default::default()
        })?;

        // Step the sender to emit Resource event
        let context = simulation.get_context().context_for_process(sender_id);
        let step_event = Event::new(
            "".to_string(),
            sender_id.to_string(),
            context.current_time(),
            EventPayload::Step,
        );

        let resource_events = sender.on_event(&step_event, &context)?;

        assert_eq!(
            resource_events.len(),
            1,
            "Sender should emit exactly one Resource event"
        );

        assert_eq!(
            sender.state.resources, resources,
            "Sender should not decrement resources yet"
        );

        assert_eq!(
            sender.state.pending_outgoing_resources,
            resources.min(flow_rate),
            "Sender should track pending outgoing resources"
        );

        // Deliver the resource event to the receiver
        let context = simulation.get_context().context_for_process(receiver_id);
        let ack_events = receiver.on_event(&resource_events[0], &context)?;

        assert!(
            ack_events
                .iter()
                .any(|e| matches!(e.payload, EventPayload::ResourceAccepted(_))),
            "Receiver should respond with ResourceAccepted"
        );

        // Feed the acknowledgment back to the sender
        let context = simulation.get_context().context_for_process(sender_id);
        for ack in ack_events {
            sender.on_event(&ack, &context)?;
        }

        assert_eq!(
            sender.state.pending_outgoing_resources, 0.0,
            "Sender should clear pending once accepted"
        );

        assert_eq!(
            sender.state.resources,
            resources - resources.min(flow_rate),
            "Sender should finally reduce resources on acceptance"
        );

        Ok(())
    }

    #[test]
    fn test_automatic_push_all() -> Result<(), SimulationError> {
        let trigger_mode = TriggerMode::Automatic;
        let action = Action::PushAll;

        let sender_id = "1";
        let receiver_a_id = "2";
        let receiver_b_id = "3";

        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id(sender_id)
            .state(PoolState {
                resources: 1.0,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(trigger_mode)
            .action(action)
            .build()
            .unwrap();

        let mut receiver_a = Pool::builder()
            .id(receiver_a_id)
            .state(PoolState::default())
            .build()
            .unwrap();

        let mut receiver_b = Pool::builder()
            .id(receiver_b_id)
            .state(PoolState::default())
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;
        simulation.add_process(receiver_a.clone())?;
        simulation.add_process(receiver_b.clone())?;

        // Connections
        simulation.add_connection(Connection {
            source_id: sender_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver_a_id.to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
            ..Default::default()
        })?;

        simulation.add_connection(Connection {
            source_id: sender_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver_b_id.to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
            ..Default::default()
        })?;

        // 1. Step with insufficient resources
        let context = simulation.get_context().context_for_process(sender_id);
        let step_event = Event::new(
            "".to_string(),
            sender_id.to_string(),
            context.current_time(),
            EventPayload::Step,
        );

        let events = sender.on_event(&step_event, &context)?;
        assert!(
            events.is_empty(),
            "Should not emit events if not enough resources to push to all outputs"
        );
        assert_eq!(
            sender.state.resources, 1.0,
            "Resources should remain unchanged"
        );

        // 2. Now with enough resources
        sender.state.resources = 2.0;

        let context = simulation.get_context().context_for_process(sender_id);
        let events = sender.on_event(&step_event, &context)?;

        assert_eq!(
            events.len(),
            2,
            "Should push to both receivers when enough resources"
        );

        assert_eq!(
            sender.state.resources, 2.0,
            "Resources should not be deducted immediately"
        );

        assert_eq!(
            sender.state.pending_outgoing_resources, 2.0,
            "Pending outgoing should track both transfers"
        );

        // 3. Feed each resource to receiver and collect ack
        let context_a = simulation.get_context().context_for_process(receiver_a_id);
        let ack_a = receiver_a.on_event(&events[0], &context_a)?;

        let context_b = simulation.get_context().context_for_process(receiver_b_id);
        let ack_b = receiver_b.on_event(&events[1], &context_b)?;

        // 4. Feed acks back to sender
        let context_sender = simulation.get_context().context_for_process(sender_id);
        for ack in ack_a.into_iter().chain(ack_b.into_iter()) {
            sender.on_event(&ack, &context_sender)?;
        }

        assert_eq!(
            sender.state.resources, 0.0,
            "Resources should be deducted after acknowledgments"
        );
        assert_eq!(
            sender.state.pending_outgoing_resources, 0.0,
            "Pending should be cleared after acknowledgments"
        );

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

        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut from_pool = Pool::builder()
            .id(from_pool_id)
            .state(PoolState {
                resources: from_resources,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(TriggerMode::Passive)
            .action(Action::PushAny) // Responds to PullRequest
            .build()
            .unwrap();

        let mut to_pool = Pool::builder()
            .id(to_pool_id)
            .state(PoolState {
                resources: to_resources,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(trigger_mode)
            .action(action)
            .capacity(10.0)
            .overflow(Overflow::Drain)
            .build()
            .unwrap();

        simulation.add_process(from_pool.clone())?;
        simulation.add_process(to_pool.clone())?;

        simulation.add_connection(Connection {
            source_id: from_pool_id.to_string(),
            source_port: Some("out".to_string()),
            target_id: to_pool_id.to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(flow_rate),
            ..Default::default()
        })?;

        // Step to_pool so it emits PullRequest
        let context = simulation.get_context().context_for_process(to_pool_id);
        let step_event = Event::new(
            "".to_string(),
            to_pool_id.to_string(),
            context.current_time(),
            EventPayload::Step,
        );

        let pull_request = to_pool.on_event(&step_event, &context)?;
        assert_eq!(pull_request.len(), 1);
        assert!(matches!(pull_request[0].payload, EventPayload::PullRequest));

        // from_pool receives PullRequest and emits Resource
        let context = simulation.get_context().context_for_process(from_pool_id);
        let resource_event = from_pool.on_event(&pull_request[0], &context)?;
        assert_eq!(resource_event.len(), 1);
        assert_eq!(
            resource_event[0].payload,
            EventPayload::Resource(from_resources),
            "Should respond with available resource amount"
        );

        // to_pool receives Resource and emits ResourceAccepted
        let context = simulation.get_context().context_for_process(to_pool_id);
        let ack_event = to_pool.on_event(&resource_event[0], &context)?;
        assert!(ack_event
            .iter()
            .any(|e| matches!(e.payload, EventPayload::ResourceAccepted(_))));
        assert_eq!(
            to_pool.state.resources, from_resources,
            "to_pool should accept the resource"
        );

        // from_pool receives ResourceAccepted and updates state
        let context = simulation.get_context().context_for_process(from_pool_id);
        for ack in ack_event {
            from_pool.on_event(&ack, &context)?;
        }

        assert_eq!(
            from_pool.state.resources, 0.0,
            "from_pool should deduct resources after acknowledgment"
        );
        assert_eq!(
            from_pool.state.pending_outgoing_resources, 0.0,
            "from_pool should clear pending after acknowledgment"
        );

        Ok(())
    }

    #[test]
    fn test_pool_drain_partial_acceptance() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![])?;

        let sender = Pool::builder()
            .id("sender")
            .state(PoolState {
                resources: 10.0,
                pending_outgoing_resources: 0.0,
            })
            .build()
            .unwrap();

        let mut receiver = Pool::builder()
            .id("receiver")
            .state(PoolState {
                resources: 9.0,
                pending_outgoing_resources: 0.0,
            })
            .capacity(10.0)
            .overflow(Overflow::Drain)
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;
        simulation.add_process(receiver.clone())?;

        simulation.add_connection(Connection {
            source_id: sender.id().to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver.id().to_string(),
            target_port: Some("in".to_string()),
            ..Default::default()
        })?;

        // Simulate resource send
        let context = simulation.get_context().context_for_process(receiver.id());
        let resource_event = Event::new(
            sender.id().to_string(),
            receiver.id().to_string(),
            context.current_time(),
            EventPayload::Resource(5.0),
        )
        .with_ports("out", "in");

        let events = receiver.on_event(&resource_event, &context)?;

        assert_eq!(
            receiver.state.resources, 10.0,
            "Should accept only up to capacity"
        );

        assert!(events
            .iter()
            .any(|e| matches!(e.payload, EventPayload::ResourceAccepted(1.0))));
        assert!(events
            .iter()
            .any(|e| matches!(e.payload, EventPayload::ResourceRejected(4.0))));

        Ok(())
    }

    #[test]
    fn test_resource_acceptance_updates_sender() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id("sender")
            .state(PoolState {
                resources: 10.0,
                pending_outgoing_resources: 5.0,
            })
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;

        let context = simulation.get_context().context_for_process(sender.id());
        let ack_event = Event::new(
            "receiver".to_string(),
            sender.id().to_string(),
            context.current_time(),
            EventPayload::ResourceAccepted(5.0),
        )
        .with_ports("in", "out");

        sender.on_event(&ack_event, &context)?;

        assert_eq!(
            sender.state.pending_outgoing_resources, 0.0,
            "Pending outgoing should be cleared"
        );
        assert_eq!(
            sender.state.resources, 5.0,
            "Resources should be reduced after acceptance"
        );

        Ok(())
    }

    #[test]
    fn test_resource_rejection_clears_pending_only() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id("sender")
            .state(PoolState {
                resources: 5.0,
                pending_outgoing_resources: 3.0,
            })
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;

        let context = simulation.get_context().context_for_process(sender.id());

        let rejection_event = Event::new(
            "receiver".to_string(),
            sender.id().to_string(),
            context.current_time(),
            EventPayload::ResourceRejected(3.0),
        )
        .with_ports("in", "out");

        sender.on_event(&rejection_event, &context)?;

        assert_eq!(
            sender.state.pending_outgoing_resources, 0.0,
            "Pending outgoing should be cleared after rejection"
        );

        assert_eq!(
            sender.state.resources, 5.0,
            "Resources should remain unchanged because they were never deducted"
        );

        Ok(())
    }

    #[test]
    fn test_push_any_blocked_by_capacity() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id("sender")
            .state(PoolState {
                resources: 5.0,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PushAny)
            .build()
            .unwrap();

        let mut receiver = Pool::builder()
            .id("receiver")
            .state(PoolState {
                resources: 10.0,
                pending_outgoing_resources: 0.0,
            })
            .capacity(10.0)
            .overflow(Overflow::Block)
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;
        simulation.add_process(receiver.clone())?;

        simulation.add_connection(Connection {
            source_id: sender.id().to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver.id().to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(5.0),
            ..Default::default()
        })?;

        // Trigger sender to attempt push
        let context = simulation.get_context().context_for_process(sender.id());
        let event = Event::new(
            "".to_string(),
            sender.id().to_string(),
            context.current_time(),
            EventPayload::Step,
        );

        let push_events = sender.on_event(&event, &context)?;
        assert!(
            push_events
                .iter()
                .any(|e| matches!(e.payload, EventPayload::Resource(5.0))),
            "Sender should attempt to push resources"
        );

        // Receiver should reject
        let context = simulation.get_context().context_for_process(receiver.id());
        let result = receiver.on_event(&push_events[0], &context)?;
        assert!(
            result
                .iter()
                .any(|e| matches!(e.payload, EventPayload::ResourceRejected(_))),
            "Receiver should reject due to capacity"
        );

        Ok(())
    }

    #[test]
    fn test_push_any_drain_partial_accept() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id("sender")
            .state(PoolState {
                resources: 5.0,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PushAny)
            .build()
            .unwrap();

        let mut receiver = Pool::builder()
            .id("receiver")
            .state(PoolState {
                resources: 8.0,
                pending_outgoing_resources: 0.0,
            })
            .capacity(10.0)
            .overflow(Overflow::Drain)
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;
        simulation.add_process(receiver.clone())?;

        simulation.add_connection(Connection {
            source_id: sender.id().to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver.id().to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(5.0),
            ..Default::default()
        })?;

        let context = simulation.get_context().context_for_process(sender.id());
        let step_event = Event::new(
            "".to_string(),
            sender.id().to_string(),
            context.current_time(),
            EventPayload::Step,
        )
        .with_ports("out", "in");

        let push_events = sender.on_event(&step_event, &context)?;
        let context = simulation.get_context().context_for_process(receiver.id());
        let result = receiver.on_event(&push_events[0], &context)?;

        assert!(result
            .iter()
            .any(|e| matches!(e.payload, EventPayload::ResourceAccepted(2.0))));
        assert!(result
            .iter()
            .any(|e| matches!(e.payload, EventPayload::ResourceRejected(3.0))));
        Ok(())
    }

    #[test]
    fn test_pull_any_full_acceptance() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![])?;

        let mut sender = Pool::builder()
            .id("sender")
            .state(PoolState {
                resources: 5.0,
                pending_outgoing_resources: 0.0,
            })
            .build()
            .unwrap();

        let mut receiver = Pool::builder()
            .id("receiver")
            .state(PoolState {
                resources: 0.0,
                pending_outgoing_resources: 0.0,
            })
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PullAny)
            .capacity(10.0)
            .overflow(Overflow::Drain)
            .build()
            .unwrap();

        simulation.add_process(sender.clone())?;
        simulation.add_process(receiver.clone())?;

        simulation.add_connection(Connection {
            source_id: sender.id().to_string(),
            source_port: Some("out".to_string()),
            target_id: receiver.id().to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(3.0),
            ..Default::default()
        })?;

        // Pull request from receiver
        let context = simulation.get_context().context_for_process(receiver.id());
        let pull_event = Event::new(
            "".to_string(),
            receiver.id().to_string(),
            context.current_time(),
            EventPayload::Step,
        )
        .with_ports("out", "in");

        let pull_requests = receiver.on_event(&pull_event, &context)?;
        assert_eq!(pull_requests.len(), 1);
        assert!(matches!(
            pull_requests[0].payload,
            EventPayload::PullRequest
        ));

        // Sender handles pull request
        let context = simulation.get_context().context_for_process(sender.id());
        let push_events = sender.on_event(&pull_requests[0], &context)?;
        assert!(matches!(
            push_events[0].payload,
            EventPayload::Resource(3.0)
        ));

        // Receiver handles resource event
        let context = simulation.get_context().context_for_process(receiver.id());
        let result = receiver.on_event(&push_events[0], &context)?;

        assert!(result
            .iter()
            .any(|e| matches!(e.payload, EventPayload::ResourceAccepted(3.0))));
        Ok(())
    }
}
