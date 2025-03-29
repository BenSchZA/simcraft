mod common;

#[cfg(test)]
mod tests {
    use log::info;

    use simcraft::model::nodes::Pool;
    use simcraft::model::nodes::Source;
    use simcraft::model::nodes::Stepper;
    use simcraft::model::ProcessState;
    use simcraft::simulator::simulation_trait::StatefulSimulation;

    use simcraft::prelude::*;

    use crate::common::{create_stepped_simulation, setup};

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
}
