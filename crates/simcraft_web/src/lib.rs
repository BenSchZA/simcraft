use js_sys::Array;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use simcraft::simulator::Simulate;
use simcraft::simulator::Simulation as CoreSimulation;

pub mod errors;
pub mod logging;

use errors::wasm_error;
use logging::init_logging;

#[wasm_bindgen]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Simulation {
    inner: CoreSimulation,
}

#[wasm_bindgen]
impl Simulation {
    pub fn new(processes: &str, connections: &str) -> Result<Self, JsValue> {
        init_logging();
        debug!("Creating new simulation");

        let simulation = Self {
            inner: CoreSimulation::new(
                serde_json::from_str(processes).unwrap(),
                serde_json::from_str(connections).unwrap(),
            )
            .map_err(wasm_error)?,
        };

        Ok(simulation)
    }

    pub fn step(&mut self) -> Result<Array, JsValue> {
        let results = self.inner.step().map_err(wasm_error)?;
        let js_results = results
            .into_iter()
            .map(|s| to_value(&s).unwrap_or(JsValue::NULL))
            .collect();

        Ok(js_results)
    }

    pub fn step_until(&mut self, until: f64) -> Result<Array, JsValue> {
        let results = self.inner.step_until(until).map_err(wasm_error)?;
        let js_results = results
            .into_iter()
            .map(|s| to_value(&s).unwrap_or(JsValue::NULL))
            .collect();

        Ok(js_results)
    }

    pub fn step_n(&mut self, n: usize) -> Result<Array, JsValue> {
        let results = self.inner.step_n(n).map_err(wasm_error)?;
        let js_results = results
            .into_iter()
            .map(|s| to_value(&s).unwrap_or(JsValue::NULL))
            .collect();

        Ok(js_results)
    }
}
