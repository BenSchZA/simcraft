use js_sys::Array;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use simcraft::model::Connection;
use simcraft::model::Process;
use simcraft::model::ProcessState;
use simcraft::simulator::SimulationState;
use wasm_bindgen::prelude::*;

use simcraft::simulator::Simulate;
use simcraft::simulator::Simulation as CoreSimulation;
use simcraft::simulator::StatefulSimulation;

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

        let processes: Vec<Process> = serde_json::from_str(processes).unwrap();
        let connections: Vec<Connection> = serde_json::from_str(connections).unwrap();
        let simulation = Self {
            inner: CoreSimulation::new(processes, connections).map_err(wasm_error)?,
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

    pub fn get_simulation_state(&self) -> JsValue {
        let state: SimulationState = self.inner.get_simulation_state();
        let js_state = to_value(&state).unwrap_or(JsValue::NULL);
        js_state
    }

    pub fn get_process_state(&self, process_id: &str) -> Result<JsValue, JsValue> {
        let state: ProcessState = self.inner.get_process_state(process_id).map_err(wasm_error)?;
        let js_state = to_value(&state).unwrap_or(JsValue::NULL);
        Ok(js_state)
    }

    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.inner.reset().map_err(wasm_error)?;
        Ok(())
    }

    pub fn add_process(&mut self, process: &str) -> Result<(), JsValue> {
        let process: Process = serde_json::from_str(process).map_err(wasm_error)?;
        self.inner.add_process(process).map_err(wasm_error)?;
        Ok(())
    }

    pub fn remove_process(&mut self, process_id: &str) -> Result<(), JsValue> {
        self.inner.remove_process(process_id).map_err(wasm_error)?;
        Ok(())
    }

    pub fn get_processes(&self) -> Result<Array, JsValue> {
        let processes = self.inner.processes();
        let js_processes = processes
            .values()
            .map(|p| to_value(&p).unwrap_or(JsValue::NULL))
            .collect();
        Ok(js_processes)
    }

    pub fn add_connection(&mut self, connection: &str) -> Result<(), JsValue> {
        let connection: Connection = serde_json::from_str(connection).map_err(wasm_error)?;
        self.inner.add_connection(connection).map_err(wasm_error)?;
        Ok(())
    }

    pub fn remove_connection(&mut self, connection_id: &str) -> Result<(), JsValue> {
        self.inner.remove_connection(connection_id).map_err(wasm_error)?;
        Ok(())
    }
}
