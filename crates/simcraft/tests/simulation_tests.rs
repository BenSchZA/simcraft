mod common;

#[cfg(test)]
mod tests {
    use log::info;

    use simcraft::model::nodes::Action;
    use simcraft::model::nodes::Delay;
    use simcraft::model::nodes::DelayAction;
    use simcraft::model::nodes::Drain;
    use simcraft::model::nodes::Overflow;
    use simcraft::model::nodes::Pool;
    use simcraft::model::nodes::Source;
    use simcraft::model::nodes::Stepper;
    use simcraft::model::nodes::TriggerMode;
    use simcraft::model::process_state::PoolState;
    use simcraft::model::ProcessState;
    use simcraft::simulator::simulation_trait::StatefulSimulation;

    use simcraft::prelude::*;

    use crate::common::{create_stepped_simulation, setup};

    #[test]
    fn test_city_traffic_simulation() -> Result<(), SimulationError> {
        let mut simulation = create_stepped_simulation(vec![], vec![])?;

        // Source: cars enter the city
        let source = Source::builder()
            .id("cars_in")
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PushAny)
            .build()
            .unwrap();

        // Delay: simulate a traffic light delay
        let delay = Delay::builder()
            .id("traffic_light")
            .action(DelayAction::Delay)
            .release_amount(1.0)
            .trigger_mode(TriggerMode::Automatic)
            .build()
            .unwrap();

        // Queue: simulate one-lane bridge
        let queue = Delay::builder()
            .id("one_lane_bridge")
            .action(DelayAction::Queue)
            .release_amount(1.0)
            .trigger_mode(TriggerMode::Automatic)
            .build()
            .unwrap();

