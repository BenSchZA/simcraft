use crate::{
    model::{Connection, Process, ProcessState},
    utils::SimulationError,
};

use super::{
    simulation_state::{SimulationResults, SimulationState},
    Event,
};

pub trait Simulate {
    fn new(processes: Vec<Process>, connections: Vec<Connection>) -> Result<Self, SimulationError>
    where
        Self: Sized;
    fn next(&mut self) -> Result<SimulationResults, SimulationError>;
    fn step(&mut self) -> Result<SimulationResults, SimulationError>;
    fn step_until(&mut self, until: f64) -> Result<SimulationResults, SimulationError>;
    fn step_n(&mut self, n: usize) -> Result<SimulationResults, SimulationError>;
    fn schedule_event(&mut self, event: Event);
    fn process_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError>;
    fn process_broadcast_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError>;
}

pub trait StatefulSimulation {
    fn get_simulation_state(&self) -> SimulationState;
    fn get_process_state(&self, process_id: &str) -> Option<ProcessState>;
}
