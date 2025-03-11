mod common;

#[cfg(test)]
mod tests {
    use log::info;
    use simcraft::dsl::*;
    use simcraft::model::process_state::ProcessState;
    use simcraft::model::nodes::{Action, TriggerMode};
    use simcraft::simulator::Simulate;
    use simcraft::utils::errors::SimulationError;

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

        // Run the simulation for 5 steps
        let results = sim.step_n(5)?;

        // Verify the results
        let final_state = results.last().unwrap();
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

        // Run the simulation for 5 steps
        let results = sim.step_n(5)?;

        // Verify the results
        let final_state = results.last().unwrap();
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

        // Run the simulation for 5 steps
        let results = sim.step_n(5)?;

        // Verify the results
        let final_state = results.last().unwrap();
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
        let results = run_simulation! {
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

        // Verify the results
        let final_state = results.last().unwrap();
        let pool_state = final_state.process_states.get("pool1").unwrap();
        if let ProcessState::Pool(state) = pool_state {
            assert_eq!(state.resources, 5.0);
        }

        Ok(())
    }

    #[test]
    fn test_source_pool_loop() -> Result<(), SimulationError> {
        setup();

        let results = run_simulation! {
            steps: 5,
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

        // Verify the final state of all processes
        let final_state = results.last().unwrap();
        
        // Check source1's state
        if let ProcessState::Source(state) = &final_state.process_states["source1"] {
            info!("Source1 resources produced: {}", state.resources_produced);
            assert_eq!(state.resources_produced, 5.0, "Source should have produced 5 resources");
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
            let results = run_simulation! {
                steps: 5,
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

            // Verify the final state of all processes
            let final_state = results.last().unwrap();

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
            let results = run_simulation! {
                steps: 5,
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

            // Verify the final state of all processes
            let final_state = results.last().unwrap();

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
            let results = run_simulation! {
                steps: 5,
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

            // Verify the final state of all processes
            let final_state = results.last().unwrap();

            // Check pool1's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
                info!("Pool1 resources: {}", state.resources);
                assert_eq!(state.resources, 0.0, "Pool1 should have 0 resources");
            }

            // Check pool2's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool2"] {
                info!("Pool2 resources: {}", state.resources);
                assert_eq!(state.resources, 0.0, "Pool2 should have 0 resources");
            }

            // Check pool3's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool3"] {
                info!("Pool3 resources: {}", state.resources);
                assert_eq!(state.resources, 1.0, "Pool3 should have 1 resources");
            }
        }

        Ok(())
    }

    #[test]
    fn test_drain() -> Result<(), SimulationError> {
        setup();
        info!("Testing drain functionality");

        {
            let results = run_simulation! {
                steps: 5,
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

            // Verify the final state of all processes
            let final_state = results.last().unwrap();
            
            // Check source's state
            if let ProcessState::Source(state) = &final_state.process_states["source1"] {
                info!("Source1 resources produced: {}", state.resources_produced);
                assert_eq!(state.resources_produced, 10.0, "Source should have produced 10 resources (2.0 per step * 5 steps)");
            }

            // Check pool's state
            if let ProcessState::Pool(state) = &final_state.process_states["pool1"] {
                info!("Pool1 resources: {}", state.resources);
                assert_eq!(state.resources, 6.0, "Pool should have 6 resources (gained 10, lost 4)");
            }

            // Check drain's state
            if let ProcessState::Drain(state) = &final_state.process_states["drain1"] {
                info!("Drain1 resources consumed: {}", state.resources_consumed);
                assert_eq!(state.resources_consumed, 4.0, "Drain should have consumed 4 resources (0 in first step, then 1.0 per step * 4 steps)");
            }
        }

        {
            // Test multiple inputs to drain
            let results = run_simulation! {
                steps: 5,
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

            let final_state = results.last().unwrap();
            
            // Check drain's state with multiple inputs
            if let ProcessState::Drain(state) = &final_state.process_states["drain1"] {
                info!("Drain1 resources consumed: {}", state.resources_consumed);
                assert_eq!(state.resources_consumed, 30.0, "Drain should have consumed 30 resources ((1.0 + 2.0) * 2 push/pull per step * 5 steps)");
            }
        }

        {
            // Test multiple inputs to drain
            let results = run_simulation! {
                steps: 5,
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

            let final_state = results.last().unwrap();
            
            // Check drain's state with multiple inputs
            if let ProcessState::Drain(state) = &final_state.process_states["drain1"] {
                info!("Drain1 resources consumed: {}", state.resources_consumed);
                assert_eq!(state.resources_consumed, 15.0, "Drain should have consumed 15 resources ((1.0 + 2.0) * 2 per step * 5 steps)");
            }
        }

        Ok(())
    }
}