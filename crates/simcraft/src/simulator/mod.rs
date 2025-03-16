use log::debug;
use log::error;
use log::trace;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{BinaryHeap, HashMap};

use crate::{
    model::{
        connection::Connection, process_state::ProcessState, process_trait::Processor, Process,
    },
    utils::SimulationError,
};

pub mod context;
pub mod event;

pub use context::SimulationContext;
pub use event::Event;
pub use event::EventPayload;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Simulation {
    // TODO Store processes and connections as HashMaps?
    processes: Vec<Process>,
    context: SimulationContext,
    event_queue: BinaryHeap<Event>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub step: usize,
    pub time: f64,
    pub process_states: HashMap<String, ProcessState>,
}

pub type SimulationResults = Vec<SimulationState>;

pub trait Simulate {
    fn new(processes: Vec<Process>, connections: Vec<Connection>) -> Result<Self, SimulationError>
    where
        Self: Sized;
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

impl Simulation {
    pub fn get_events(&self) -> &[Event] {
        self.event_queue.as_slice()
    }

    pub fn get_current_time(&self) -> f64 {
        self.context.current_time()
    }

    pub fn get_process_ids(&self) -> Vec<String> {
        self.processes.iter().map(|p| p.id().to_string()).collect()
    }

    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    pub fn has_process(&self, id: &str) -> bool {
        self.processes.iter().any(|p| p.id() == id)
    }

    pub fn add_process(&mut self, process: Process) -> Result<(), SimulationError> {
        // Check for duplicate IDs
        if self.processes.iter().any(|p| p.id() == process.id()) {
            return Err(SimulationError::DuplicateProcess(process.id().to_string()));
        }

        debug!("Adding process: {}", process.id());
        self.processes.push(process);
        Ok(())
    }

    pub fn add_processes(&mut self, processes: Vec<Process>) -> Result<(), SimulationError> {
        for process in processes {
            self.add_process(process)?;
        }
        Ok(())
    }

    pub fn remove_process(&mut self, id: &str) -> Result<Process, SimulationError> {
        if let Some(pos) = self.processes.iter().position(|p| p.id() == id) {
            Ok(self.processes.remove(pos))
        } else {
            Err(SimulationError::ProcessNotFound(id.to_string()))
        }
    }

    pub fn add_connections(&mut self, connections: Vec<Connection>) -> Result<(), SimulationError> {
        for connection in connections {
            self.add_connection(connection)?;
        }
        Ok(())
    }

    pub fn add_connection(&mut self, connection: Connection) -> Result<(), SimulationError> {
        // Validate source process and port
        let source_process = self
            .processes
            .iter()
            .find(|p| *p.id() == connection.source_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(connection.source_id.clone()))?;

        if let Some(port) = &connection.source_port {
            if !source_process.get_output_ports().contains(port) {
                return Err(SimulationError::InvalidPort {
                    process: connection.source_id.clone(),
                    port: port.clone(),
                    port_type: "output".to_string(),
                });
            }
        }

        // Validate target process and port
        let target_process = self
            .processes
            .iter()
            .find(|p| *p.id() == connection.target_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(connection.target_id.clone()))?;

        if let Some(port) = &connection.target_port {
            if !target_process.get_input_ports().contains(port) {
                return Err(SimulationError::InvalidPort {
                    process: connection.target_id.clone(),
                    port: port.clone(),
                    port_type: "input".to_string(),
                });
            }
        }

        // If validation passes, add the connection
        self.context.add_connection(connection);
        Ok(())
    }
}

impl Simulate for Simulation {
    fn new(processes: Vec<Process>, connections: Vec<Connection>) -> Result<Self, SimulationError> {
        let mut simulation = Self {
            processes,
            context: SimulationContext::default(),
            event_queue: BinaryHeap::new(),
        };

        simulation.add_connections(connections)?;

        // Schedule initial simulation start event
        simulation.schedule_event(Event {
            time: simulation.context.current_time,
            source_id: "simulation".to_string(),
            source_port: None,
            target_id: "broadcast".to_string(),
            target_port: None,
            payload: EventPayload::SimulationStart,
        });

        Ok(simulation)
    }

