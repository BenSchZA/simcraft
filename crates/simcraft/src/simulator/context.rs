use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::connection::Connection;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimulationContext {
    // pub(crate) rng: Rng,
    pub(crate) current_time: f64,
    pub(crate) dt: f64,
    input_map: HashMap<String, PortMap>,
    output_map: HashMap<String, PortMap>,
}

impl Default for SimulationContext {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            dt: 1.0, // TODO Dynamically update this from the simulation steps
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

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn set_current_time(&mut self, time: f64) {
        self.current_time = time;
    }

    // TODO Consider whether this should be implemented in simulation
    pub fn add_connection(&mut self, connection: Connection) {
        self.input_map
            .entry(connection.target_id.clone())
            .or_default()
            .connections
            .entry(connection.target_port.clone())
            .or_default()
            .push(connection.clone());

        self.output_map
            .entry(connection.source_id.clone())
            .or_default()
            .connections
            .entry(connection.source_port.clone())
            .or_default()
            .push(connection);
    }

    pub fn get_inputs(&self, process_id: &str, port: Option<&str>) -> &[Connection] {
        // TODO Ensure inputs and outputs are returned sorted by creation time
        self.input_map
            .get(process_id)
            .map(|port_map| port_map.get_connections(port))
            .unwrap_or(&[])
    }

    pub fn get_outputs(&self, process_id: &str, port: Option<&str>) -> &[Connection] {
        // TODO Ensure inputs and outputs are returned sorted by creation time
        self.output_map
            .get(process_id)
            .map(|port_map| port_map.get_connections(port))
            .unwrap_or(&[])
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct PortMap {
    connections: HashMap<Option<String>, Vec<Connection>>,
}

impl PortMap {
    fn get_connections(&self, port: Option<&str>) -> &[Connection] {
        self.connections
            .get(&port.map(String::from))
            .map(|conns| conns.as_slice())
            .unwrap_or(&[])
    }
}
