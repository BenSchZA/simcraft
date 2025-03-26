use log::debug;
use serde_json::Value;
use serde_wasm_bindgen::from_value;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_test::wasm_bindgen_test;

use simcraft::utils::SimulationError;
use simcraft_web::errors::CustomJsError;
use simcraft_web::Simulation;

const TEST_PROCESSES: &str = r#"
[
    {
        "type": "Stepper",
        "id": "stepper"
    },
    {
        "type": "Source",
        "id": "source-1"
    },
    {
        "type": "Pool",
        "id": "pool-1"
    }
]"#;

const TEST_DUPLICATE_PROCESSES: &str = r#"
[
    {
        "type": "Pool",
        "id": "pool-1"
    },
    {
        "type": "Pool",
        "id": "pool-1"
    }
]"#;

const TEST_CONNECTIONS: &str = r#"
[
{
    "id": "connection-1",
    "sourceID": "source-1",
    "sourcePort": "out",
    "targetID": "pool-1",
    "targetPort": "in"
}
]"#;

#[test]
#[wasm_bindgen_test]
fn simulation_with_duplicate_processes() {
    let simulation_result = Simulation::new(TEST_DUPLICATE_PROCESSES, "[]");

    assert!(
        simulation_result.is_err(),
        "Expected simulation initialisation to return a `DuplicateProcess` error."
    );

    let error_value = simulation_result.unwrap_err();
    let error_json: Value = from_value(error_value).expect("Failed to convert JsValue to JSON");

    let simulation_error = SimulationError::DuplicateProcess("pool-1".to_string());
    let expected_error = CustomJsError {
        error: format!("{:?}", simulation_error,),
        message: simulation_error.to_string(),
    };
    let expected_json: Value = to_value(&expected_error)
        .and_then(from_value)
        .expect("Failed to convert CustomJsError to JSON");

    assert_eq!(error_json, expected_json, "Error JSON objects do not match");
}

#[test]
#[wasm_bindgen_test]
fn simulation_step() {
    let mut simulation = Simulation::new(TEST_PROCESSES, TEST_CONNECTIONS).unwrap();

    let results = simulation.step().unwrap();
    debug!("Step results length: {}", results.length());
    assert!(
        results.length() > 0,
        "Expected non-empty results after step."
    );

    // Get the last state from results
    let last_index = results.length() - 1;
    debug!("Getting last state at index: {}", last_index);
    let last_state = results.get(last_index);
    debug!("Last state raw: {:?}", last_state);

    let state_value: Value = from_value(last_state).expect("Failed to convert state to JSON");
    debug!("Converted state value: {:?}", state_value);

    // Check pool resources
    let process_states = state_value
        .get("process_states")
        .expect("No process_states found");
    debug!("Process states: {:?}", process_states);

    let pool_1 = process_states.get("pool-1").expect("No pool-1 found");
    debug!("Pool-1 state: {:?}", pool_1);

    let pool_state = pool_1.get("Pool").expect("No Pool variant found");
    debug!("Pool state: {:?}", pool_state);

    let resources = pool_state.get("resources").expect("No resources found");
    debug!("Resources: {:?}", resources);

    let pool_resources = resources.as_f64().expect("Resources not a number");
    assert_eq!(
        pool_resources, 1.0,
        "Pool should have received 1.0 resources from source"
    );

    // Check source resources produced
    let source_1 = process_states.get("source-1").expect("No source-1 found");
    debug!("Source-1 state: {:?}", source_1);

    let source_state = source_1.get("Source").expect("No Source variant found");
    debug!("Source state: {:?}", source_state);

    let produced = source_state
        .get("resources_produced")
        .expect("No resources_produced found");
    debug!("Resources produced: {:?}", produced);

    let resources_produced = produced.as_f64().expect("Resources produced not a number");
    assert_eq!(
        resources_produced, 1.0,
        "Source should have produced 1.0 resources"
    );
}
