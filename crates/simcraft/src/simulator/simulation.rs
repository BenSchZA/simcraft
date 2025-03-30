use serde::Deserialize;
use serde::Serialize;
use std::collections::{BinaryHeap, HashMap};
use tracing::instrument;
use tracing::{debug, error, trace};

use super::simulation_context::SimulationContext;
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

    pub fn current_step(&self) -> u64 {
        self.context.current_step()
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
        // Process connections are stored in SimulationContext input/output maps
        simulation.add_connections(connections)?;

        Ok(simulation)
    }

    fn next(&mut self) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();

        // Pre-simulation: capture initial state and broadcast SimulationStart
        if self.context.current_step() == 0 {
            let start_event = Event {
                time: self.context.current_time(),
                source_id: "simulation".to_string(),
                source_port: None,
                target_id: "broadcast".to_string(),
                target_port: None,
                payload: EventPayload::SimulationStart,
            };

            let new_events = self.process_broadcast_event(&start_event)?;
            self.schedule_events(new_events);
        }

        // Advance to next event if available
        if let Some(next_event) = self.event_queue.pop() {
            // If next event time is greater than current time, increment step
            if (next_event.time - self.context.current_time()).abs() > f64::EPSILON {
                self.context.increment_current_step();
            }
            self.context.set_current_time(next_event.time);

            let new_events = if next_event.target_id == "broadcast" {
                self.process_broadcast_event(&next_event)?
            } else {
                self.process_event(&next_event)?
            };

            self.schedule_events(new_events);
            processed_events.push(next_event);
        }

        // If queue is empty, broadcast SimulationEnd
        if self.event_queue.is_empty() {
            let end_event = Event {
                time: self.context.current_time(),
                source_id: "simulation".to_string(),
                source_port: None,
                target_id: "broadcast".to_string(),
                target_port: None,
                payload: EventPayload::SimulationEnd,
            };
            // Throw away new events at simulation end
            let _ = self.process_broadcast_event(&end_event)?;
        }

        Ok(processed_events)
    }

    #[instrument(skip_all, fields(step = %self.current_step(), time = %self.current_time()))]
    fn step(&mut self) -> Result<Vec<Event>, SimulationError> {
        // Pre-simulation: broadcast SimulationStart
        if self.context.current_step() == 0 {
            let start_event = Event {
                time: self.context.current_time(),
                source_id: "simulation".to_string(),
                source_port: None,
                target_id: "broadcast".to_string(),
                target_port: None,
                payload: EventPayload::SimulationStart,
            };

            let new_events = self.process_broadcast_event(&start_event)?;
            self.schedule_events(new_events);
        }

        // Increment step and time to next event
        if let Some(next_event) = self.event_queue.peek() {
            self.context.increment_current_step();
            self.context.set_current_time(next_event.time);
        }

        // Process all events at current time
        let current_time = self.context.current_time();
        let processed_events = self.process_events_at(current_time)?;

        // If queue is empty, broadcast SimulationEnd
        if self.event_queue.is_empty() {
            let end_event = Event {
                time: current_time,
                source_id: "simulation".to_string(),
                source_port: None,
                target_id: "broadcast".to_string(),
                target_port: None,
                payload: EventPayload::SimulationEnd,
            };
            // Throw away new events at simulation end
            let _ = self.process_broadcast_event(&end_event)?;
        }

        Ok(processed_events)
    }

    fn step_until(&mut self, until: f64) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();

        // TODO Enable user-defined tolerance and use instead of f64::EPSILON
        while self.context.current_time() < until + f64::EPSILON {
            match self.step() {
                Ok(events) => processed_events.extend(events),
                Err(e) => {
                    error!("Error during step: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(processed_events)
    }

    fn step_n(&mut self, n: usize) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();

        for _ in 0..n {
            match self.step() {
                Ok(events) => processed_events.extend(events),
                Err(e) => {
                    error!("Error during step: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(processed_events)
    }

    fn by_event(&mut self) -> EventIterator<'_> {
        EventIterator { sim: self }
    }

    fn by_step(&mut self) -> StepIterator<'_> {
        StepIterator { sim: self }
    }

    fn schedule_event(&mut self, event: Event) {
        trace!("Scheduling event: {:?}", event);
        self.event_queue.push(event);
    }

    fn schedule_events(&mut self, events: Vec<Event>) {
        for event in events {
            self.schedule_event(event);
        }
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

    /// Processes all events scheduled at the given time and schedules any resulting events.
    fn process_events_at(&mut self, time: f64) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();

        while let Some(event) = self.event_queue.peek() {
            if (event.time - time).abs() > f64::EPSILON {
                break;
            }

            let event = self.event_queue.pop().unwrap();
            debug!("Processing event at time {}: {:?}", time, event);

            let new_events = if event.target_id == "broadcast" {
                self.process_broadcast_event(&event)?
            } else {
                self.process_event(&event)?
            };

            self.schedule_events(new_events);
            processed_events.push(event);
        }

        Ok(processed_events)
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

pub struct EventIterator<'a> {
    sim: &'a mut Simulation,
}

pub struct StepIterator<'a> {
    sim: &'a mut Simulation,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sim.event_queue.is_empty() {
            return None;
        }

        // Only pop one event and process it
        if let Ok(mut events) = self.sim.next() {
            if !events.is_empty() {
                return Some(events.remove(0)); // Return the first processed event
            }
        }

        self.next() // Try again until we hit an actual event
    }
}

impl<'a> Iterator for StepIterator<'a> {
    type Item = Vec<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sim.event_queue.is_empty() {
            return None;
        }

        match self.sim.step() {
            Ok(events) if !events.is_empty() => Some(events),
            Ok(_) => self.next(), // skip empty steps
            Err(_) => None,
        }
    }
}
