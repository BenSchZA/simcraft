mod common;

#[cfg(test)]
mod tests {
    use log::info;
    use simcraft::dsl::*;
    use simcraft::model::nodes::{Action, TriggerMode};
    use simcraft::model::process_state::ProcessState;
    use simcraft::prelude::*;
    use simcraft::simulator::SimulationState;

    use crate::common::setup;

    #[test]
    fn test_dsl_single_source_to_pool() -> Result<(), SimulationError> {
        setup();
        info!("Testing DSL with single source to pool flow");

        // Create a simulation using the DSL
        let mut sim = simulation! {
            processes {
                source "source1" {}
                pool "pool1" {}
            }
            connections {
                "source1.out" -> "pool1.in" {
                    id: "conn1",
                    flow_rate: 1.0
                }
            }
        }?;

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
    fn test_dsl_multiple_sources_to_pool() -> Result<(), SimulationError> {
        setup();
        info!("Testing DSL with multiple sources to pool flow");

        // Create a simulation using the DSL
        let mut sim = simulation! {
            processes {
                source "source1" {}
                source "source2" {}
                pool "pool1" {}
            }
            connections {
                "source1.out" -> "pool1.in" {
                    id: "conn1",
                    flow_rate: 1.0
                }
                "source2.out" -> "pool1.in" {
                    id: "conn2",
                    flow_rate: 2.0
                }
            }
        }?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Run simulation for 5 steps
        let _ = sim.step_n(5)?;

        // Check final state
        let final_state = sim.get_simulation_state();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 15.0); // 5 steps * (1.0 + 2.0) = 15.0
        }

