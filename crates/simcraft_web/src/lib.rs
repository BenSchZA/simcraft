use js_sys::Array;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use simcraft::prelude::*;

pub mod errors;
pub mod logging;

use errors::wasm_error;
use logging::init_logging;

#[wasm_bindgen]
#[derive(Default, Serialize, Deserialize)]
pub struct WebSimulation {
    inner: Simulation,
}

#[wasm_bindgen]
impl WebSimulation {
    pub fn new(processes: &str, connections: &str) -> Self {
        init_logging();
        debug!("Creating new simulation");

        Self {
            inner: Simulation::new(
                serde_json::from_str(processes).unwrap(),
                serde_json::from_str(connections).unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn step(&mut self) -> Result<Array, JsValue> {
        let results = self.inner.step().map_err(wasm_error)?;

        Ok(results
            .into_iter()
            .map(|s| to_value(&s).unwrap_or(JsValue::NULL))
            .collect())
    }

    pub fn step_until(&mut self, until: f64) -> Result<Array, JsValue> {
        let results = self.inner.step_until(until).map_err(wasm_error)?;

        Ok(results
            .into_iter()
            .map(|s| to_value(&s).unwrap_or(JsValue::NULL))
            .collect())
    }

    pub fn step_n(&mut self, n: usize) -> Result<Array, JsValue> {
        let results = self.inner.step_n(n).map_err(wasm_error)?;

        Ok(results
            .into_iter()
            .map(|s| to_value(&s).unwrap_or(JsValue::NULL))
            .collect())
    }
}