        // Pool: central hub (e.g. roundabout)
        let pool = Pool::builder()
            .id("roundabout")
            .capacity(10.0)
            .overflow(Overflow::Drain)
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PushAny)
            .build()
            .unwrap();

        // Drain: cars leave the city
        let drain = Drain::builder()
            .id("exit")
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PullAny)
            .build()
            .unwrap();

        simulation.add_process(source)?;
        simulation.add_process(delay)?;
        simulation.add_process(queue)?;
        simulation.add_process(pool)?;
        simulation.add_process(drain)?;

        // Connections with delays (output flow_rate interpreted as delay in seconds)
        simulation.add_connection(Connection {
            source_id: "cars_in".to_string(),
            source_port: Some("out".to_string()),
            target_id: "traffic_light".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(5.0),
            ..Default::default()
        })?;

        simulation.add_connection(Connection {
            source_id: "traffic_light".to_string(),
            source_port: Some("out".to_string()),
            target_id: "one_lane_bridge".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(2.0), // 2s delay per car through bridge
            ..Default::default()
        })?;

        simulation.add_connection(Connection {
            source_id: "one_lane_bridge".to_string(),
            source_port: Some("out".to_string()),
            target_id: "roundabout".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(2.0),
            ..Default::default()
        })?;

        simulation.add_connection(Connection {
            source_id: "roundabout".to_string(),
            source_port: Some("out".to_string()),
            target_id: "exit".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
            ..Default::default()
        })?;

        // Step the simulation
        simulation.step_n(10)?;

        let state = simulation.get_simulation_state();

        if let ProcessState::Source(s) = &state.process_states["cars_in"] {
            assert_eq!(s.resources_produced, 50.0, "Expected 10 cars produced");
        }

        if let ProcessState::Delay(d) = &state.process_states["traffic_light"] {
            assert_eq!(
                d.resources_received, 50.0,
                "All cars should be received by traffic light"
            );
        }

        if let ProcessState::Delay(d) = &state.process_states["traffic_light"] {
            assert_eq!(
                d.resources_released, 40.0,
                "Some of the cars should be released by traffic light"
            );
        }

        if let ProcessState::Delay(q) = &state.process_states["one_lane_bridge"] {
            assert_eq!(
                q.resources_received, 40.0,
                "Cars should be queued after delay"
            );
        }

        if let ProcessState::Pool(p) = &state.process_states["roundabout"] {
            assert_eq!(p.resources, 0.0, "Roundabout should contain no cars");
        }

        if let ProcessState::Drain(d) = &state.process_states["exit"] {
            assert_eq!(d.resources_consumed, 3.0, "Three cars should have exited");
        }

        Ok(())
    }

    #[test]
    fn test_source_delay_pool() -> Result<(), SimulationError> {
        let mut simulation = create_stepped_simulation(vec![], vec![])?;

        // Source: automatic, pushes 5 resources per step
        let source = Source::builder()
            .id("source".to_string())
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PushAny)
            .build()
            .unwrap();

        // Delay: automatic, releases after 2s delay
        let delay = Delay::builder()
            .id("delay".to_string())
            .action(DelayAction::Delay)
            .release_amount(1.0)
            .trigger_mode(TriggerMode::Automatic)
            .build()
            .unwrap();

        // Pool: automatic, pulls any available resources
        let pool = Pool::builder()
            .id("pool".to_string())
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PullAny)
            .build()
            .unwrap();

        simulation.add_process(source)?;
        simulation.add_process(delay)?;
        simulation.add_process(pool)?;

        // Connection: source -> delay (flow_rate = 5.0 per step)
        simulation.add_connection(Connection {
            source_id: "source".to_string(),
            source_port: Some("out".to_string()),
            target_id: "delay".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(5.0),
            ..Default::default()
        })?;

        // Connection: delay -> pool (flow_rate = 2.0 interpreted as 2s delay)
        simulation.add_connection(Connection {
            source_id: "delay".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(2.0),
            ..Default::default()
        })?;

        simulation.step_n(10)?;
        let state = simulation.get_simulation_state();

        if let ProcessState::Source(s) = &state.process_states["source"] {
            assert_eq!(
                s.resources_produced, 100.0,
                "Source should have produced 100 resources ((5 automatic push + 5 automatic pull) x 10 steps)"
            );
        }

        if let ProcessState::Delay(d) = &state.process_states["delay"] {
            assert_eq!(
                d.resources_received, 100.0,
                "Delay should have received 100.0 resources from source"
            );
            assert_eq!(
                d.resources_released, 80.0,
                "Not all resources should have been released due to delay"
            );
        }

        if let ProcessState::Pool(p) = &state.process_states["pool"] {
            assert_eq!(
                p.resources, 80.0,
                "Pool should have received 80.0 delayed resources"
            );
        }

        Ok(())
    }

    #[test]
    fn test_push_pull_pool_interaction() -> Result<(), SimulationError> {
        let mut simulation = create_stepped_simulation(vec![], vec![])?;

        // Push pool: starts with 10 resources, pushes any available
        let push_pool = Pool::builder()
            .id("push_pool".to_string())
            .trigger_mode(TriggerMode::Automatic)
            .state(PoolState {
                resources: 10.0,
                pending_outgoing_resources: 0.0,
            })
            .action(Action::PushAny)
            .build()
            .unwrap();

        // Pull pool: starts with 0 resources, pulls any available
        let pull_pool = Pool::builder()
            .id("pull_pool".to_string())
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PullAny)
            .build()
            .unwrap();

        simulation.add_process(push_pool)?;
        simulation.add_process(pull_pool)?;

        // Connection: push_pool -> pull_pool
        simulation.add_connection(Connection {
            source_id: "push_pool".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pull_pool".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
            ..Default::default()
        })?;

        simulation.step_n(3)?;
        let state = simulation.get_simulation_state();

        if let ProcessState::Pool(p) = &state.process_states["push_pool"] {
            assert_eq!(
                p.resources, 4.0,
                "Push pool should have transferred 6 resources"
            );
        }

        if let ProcessState::Pool(p) = &state.process_states["pull_pool"] {
            assert_eq!(
                p.resources, 6.0,
                "Pull pool should have received 6 resources"
            );
        }

        Ok(())
    }

    #[test]
    fn test_single_source_to_pool() -> Result<(), SimulationError> {
        setup();
        info!("Testing single source to pool flow");

        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
        };

        let mut sim = create_stepped_simulation(vec![source, pool], vec![connection])?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Run simulation for 5 steps
        let _ = sim.step_n(5)?;

        // Check final state
        let final_state = sim.get_simulation_state();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 5.0);
        }

        Ok(())
    }

    #[test]
    fn test_multiple_sources_to_pool() -> Result<(), SimulationError> {
        setup();
        info!("Testing multiple sources to single pool");

        let source1 = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let source2 = Process::new(Box::new(
            Source::builder().id("source2".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let connections = vec![
            Connection {
                id: "conn1".to_string(),
                source_id: "source1".to_string(),
                source_port: Some("out".to_string()),
                target_id: "pool1".to_string(),
                target_port: Some("in".to_string()),
                flow_rate: Some(1.0),
            },
            Connection {
                id: "conn2".to_string(),
                source_id: "source2".to_string(),
                source_port: Some("out".to_string()),
                target_id: "pool1".to_string(),
                target_port: Some("in".to_string()),
                flow_rate: Some(2.0),
            },
        ];

        let mut sim = create_stepped_simulation(vec![source1, source2, pool], connections)?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Run simulation for 3 steps
        let _ = sim.step_n(3)?;

        // Check final state
        let final_state = sim.get_simulation_state();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 9.0); // (1 + 2) * 3 steps
        }

        Ok(())
    }

    #[test]
    fn test_source_to_multiple_pools() -> Result<(), SimulationError> {
        setup();
        info!("Testing single source to multiple pools");

        // Create processes
        let source = Process::new(Box::new(Source::builder().id("source1").build().unwrap()));
        let pool1 = Process::new(Box::new(Pool::builder().id("pool1").build().unwrap()));
        let pool2 = Process::new(Box::new(Pool::builder().id("pool2").build().unwrap()));

        // Create connections with different flow rates
        let connections = vec![
            Connection {
                id: "conn1".to_string(),
                source_id: "source1".to_string(),
                source_port: Some("out".to_string()),
                target_id: "pool1".to_string(),
                target_port: Some("in".to_string()),
                flow_rate: Some(1.0),
            },
            Connection {
                id: "conn2".to_string(),
                source_id: "source1".to_string(),
                source_port: Some("out".to_string()),
                target_id: "pool2".to_string(),
                target_port: Some("in".to_string()),
                flow_rate: Some(2.0),
            },
        ];

        // Create and run simulation
        let mut sim = create_stepped_simulation(vec![source, pool1, pool2], connections)?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Track states at each step
        let mut states = vec![initial_state];

        // Run simulation for 3 steps
        for _ in 0..3 {
            let _ = sim.step()?;
            states.push(sim.get_simulation_state());
        }

        // Check final state
        let final_state = states.last().unwrap();

        // Pool1 should receive 1 resource per step
        let pool1_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool1_state {
            assert_eq!(state.resources, 3.0); // 1.0 * 3 steps
        }

        // Pool2 should receive 2 resources per step
        let pool2_state = final_state.process_states.get("pool2").unwrap();
        if let ProcessState::Pool(state) = pool2_state {
            assert_eq!(state.resources, 6.0); // 2.0 * 3 steps
        }

        // Check intermediate states
        for (i, state) in states.iter().enumerate() {
            // Check Pool1
            if let ProcessState::Pool(state) = &state.process_states["pool1"] {
                assert_eq!(state.resources, i as f64 * 1.0);
            }

            // Check Pool2
            if let ProcessState::Pool(state) = &state.process_states["pool2"] {
                assert_eq!(state.resources, i as f64 * 2.0);
            }
        }

        Ok(())
    }

    #[test]
    fn test_pool_with_capacity() -> Result<(), SimulationError> {
        setup();
        info!("Testing pool with capacity limit");

        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder()
                .id("pool1".to_string())
                .capacity(3.0)
                .build()
                .unwrap(),
        ));

        let connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
        };

        let mut sim = create_stepped_simulation(vec![source, pool], vec![connection])?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Run simulation for 5 steps
        let _ = sim.step_n(5)?;

        // Check final state
        let final_state = sim.get_simulation_state();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 3.0); // Should cap at capacity
        }

        Ok(())
    }

    #[test]
    fn test_invalid_connection() {
        setup();
        info!("Testing invalid connection handling");

        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let invalid_connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("invalid_port".to_string()), // Invalid port
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
        };

        let result = create_stepped_simulation(vec![source, pool], vec![invalid_connection]);
        assert!(result.is_err());
    }

    #[test]
    fn test_simulation_state_history() -> Result<(), SimulationError> {
        setup();
        info!("Testing simulation state history");

        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
        };

        let mut sim = create_stepped_simulation(vec![source, pool], vec![connection])?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Track states at each step
        let mut states = vec![initial_state];

        // Run simulation for 3 steps
        for _ in 0..3 {
            let _ = sim.step()?;
            states.push(sim.get_simulation_state());
        }

        // Check state history
        assert_eq!(states.len(), 4); // Initial state + 3 steps

        // Check progression of resources
        for (i, state) in states.iter().enumerate() {
            let pool_state = state.process_states.get("pool1").unwrap();
            if let ProcessState::Pool(state) = pool_state {
                assert_eq!(state.resources, i as f64);
            }
        }

        Ok(())
    }

    #[test]
    fn test_stepping_behavior() -> Result<(), SimulationError> {
        setup();
        info!("Testing different stepping methods");

        // Create a simple source -> pool setup
        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
        };

        // Test step()
        {
            info!("Testing single step");
            let mut sim = create_stepped_simulation(
                vec![source.clone(), pool.clone()],
                vec![connection.clone()],
            )?;

            let initial_state = sim.get_simulation_state();
            assert_eq!(initial_state.step, 0);

            let _ = sim.step()?;
            assert_eq!(sim.current_time(), 1.0);

            let state = sim.get_simulation_state();
            if let ProcessState::Pool(state) = &state.process_states["pool1"] {
                assert_eq!(state.resources, 1.0);
            }
        }

        // Test step_n()
        {
            info!("Testing step_n");
            let mut sim = create_stepped_simulation(
                vec![source.clone(), pool.clone()],
                vec![connection.clone()],
            )?;

            let initial_state = sim.get_simulation_state();
            let mut states = vec![initial_state];

            for _ in 0..3 {
                let _ = sim.step()?;
                states.push(sim.get_simulation_state());
            }

            assert_eq!(sim.current_time(), 3.0);
            assert_eq!(states.len(), 4);

            // Check progression
            for (i, state) in states.iter().enumerate() {
                if let ProcessState::Pool(state) = &state.process_states["pool1"] {
                    assert_eq!(state.resources, i as f64);
                }
            }
        }

        // Test step_until()
        {
            info!("Testing step_until");
            let mut sim = create_stepped_simulation(
                vec![source.clone(), pool.clone()],
                vec![connection.clone()],
            )?;

            let events = sim.step_until(2.5)?;

            let final_state = sim.get_simulation_state();
            if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
                assert_eq!(state.resources, 3.0, "Events: {:?}", events);
            }
        }

        Ok(())
    }

    #[test]
    fn test_step_until_time_ordering() -> Result<(), SimulationError> {
        setup();
        info!("Testing step_until time progression");

        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
        };

        let mut sim = create_stepped_simulation(vec![source, pool], vec![connection])?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        let mut states = vec![initial_state];

        // Step until 2.5 - with dt=1.0, this should give us steps at t=0,1,2,3
        let events = sim.step_until(2.5)?;
        states.push(sim.get_simulation_state());

        // Check time ordering and values
        let times: Vec<f64> = states.iter().map(|state| state.time).collect();
        assert_eq!(times, vec![0.0, 3.0], "Events: {:?}", events);

        Ok(())
    }

    #[test]
    fn test_step_until_with_fractional_dt() -> Result<(), SimulationError> {
        setup();
        info!("Testing step_until with dt=0.5");

        let source = Process::new(Box::new(
            Source::builder().id("source1".to_string()).build().unwrap(),
        ));
        let pool = Process::new(Box::new(
            Pool::builder().id("pool1".to_string()).build().unwrap(),
        ));

        let connection = Connection {
            id: "conn1".to_string(),
            source_id: "source1".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool1".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(0.5),
        };

        let mut sim = Simulation::new(vec![source, pool], vec![connection])?;

        let mut stepper = Stepper::builder()
            .id("stepper".to_string())
            .build()
            .unwrap();
        stepper.set_dt(0.5)?;
        let stepper_process = Process::new(Box::new(stepper));
        sim.add_process(stepper_process).unwrap();

        // Get initial state
        let initial_state = sim.get_simulation_state();
        let mut states = vec![initial_state];

        // Step until 2.5 - with dt=0.5, this should give us steps at t=0,0.5,1.0,1.5,2.0,2.5
        let events = sim.step_until(2.5)?;
        states.push(sim.get_simulation_state());

        // Check time ordering and values
        let times: Vec<f64> = states.iter().map(|state| state.time).collect();
        assert_eq!(times, vec![0.0, 2.5], "Events: {:?}", events);

        Ok(())
    }

    #[test]
    fn test_automatic_source_and_pool_transfer() -> Result<(), SimulationError> {
        let mut simulation = create_stepped_simulation(vec![], vec![])?;

        // Source: produces 1 resource per step automatically
        let source = Source::builder()
            .id("source")
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PushAny)
            .build()
            .unwrap();

        // Pool: pulls 1 resource per step automatically
        let pool = Pool::builder()
            .id("pool")
            .trigger_mode(TriggerMode::Automatic)
            .action(Action::PullAny)
            .capacity(-1.0) // unlimited capacity
            .overflow(Overflow::Drain)
            .build()
            .unwrap();

        simulation.add_process(source)?;
        simulation.add_process(pool)?;

        // Connect source to pool with flow_rate of 1 resource per step
        simulation.add_connection(Connection {
            source_id: "source".to_string(),
            source_port: Some("out".to_string()),
            target_id: "pool".to_string(),
            target_port: Some("in".to_string()),
            flow_rate: Some(1.0),
            ..Default::default()
        })?;

        // Run for 10 steps
        simulation.step_n(10)?;

        let state = simulation.get_simulation_state();

        if let ProcessState::Source(source_state) = &state.process_states["source"] {
            assert_eq!(
                source_state.resources_produced, 20.0,
                "Source should produce 20 resources over 10 steps"
            );
        }

        if let ProcessState::Pool(pool_state) = &state.process_states["pool"] {
            assert_eq!(
                pool_state.resources, 20.0,
                "Pool should receive 20 resources from source over 10 steps"
            );
        }

        Ok(())
    }
}