    fn step(&mut self) -> Result<SimulationResults, SimulationError> {
        let mut results = Vec::new();
        let current_step = self.context.current_step();
        let current_time = self.context.current_time();

        // Capture initial state
        if current_step == 0 {
            results.push(self.get_simulation_state());
        }

        // Process all events at current time
        while let Some(next_event) = self.event_queue.peek() {
            if (next_event.time - current_time).abs() > f64::EPSILON {
                break;
            }

            let event = self.event_queue.pop().unwrap();
            debug!("Processing event at time {}: {:?}", current_time, event);

            let new_events = if event.target_id == "broadcast" {
                self.process_broadcast_event(&event)?
            } else {
                self.process_event(&event)?
            };

            let new_events_len = new_events.len();
            for event in new_events {
                self.schedule_event(event);
            }
            debug!(
                "Scheduled {} new events at time {}",
                new_events_len, current_time
            );
        }

        // TODO Results should be returned even if no events here!
        // Advance to next event time
        if let Some(next_event) = self.event_queue.peek() {
            results.push(self.get_simulation_state());
            self.context.increment_current_step();
            self.context.set_current_time(next_event.time);
            Ok(results)
        } else {
            // No more events, send simulation end event
            self.schedule_event(Event {
                time: current_time,
                source_id: "simulation".to_string(),
                source_port: None,
                target_id: "broadcast".to_string(),
                target_port: None,
                payload: EventPayload::SimulationEnd,
            });
            Err(SimulationError::NoEvents)
        }
    }

    fn step_until(&mut self, until: f64) -> Result<SimulationResults, SimulationError> {
        let mut results = Vec::new();

        while self.context.current_time() < until {
            match self.step() {
                Ok(state_changes) => results.extend(state_changes),
                Err(SimulationError::NoEvents) => {
                    warn!(
                        "No events left in queue, stopping at time {}",
                        self.context.current_time()
                    );
                    return Err(SimulationError::NoEvents);
                }
                Err(e) => {
                    error!("Error during step: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    fn step_n(&mut self, n: usize) -> Result<SimulationResults, SimulationError> {
        let mut results = Vec::new();

        for i in 0..n {
            debug!("Starting step {} of {}", i + 1, n);
            debug!("Current time: {}", self.context.current_time());
            debug!("Event queue size: {}", self.event_queue.len());

            match self.step() {
                Ok(state_changes) => results.extend(state_changes),
                Err(SimulationError::NoEvents) => {
                    warn!("No events left in queue, stopping at step {}", i + 1);
                    return Err(SimulationError::NoEvents);
                }
                Err(e) => {
                    error!("Error during step: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    fn schedule_event(&mut self, event: Event) {
        trace!("Scheduling event: {:?}", event);
        self.event_queue.push(event)
    }

    fn process_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError> {
        // Validate target process and port
        let target_idx = self
            .processes
            .iter()
            .position(|p| p.id() == event.target_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(event.target_id.clone()))?;

        // Validate port
        if let Some(port) = &event.target_port {
            let valid_ports = self.processes[target_idx].get_input_ports();
            if !valid_ports.contains(port) {
                return Err(SimulationError::InvalidPort {
                    process: event.target_id.clone(),
                    port: port.clone(),
                    port_type: "input".to_string(),
                });
            }
        }

        // Process event
        let target_process = self
            .processes
            .iter_mut()
            .find(|p| p.id() == event.target_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(event.target_id.clone()))?;

        let new_events = target_process.on_event(event, &mut self.context)?;

        // Validate new events
        for event in &new_events {
            if let Some(port) = &event.source_port {
                let valid_ports = target_process.get_output_ports();
                if !valid_ports.contains(port) {
                    return Err(SimulationError::InvalidPort {
                        process: event.source_id.clone(),
                        port: port.clone(),
                        port_type: "output".to_string(),
                    });
                }
            }
        }

        Ok(new_events)
    }

    fn process_broadcast_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();
        for process in &mut self.processes {
            new_events.extend(process.on_event(event, &mut self.context)?);
        }
        Ok(new_events)
    }
}

impl StatefulSimulation for Simulation {
    fn get_simulation_state(&self) -> SimulationState {
        let mut process_states = HashMap::new();

        for process in &self.processes {
            process_states.insert(process.id().to_string(), process.get_state());
        }

        SimulationState {
            step: self.context.current_step(),
            time: self.context.current_time(),
            process_states,
        }
    }

    fn get_process_state(&self, process_id: &str) -> Option<ProcessState> {
        self.processes
            .iter()
            .find(|p| p.id() == process_id)
            .map(|p| p.get_state())
    }
}
