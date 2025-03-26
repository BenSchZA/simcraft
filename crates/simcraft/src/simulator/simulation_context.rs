use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::{connection::Connection, ProcessContext};

type ProcessId = String;
type PortId = String;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulationContext {
    // pub(crate) rng: Rng,
    pub(crate) current_step: u64,
    pub(crate) current_time: f64,
    pub(crate) dt: f64,
    pub(crate) input_map: HashMap<ProcessId, HashMap<Option<PortId>, Vec<Connection>>>,
    pub(crate) output_map: HashMap<ProcessId, HashMap<Option<PortId>, Vec<Connection>>>,
}

impl Default for SimulationContext {
    fn default() -> Self {
        Self {
            current_step: 0,
            current_time: 0.0,
            dt: 1.0,
            input_map: HashMap::new(),
            output_map: HashMap::new(),
        }
    }
}

impl SimulationContext {
    // TODO Implement global rng
    // pub fn rng(&self) -> Rng {
    //     self.rng.clone()
    // }

    pub fn current_step(&self) -> u64 {
        self.current_step
    }

    pub fn set_current_step(&mut self, step: u64) {
        self.current_step = step;
    }

    pub fn increment_current_step(&mut self) {
        self.current_step += 1;
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn set_current_time(&mut self, time: f64) {
        self.current_time = time;
    }
}

impl SimulationContext {
    pub fn context_for_process<'a>(&'a self, process_id: &str) -> ProcessContext<'a> {
        ProcessContext::new(
            self.current_step,
            self.current_time,
            self.process_inputs(process_id),
            self.process_outputs(process_id),
        )
    }

    /// Returns all input connections for the given process.
    pub fn process_inputs(&self, process_id: &str) -> Vec<&Connection> {
        self.input_map
            .get(process_id)
            .map(|ports| ports.values().flat_map(|v| v.iter()).collect())
            .unwrap_or_default()
    }

    /// Returns input connections for a specific port (or the default port if `None`).
    pub fn process_inputs_for_port(&self, process_id: &str, port: Option<&str>) -> &[Connection] {
        let port_str = port.map(String::from);
        self.input_map
            .get(process_id)
            .and_then(|ports| ports.get(&port_str))
            .map(|c| c.as_slice())
            .unwrap_or(&[])
    }

    /// Returns all output connections for the given process.
    pub fn process_outputs(&self, process_id: &str) -> Vec<&Connection> {
        self.output_map
            .get(process_id)
            .map(|ports| ports.values().flat_map(|v| v.iter()).collect())
            .unwrap_or_default()
    }

    /// Returns output connections for a specific port (or the default port if `None`).
    pub fn process_outputs_for_port(&self, process_id: &str, port: Option<&str>) -> &[Connection] {
        let port_str = port.map(String::from);
        self.output_map
            .get(process_id)
            .and_then(|ports| ports.get(&port_str))
            .map(|c| c.as_slice())
            .unwrap_or(&[])
    }
}
