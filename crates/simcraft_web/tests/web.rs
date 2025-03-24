use serde_json::Value;
use serde_wasm_bindgen::from_value;
use serde_wasm_bindgen::to_value;
use simcraft::utils::SimulationError;
use simcraft_web::errors::CustomJsError;
use simcraft_web::WebSimulation;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::console::debug;

wasm_bindgen_test_configure!(run_in_browser);

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
fn simulation_step_no_events() {
    let mut simulation = WebSimulation::new(TEST_PROCESSES, "");

    let result = simulation.step();
    assert!(
        result.is_err(),
        "Expected step() to return a `NoEvents` error."
    );

    let error_value = result.unwrap_err();
    let error_json: Value = from_value(error_value).expect("Failed to convert JsValue to JSON");

    let expected_error = CustomJsError {
        error: format!("{:?}", SimulationError::NoEvents),
        message: SimulationError::NoEvents.to_string(),
    };
    let expected_json: Value = to_value(&expected_error)
        .and_then(from_value)
        .expect("Failed to convert CustomJsError to JSON");

    assert_eq!(error_json, expected_json, "Error JSON objects do not match");
}

#[test]
#[wasm_bindgen_test]
fn simulation_step() {
    let mut simulation = WebSimulation::new(TEST_PROCESSES, TEST_CONNECTIONS);

    let result = simulation.step();
    let events = result.unwrap();
    assert!(events.length() > 0, "Expected non-empty events after step.");
}
