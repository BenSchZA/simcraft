use serde_json::Value;
use serde_wasm_bindgen::from_value;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_test::wasm_bindgen_test;

use simcraft::utils::SimulationError;
use simcraft_web::errors::CustomJsError;
use simcraft_web::WebSimulation;

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
    let simulation_result = WebSimulation::new(TEST_DUPLICATE_PROCESSES, "[]");

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
    let mut simulation = WebSimulation::new(TEST_PROCESSES, TEST_CONNECTIONS).unwrap();

    let results = simulation.step().unwrap();
    assert!(
        results.length() > 0,
        "Expected non-empty results after step."
    );
}
