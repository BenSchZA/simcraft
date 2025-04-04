mod common;

use crate::common::{create_stepped_simulation, setup};
use simcraft::{
    model::{
        nodes::{Drain, Pool},
        process_state::PoolState,
        Connection, ProcessState,
    },
    prelude::StatefulSimulation,
    simulator::Simulate,
};

#[test]
fn test_connection_order_priority() {
    setup();

    // Create a simulation with a pool and three drains
    let mut sim = create_stepped_simulation(vec![], vec![]).unwrap();

    // Create a passive pool with 2 resources
    let pool = Pool::builder()
        .id("pool")
        .state(PoolState {
            resources: 2.0,
            ..Default::default()
        })
        .build()
        .unwrap();

    // Create three automatic pull-any drains
    let drain1 = Drain::new("drain1");
    let drain2 = Drain::new("drain2");
    let drain3 = Drain::new("drain3");

    // Add processes to simulation
    sim.add_process(pool).unwrap();
    sim.add_process(drain1).unwrap();
    sim.add_process(drain2).unwrap();
    sim.add_process(drain3).unwrap();

    // Create connections in sequence - this order should determine resource distribution
    let conn1 = Connection::new(
        "conn1".to_string(),
        "pool".to_string(),
        Some("out".to_string()),
        "drain1".to_string(),
        Some("in".to_string()),
        Some(1.0),
    );
    let conn2 = Connection::new(
        "conn2".to_string(),
        "pool".to_string(),
        Some("out".to_string()),
        "drain2".to_string(),
        Some("in".to_string()),
        Some(1.0),
    );
    let conn3 = Connection::new(
        "conn3".to_string(),
        "pool".to_string(),
        Some("out".to_string()),
        "drain3".to_string(),
        Some("in".to_string()),
        Some(1.0),
    );

    // Add connections in sequence
    sim.add_connection(conn1).unwrap();
    sim.add_connection(conn2).unwrap();
    sim.add_connection(conn3).unwrap();

    // Run simulation for one step
    let events = sim.step().unwrap();
    let state = sim.get_simulation_state();

    // Assert that only drain1 and drain2 got resources
    if let ProcessState::Pool(pool_state) = state.process_states.get("pool").unwrap() {
        assert_eq!(pool_state.resources, 0.0);
    }

    if let ProcessState::Drain(drain_state) = state.process_states.get("drain1").unwrap() {
        assert_eq!(drain_state.resources_consumed, 1.0);
    }

    if let ProcessState::Drain(drain_state) = state.process_states.get("drain2").unwrap() {
        assert_eq!(drain_state.resources_consumed, 1.0);
    }

    if let ProcessState::Drain(drain_state) = state.process_states.get("drain3").unwrap() {
        assert_eq!(drain_state.resources_consumed, 0.0);
    }
}
