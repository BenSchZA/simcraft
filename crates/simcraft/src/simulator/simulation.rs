use serde::Deserialize;
use serde::Serialize;
use std::collections::{BinaryHeap, HashMap};
use tracing::{debug, error, trace, warn};

use super::simulation_context::SimulationContext;
use super::simulation_state::SimulationResults;
use super::simulation_state::SimulationState;
use super::simulation_trait::Simulate;
use super::simulation_trait::StatefulSimulation;
use super::Event;
use super::EventPayload;
use crate::utils::logging::init_logging_once;
use crate::{
    model::{
        connection::Connection, process_state::ProcessState, process_trait::Processor, Process,
    },
    utils::SimulationError,
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Simulation {
    processes: HashMap<String, Process>,
    context: SimulationContext,
    event_queue: BinaryHeap<Event>,
}

impl Simulation {
    pub fn get_context(&self) -> &SimulationContext {
        &self.context
    }

    pub fn get_events(&self) -> &[Event] {
        self.event_queue.as_slice()
    }

    pub fn current_time(&self) -> f64 {
        self.context.current_time()
    }

    pub fn process_ids(&self) -> Vec<String> {
        self.processes.keys().cloned().collect()
    }

    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    pub fn has_process(&self, id: &str) -> bool {
        self.processes.contains_key(id)
    }

    pub fn add_process<P: Processor + 'static>(
        &mut self,
        processor: P,
    ) -> Result<(), SimulationError> {
        let process = Process::new(Box::new(processor));
        let id = process.id().to_string();

        if self.processes.contains_key(&id) {
            return Err(SimulationError::DuplicateProcess(id));
        }

        self.processes.insert(id, process);
        Ok(())
    }

    pub fn add_processes(&mut self, processes: Vec<Process>) -> Result<(), SimulationError> {
        for process in processes {
            self.add_process(process)?;
        }
        Ok(())
    }

    pub fn remove_process(&mut self, id: &str) -> Result<Process, SimulationError> {
        self.processes
            .remove(id)
            .ok_or_else(|| SimulationError::ProcessNotFound(id.to_string()))
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
            .get(&connection.source_id)
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
            .get(&connection.target_id)
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

        // Add connection to input map
        self.context
            .input_map
            .entry(connection.target_id.clone())
            .or_default()
            .entry(connection.target_port.clone())
            .or_default()
            .push(connection.clone());

        // Add connection to output map
        self.context
            .output_map
            .entry(connection.source_id.clone())
            .or_default()
            .entry(connection.source_port.clone())
            .or_default()
            .push(connection);

        Ok(())
    }
}

impl Simulate for Simulation {
    fn new(processes: Vec<Process>, connections: Vec<Connection>) -> Result<Self, SimulationError> {
        init_logging_once();

        let mut simulation = Self {
            processes: HashMap::new(),
            context: SimulationContext::default(),
            event_queue: BinaryHeap::new(),
        };

        simulation.add_processes(processes)?;
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

    fn next(&mut self) -> Result<SimulationResults, SimulationError> {
        // TODO Go to next event
        unimplemented!();

        let mut results = Vec::new();
        let current_step = self.context.current_step();
        let current_time = self.context.current_time();

        // Capture initial state
        if current_step == 0 {
            results.push(self.get_simulation_state());
        }

        if let Some(event) = self.event_queue.pop() {
            if (event.time - current_time).abs() > f64::EPSILON {
                self.context.increment_current_step();
                self.context.set_current_time(event.time);
            }
            let current_time = self.current_time();
            debug!("Processing event at time {}: {:?}", current_time, event);

            let new_events = if event.target_id == "broadcast" {
                self.process_broadcast_event(&event)?
            } else {
                self.process_event(&event)?
            };

            for event in new_events {
                self.schedule_event(event);
            }

            results.push(self.get_simulation_state());
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

        // Update results with current simulation state
        // TODO Consider letting simulation user handle getting state at appropriate intervals
        // and returning processed events instead
        results.push(self.get_simulation_state());

        // TODO Results should be returned even if no events here!
        // Advance to next event time
        if let Some(next_event) = self.event_queue.peek() {
            self.context.increment_current_step();
            self.context.set_current_time(next_event.time);
        } else {
            let end_event = Event {
                time: current_time,
                source_id: "simulation".to_string(),
                source_port: None,
                target_id: "broadcast".to_string(),
                target_port: None,
                payload: EventPayload::SimulationEnd,
            };
            let _ = self.process_broadcast_event(&end_event)?;
            // Err(SimulationError::NoEvents)
        }

        Ok(results)
    }

    fn step_until(&mut self, until: f64) -> Result<SimulationResults, SimulationError> {
        let mut results = Vec::new();

        while self.context.current_time() <= until {
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
        // Validate target process exists
        if !self.processes.contains_key(&event.target_id) {
            return Err(SimulationError::ProcessNotFound(event.target_id.clone()));
        }

        // Validate port
        if let Some(port) = &event.target_port {
            let valid_ports = self.processes[&event.target_id].get_input_ports();
            if !valid_ports.contains(port) {
                return Err(SimulationError::InvalidPort {
                    process: event.target_id.clone(),
                    port: port.clone(),
                    port_type: "input".to_string(),
                });
            }
        }

        let target_process = self
            .processes
            .get_mut(&event.target_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(event.target_id.clone()))?;

        let context = self.context.context_for_process(target_process.id());

        let new_events = target_process.on_event(event, &context)?;

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

        for (id, process) in self.processes.iter_mut() {
            let context = self.context.context_for_process(id);
            new_events.extend(process.on_event(event, &context)?);
        }

        Ok(new_events)
    }
}

impl StatefulSimulation for Simulation {
    fn get_simulation_state(&self) -> SimulationState {
        let mut process_states = HashMap::new();

        for (id, process) in &self.processes {
            process_states.insert(id.clone(), process.get_state());
        }

        SimulationState {
            step: self.context.current_step(),
            time: self.context.current_time(),
            process_states,
        }
    }

    fn get_process_state(&self, process_id: &str) -> Option<ProcessState> {
        self.processes.get(process_id).map(|p| p.get_state())
    }
}