        Ok(())
    }

    #[test]
    fn test_dsl_pool_with_capacity() -> Result<(), SimulationError> {
        setup();
        info!("Testing DSL with pool capacity");

        // Create a simulation using the DSL with a pool that has a capacity
        let mut sim = simulation! {
            processes {
                source "source1" {}
                pool "pool1" {
                    capacity: 3.0,
                }
            }
            connections {
                "source1.out" -> "pool1.in" {
                    id: "conn1",
                    flow_rate: 1.0
                }
            }
        }?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Run simulation for 5 steps
        let _ = sim.step_n(5)?;

        // Check final state
        let final_state = sim.get_simulation_state();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 3.0); // Should be capped at capacity
        }

        Ok(())
    }

    #[test]
    fn test_run_simulation_macro() -> Result<(), SimulationError> {
        setup();
        info!("Testing run_simulation macro");

        // Use the run_simulation macro to create and run a simulation
        let (_, states) = run_simulation! {
            steps: 5,
            processes {
                source "source1" {}
                pool "pool1" {}
            }
            connections {
                "source1.out" -> "pool1.in" {
                    id: "conn1",
                    flow_rate: 1.0
                }
            }
        }?;

        assert_eq!(states.len(), 6);

        // Check initial state
        let initial_state = &states.first().unwrap();
        assert_eq!(initial_state.step, 0);
        if let ProcessState::Pool(state) = &initial_state.process_states["pool1"] {
            assert_eq!(state.resources, 0.0);
        }

        // Check final state
        let final_state = &states.last().unwrap();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 5.0);
        }

        Ok(())
    }

    #[test]
    fn test_source_pool_loop() -> Result<(), SimulationError> {
        setup();

        let mut sim = simulation! {
            processes {
                source "source1" {}
                pool "pool1" {
                    trigger_mode: TriggerMode::Automatic,
                    action: Action::PushAny,
                }
                pool "pool2" {
                    trigger_mode: TriggerMode::Automatic,
                    action: Action::PushAny,
                }
            }
            connections {
                "source1.out" -> "pool1.in" {
                    id: "conn1"
                }
                "pool1.out" -> "pool2.in" {
                    id: "conn2"
                }
                "pool2.out" -> "pool1.in" {
                    id: "conn3"
                }
            }
        }?;

        // Get initial state
        let initial_state = sim.get_simulation_state();
        assert_eq!(initial_state.step, 0);

        // Run simulation for 5 steps
        let _ = sim.step_n(5)?;

        // Check final state
        let final_state = sim.get_simulation_state();

        // Check source1's state
        if let ProcessState::Source(state) = &final_state.process_states["source1"] {
            info!("Source1 resources produced: {}", state.resources_produced);
            assert_eq!(
                state.resources_produced, 5.0,
                "Source should have produced 5 resources"
            );
        }

        // Check pool1's state
        if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
            info!("Pool1 resources: {}", state.resources);
            assert_eq!(state.resources, 4.0, "Pool1 should have 5 resources");
        }

        // Check pool2's state
        if let ProcessState::Pool(state) = &final_state.process_states["pool2"] {
            info!("Pool2 resources: {}", state.resources);
            assert_eq!(state.resources, 1.0, "Pool2 should have 5 resources");
        }

        Ok(())
    }

    #[test]
    fn test_pool_loop() -> Result<(), SimulationError> {
        setup();

        {
            let mut sim = simulation! {
                processes {
                    pool "pool1" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PushAny,
                        resources: 1.0,
                    }
                    pool "pool2" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PushAny,
                    }
                }
                connections {
                    "pool1.out" -> "pool2.in" {
                        id: "conn1"
                    }
                    "pool2.out" -> "pool1.in" {
                        id: "conn2"
                    }
                }
            }?;

            // Get initial state
            let initial_state = sim.get_simulation_state();
            assert_eq!(initial_state.step, 0);

            // Run simulation for 5 steps
            let _ = sim.step_n(5)?;

            // Check final state
            let final_state = sim.get_simulation_state();

            // Check pool1's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
                info!("Pool1 resources: {}", state.resources);
                assert_eq!(state.resources, 0.0, "Pool1 should have 0 resources");
            }

            // Check pool2's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool2"] {
                info!("Pool2 resources: {}", state.resources);
                assert_eq!(state.resources, 1.0, "Pool2 should have 1 resources");
            }
        }

        {
            let mut sim = simulation! {
                processes {
                    pool "pool1" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PushAny,
                        resources: 1.0,
                    }
                    pool "pool2" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PushAny,
                        resources: 1.0,
                    }
                }
                connections {
                    "pool1.out" -> "pool2.in" {
                        id: "conn1"
                    }
                    "pool2.out" -> "pool1.in" {
                        id: "conn2"
                    }
                }
            }?;

            // Get initial state
            let initial_state = sim.get_simulation_state();
            assert_eq!(initial_state.step, 0);

            // Run simulation for 5 steps
            let _ = sim.step_n(5)?;

            // Check final state
            let final_state = sim.get_simulation_state();

            // Check pool1's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
                info!("Pool1 resources: {}", state.resources);
                assert_eq!(state.resources, 1.0, "Pool1 should have 1 resources");
            }

            // Check pool2's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool2"] {
                info!("Pool2 resources: {}", state.resources);
                assert_eq!(state.resources, 1.0, "Pool2 should have 1 resources");
            }
        }

        {
            let mut sim = simulation! {
                processes {
                    pool "pool1" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PullAny,
                        resources: 1.0,
                    }
                    pool "pool2" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PullAny,
                    }
                    pool "pool3" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PullAny,
                    }
                }
                connections {
                    "pool1.out" -> "pool2.in" {
                        id: "conn1"
                    }
                    "pool2.out" -> "pool3.in" {
                        id: "conn2"
                    }
                    "pool3.out" -> "pool1.in" {
                        id: "conn3"
                    }
                }
            }?;

            fn get_pool_values(state: &SimulationState) -> (f64, f64, f64) {
                (
                    match &state.process_states["pool1"] {
                        ProcessState::Pool(p) => p.resources,
                        _ => panic!("Expected Pool1"),
                    },
                    match &state.process_states["pool2"] {
                        ProcessState::Pool(p) => p.resources,
                        _ => panic!("Expected Pool2"),
                    },
                    match &state.process_states["pool3"] {
                        ProcessState::Pool(p) => p.resources,
                        _ => panic!("Expected Pool3"),
                    },
                )
            }

            let state = sim.get_simulation_state();
            assert_eq!(get_pool_values(&state), (1.0, 0.0, 0.0));

            sim.step()?;

            let state = sim.get_simulation_state();
            assert_eq!(get_pool_values(&state), (0.0, 1.0, 0.0));

            sim.step()?;

            let state = sim.get_simulation_state();
            assert_eq!(get_pool_values(&state), (0.0, 0.0, 1.0));

            sim.step()?;

            let state = sim.get_simulation_state();
            assert_eq!(get_pool_values(&state), (1.0, 0.0, 0.0));
        }

        Ok(())
    }

    #[test]
    fn test_drain() -> Result<(), SimulationError> {
        setup();
        info!("Testing drain functionality");

        {
            let mut sim = simulation! {
                processes {
                    source "source1" {}
                    pool "pool1" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PushAny,
                    }
                    drain "drain1" {
                        trigger_mode: TriggerMode::Passive,
                        action: Action::PullAny,
                    }
                }
                connections {
                    "source1.out" -> "pool1.in" {
                        id: "conn1",
                        flow_rate: 2.0
                    }
                    "pool1.out" -> "drain1.in" {
                        id: "conn2",
                        flow_rate: 1.0
                    }
                }
            }?;

            // Get initial state
            let initial_state = sim.get_simulation_state();
            assert_eq!(initial_state.step, 0);

            // Run simulation for 5 steps
            let _ = sim.step_n(5)?;

            // Check final state
            let final_state = sim.get_simulation_state();

            // Check source's state
            if let ProcessState::Source(state) = &final_state.process_states["source1"] {
                info!("Source1 resources produced: {}", state.resources_produced);
                assert_eq!(
                    state.resources_produced, 10.0,
                    "Source should have produced 10 resources (2.0 per step * 5 steps)"
                );
            }

            // Check pool's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
                info!("Pool1 resources: {}", state.resources);
                assert_eq!(
                    state.resources, 6.0,
                    "Pool should have 6 resources (gained 10, lost 4)"
                );
            }

            // Check drain's state
            if let ProcessState::Drain(state) = &final_state.process_states["drain1"] {
                info!("Drain1 resources consumed: {}", state.resources_consumed);
                assert_eq!(state.resources_consumed, 4.0, "Drain should have consumed 4 resources (0 in first step, then 1.0 per step * 4 steps)");
            }
        }

        {
            // Test multiple inputs to drain
            let mut sim = simulation! {
                processes {
                    source "source1" {}
                    source "source2" {}
                    drain "drain1" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PullAny,
                    }
                }
                connections {
                    "source1.out" -> "drain1.in" {
                        id: "conn1",
                        flow_rate: 1.0
                    }
                    "source2.out" -> "drain1.in" {
                        id: "conn2",
                        flow_rate: 2.0
                    }
                }
            }?;

            // Get initial state
            let initial_state = sim.get_simulation_state();
            assert_eq!(initial_state.step, 0);

            // Run simulation for 5 steps
            let _ = sim.step_n(5)?;

            // Check final state
            let final_state = sim.get_simulation_state();

            // Check drain's state with multiple inputs
            if let ProcessState::Drain(state) = &final_state.process_states["drain1"] {
                info!("Drain1 resources consumed: {}", state.resources_consumed);
                assert_eq!(state.resources_consumed, 30.0, "Drain should have consumed 30 resources ((1.0 + 2.0) * 2 push/pull per step * 5 steps)");
            }
        }

        {
            // Test multiple inputs to drain
            let mut sim = simulation! {
                processes {
                    source "source1" {
                        trigger_mode: TriggerMode::Passive,
                    }
                    source "source2" {
                        trigger_mode: TriggerMode::Passive,
                    }
                    drain "drain1" {
                        trigger_mode: TriggerMode::Automatic,
                        action: Action::PullAny,
                    }
                }
                connections {
                    "source1.out" -> "drain1.in" {
                        id: "conn1",
                        flow_rate: 1.0
                    }
                    "source2.out" -> "drain1.in" {
                        id: "conn2",
                        flow_rate: 2.0
                    }
                }
            }?;

            // Get initial state
            let initial_state = sim.get_simulation_state();
            assert_eq!(initial_state.step, 0);

            // Run simulation for 5 steps
            let _ = sim.step_n(5)?;

            // Check final state
            let final_state = sim.get_simulation_state();

            // Check drain's state with multiple inputs
            if let ProcessState::Drain(state) = &final_state.process_states["drain1"] {
                info!("Drain1 resources consumed: {}", state.resources_consumed);
                assert_eq!(
                    state.resources_consumed, 15.0,
                    "Drain should have consumed 15 resources ((1.0 + 2.0) * 2 per step * 5 steps)"
                );
            }
        }

        Ok(())
    }
}
