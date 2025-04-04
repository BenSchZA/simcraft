use crate::{
    model::{Connection, Process, ProcessState},
    utils::SimulationError,
};

use super::{
    simulation::{EventIterator, StepIterator},
    simulation_state::SimulationState,
    Event,
};

pub trait Simulate {
    fn new(processes: Vec<Process>, connections: Vec<Connection>) -> Result<Self, SimulationError>
    where
        Self: Sized;
    fn next(&mut self) -> Result<Vec<Event>, SimulationError>;
    fn step(&mut self) -> Result<Vec<Event>, SimulationError>;
    fn step_until(&mut self, until: f64) -> Result<Vec<Event>, SimulationError>;
    fn step_n(&mut self, n: usize) -> Result<Vec<Event>, SimulationError>;
    fn by_event(&mut self) -> EventIterator<'_>;
    fn by_step(&mut self) -> StepIterator<'_>;
    fn schedule_event(&mut self, event: Event) -> Result<(), SimulationError>;
    fn schedule_events(&mut self, event: Vec<Event>) -> Result<(), SimulationError>;
    fn process_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError>;
    fn process_broadcast_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError>;
    fn process_events_at(&mut self, time: f64) -> Result<Vec<Event>, SimulationError>;
}

pub trait StatefulSimulation {
    fn get_simulation_state(&self) -> SimulationState;
    fn get_process_state(&self, process_id: &str) -> Result<ProcessState, SimulationError>;
    fn reset(&mut self) -> Result<(), SimulationError>;
}
