use serde::Deserialize;
use serde::Serialize;
use std::collections::{BinaryHeap, HashMap};
use tracing::instrument;
use tracing::{debug, error};

use super::simulation_context::SimulationContext;
use super::simulation_state::SimulationState;
use super::simulation_trait::Simulate;
use super::simulation_trait::StatefulSimulation;
use super::Event;
use super::EventPayload;
use crate::analysis::utils::visualise_resource_transfers;
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
    event_sequence_number: u64,
    connection_sequence_number: u64,
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

    pub fn processes(&self) -> &HashMap<String, Process> {
        &self.processes
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

    pub fn update_process(&mut self, id: &str, process: Process) -> Result<(), SimulationError> {
        self.processes
            .insert(id.to_string(), process);
        Ok(())
    }

    pub fn remove_process(&mut self, id: &str) -> Result<Process, SimulationError> {
        self.processes
            .remove(id)
            .ok_or_else(|| SimulationError::ProcessNotFound(id.to_string()))
    }

    pub fn get_process(&self, id: &str) -> Result<&Process, SimulationError> {
        self.processes
            .get(id)
            .ok_or_else(|| SimulationError::ProcessNotFound(id.to_string()))
    }

    fn validate_connection(&self, connection: &Connection) -> Result<(), SimulationError> {
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

        Ok(())
    }

    fn add_connection_to_io_maps(&mut self, connection: Connection) -> Result<(), SimulationError> {
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
            .push(connection.clone());

        Ok(())
    }

    pub fn add_connection(&mut self, mut connection: Connection) -> Result<(), SimulationError> {
        self.validate_connection(&connection)?;

        // Set sequence number for connection ordering
        connection.sequence_number = self.connection_sequence_number;
        self.connection_sequence_number += 1;

        // Add connection to input and output maps
        self.add_connection_to_io_maps(connection)?;

        Ok(())
    }

    pub fn add_connections(&mut self, connections: Vec<Connection>) -> Result<(), SimulationError> {
        for connection in connections {
            self.add_connection(connection)?;
        }
        Ok(())
    }

    pub fn update_connection(&mut self, connection_id: &str, mut connection: Connection) -> Result<(), SimulationError> {
        self.validate_connection(&connection)?;

        // Set sequence number to the same as the existing connection
        let existing_connection = self.get_connection(connection_id)?;
        connection.sequence_number = existing_connection.sequence_number;

        // Remove old connection from input and output maps
        self.remove_connection(connection_id)?;

        // Add new connection to input and output maps
        self.add_connection_to_io_maps(connection)?;

        Ok(())
    }

    pub fn remove_connection(&mut self, connection_id: &str) -> Result<(), SimulationError> {
        let key = connection_id.to_string();

        fn remove_from_map(
            map: &mut HashMap<String, HashMap<Option<String>, Vec<Connection>>>,
            key: &str,
        ) -> bool {
            let mut found = false;
            for port_map in map.values_mut() {
                for connections in port_map.values_mut() {
                    if let Some(index) = connections.iter().position(|con| con.id == key) {
                        connections.remove(index);
                        found = true;
                    }
                }
            }
            found
        }

        let found_in_input = remove_from_map(&mut self.context.input_map, &key);
        let found_in_output = remove_from_map(&mut self.context.output_map, &key);

        if found_in_input || found_in_output {
            Ok(())
        } else {
            Err(SimulationError::ConnectionNotFound(
                connection_id.to_string(),
            ))
        }
    }

    pub fn get_connection(&self, connection_id: &str) -> Result<&Connection, SimulationError> {
        // Check input and output maps for connection
        if let Some(connections) = self.context.input_map.get(connection_id) {
            if let Some(connection) = connections.values().flatten().find(|con| con.id == connection_id) {
                return Ok(connection);
            }
        }

        if let Some(connections) = self.context.output_map.get(connection_id) {
            if let Some(connection) = connections.values().flatten().find(|con| con.id == connection_id) {
                return Ok(connection);
            }
        }

        Err(SimulationError::ConnectionNotFound(connection_id.to_string()))
    }

    /// Collects all events that occur at the same time as the given event
    fn collect_simultaneous_events(&mut self, first_event: Event) -> Vec<Event> {
        let mut events = vec![first_event];
        let target_time = events[0].time;

        // Collect all events at the same time
        while let Some(next_event) = self.event_queue.peek() {
            if (next_event.time - target_time).abs() > f64::EPSILON {
                break;
            }
            events.push(self.event_queue.pop().unwrap());
        }

        events
    }

    /// Groups events by target process while maintaining sequence order
    fn group_events_by_target(&self, events: Vec<Event>) -> HashMap<String, Vec<Event>> {
        let mut grouped_events: HashMap<String, Vec<Event>> = HashMap::new();

        for event in events {
            grouped_events
                .entry(event.target_id.clone())
                .or_default()
                .push(event);
        }

        grouped_events
    }

    /// Process a batch of events at the same time and return any new events generated
    fn process_event_batch(&mut self, events: Vec<Event>) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();
        let grouped_events = self.group_events_by_target(events.clone());

        for (target_id, target_events) in grouped_events {
            let events = if target_id == "broadcast" {
                target_events
                    .iter()
                    .map(|event| self.process_broadcast_event(event))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect()
            } else {
                let target_process = self
                    .processes
                    .get_mut(&target_id)
                    .ok_or_else(|| SimulationError::ProcessNotFound(target_id.clone()))?;

                let context = self.context.context_for_process(&target_id);
                target_process.on_events(&target_events, &context)?
            };

            self.schedule_events(events)?;
            processed_events.extend(target_events);
        }

        Ok(processed_events)
    }

    /// Validates that an event's source, target, and ports match the simulation's connections
    fn validate_event(&self, event: &Event) -> Result<(), SimulationError> {
        // We assume that broadcast and simulation events are always valid
        if event.target_id == "broadcast" || event.source_id == "simulation" {
            return Ok(());
        }

        // Validate target process exists
        if let Some(process) = self.processes.get(&event.target_id) {
            // Validate target port
            if let Some(port) = &event.target_port {
                let valid_ports = process.get_input_ports();
                if !valid_ports.contains(port) {
                    return Err(SimulationError::InvalidPort {
                        process: event.target_id.clone(),
                        port: port.clone(),
                        port_type: "input".to_string(),
                    });
                }
            }
        } else {
            return Err(SimulationError::ProcessNotFound(event.target_id.clone()));
        }

        // Validate source process exists
        if let Some(process) = self.processes.get(&event.source_id) {
            // Validate source port
            if let Some(port) = &event.source_port {
                let valid_ports = process.get_output_ports();
                if !valid_ports.contains(port) {
                    return Err(SimulationError::InvalidPort {
                        process: event.source_id.clone(),
                        port: port.clone(),
                        port_type: "output".to_string(),
                    });
                }
            }
        } else {
            return Err(SimulationError::ProcessNotFound(event.source_id.clone()));
        }

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
            event_sequence_number: 0,
            connection_sequence_number: 0,
        };

        simulation.add_processes(processes)?;
        simulation.add_connections(connections)?;

        Ok(simulation)
    }

    fn next(&mut self) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();

        // Pre-simulation: capture initial state and broadcast SimulationStart
        if self.context.current_step() == 0 {
            let start_event = Event::new(
                "simulation",
                "broadcast",
                self.context.current_time(),
                EventPayload::SimulationStart,
            );

            let new_events = self.process_broadcast_event(&start_event)?;
            self.schedule_events(new_events)?;
        }

        // Process next event if available
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

            self.schedule_events(new_events)?;
            processed_events.push(next_event);
        }

        // If queue is empty, broadcast SimulationEnd
        if self.event_queue.is_empty() {
            let end_event = Event::new(
                "simulation",
                "broadcast",
                self.context.current_time(),
                EventPayload::SimulationEnd,
            );
            // Throw away new events at simulation end
            let _ = self.process_broadcast_event(&end_event)?;
        }

        Ok(processed_events)
    }

    #[instrument(skip_all, fields(step = %self.current_step(), time = %self.current_time()))]
    fn step(&mut self) -> Result<Vec<Event>, SimulationError> {
        let mut processed_events = Vec::new();

        // Pre-simulation: broadcast SimulationStart
        if self.context.current_step() == 0 {
            let start_event = Event::new(
                "simulation",
                "broadcast",
                self.context.current_time(),
                EventPayload::SimulationStart,
            );

            let new_events = self.process_broadcast_event(&start_event)?;
            self.schedule_events(new_events)?;
        }

        // If no events in queue, send SimulationEnd and return
        if self.event_queue.is_empty() {
            let end_event = Event::new(
                "simulation",
                "broadcast",
                self.context.current_time(),
                EventPayload::SimulationEnd,
            );
            self.process_broadcast_event(&end_event)?;
            return Ok(processed_events);
        }

        // Get next event and update time
        let next_event = self.event_queue.pop().unwrap();
        if (next_event.time - self.context.current_time()).abs() > f64::EPSILON {
            self.context.increment_current_step();
            self.context.set_current_time(next_event.time);
        }

        // Process all events at the current timestep
        let mut events_to_process = self.collect_simultaneous_events(next_event);
        while !events_to_process.is_empty() {
            processed_events.extend(self.process_event_batch(events_to_process)?);

            // Check for new events at the current time
            if let Some(event) = self.event_queue.peek() {
                let event_time = event.time;
                if (event_time - self.context.current_time()).abs() > f64::EPSILON {
                    break;
                }
                let next_event = self.event_queue.pop().unwrap();
                events_to_process = self.collect_simultaneous_events(next_event);
            } else {
                break;
            }
        }
        
        debug!(
            "Step {} completed with {} events processed",
            self.current_step(),
            processed_events.len()
        );
        debug!("\n{}", visualise_resource_transfers(&processed_events));

        // If queue is now empty after processing, send SimulationEnd
        if self.event_queue.is_empty() {
            let end_event = Event::new(
                "simulation",
                "broadcast",
                self.context.current_time(),
                EventPayload::SimulationEnd,
            );
            self.process_broadcast_event(&end_event)?;
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

    #[instrument(skip_all, fields(payload = ?event.payload, source = event.source_id, target = event.target_id, time = event.time, sequence_number = self.event_sequence_number + 1))]
    fn schedule_event(&mut self, mut event: Event) -> Result<(), SimulationError> {
        self.validate_event(&event)?;
        event.sequence_number = self.event_sequence_number;
        self.event_sequence_number += 1;
        self.event_queue.push(event);
        Ok(())
    }

    fn schedule_events(&mut self, events: Vec<Event>) -> Result<(), SimulationError> {
        for event in events {
            self.schedule_event(event)?;
        }
        Ok(())
    }

    fn process_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError> {
        let target_process = self
            .processes
            .get_mut(&event.target_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(event.target_id.clone()))?;

        let context = self.context.context_for_process(target_process.id());
        let new_events = target_process.on_events(&vec![event.clone()], &context)?;

        Ok(new_events)
    }

    fn process_broadcast_event(&mut self, event: &Event) -> Result<Vec<Event>, SimulationError> {
        let mut new_events = Vec::new();
        for (id, process) in self.processes.iter_mut() {
            let context = self.context.context_for_process(id);
            new_events.extend(process.on_events(&vec![event.clone()], &context)?);
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

            self.schedule_events(new_events)?;
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

    fn get_process_state(&self, process_id: &str) -> Result<ProcessState, SimulationError> {
        self.processes
            .get(process_id)
            .ok_or_else(|| SimulationError::ProcessNotFound(process_id.to_string()))
            .map(|p| p.get_state())
    }

    fn reset(&mut self) -> Result<(), SimulationError> {
        for process in self.processes.values_mut() {
            process.reset();
        }

        self.context.reset();
        self.event_queue.clear();

        Ok(())
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

#[cfg(test)]
mod simulation_unit_tests {
    use crate::model::nodes::{Pool, Source};

    use super::*;

    #[test]
    fn test_remove_connection() -> Result<(), SimulationError> {
        let mut simulation = Simulation::new(vec![], vec![]).unwrap();

        let source = Process::new(Box::new(Source::new("source")));
        let target = Process::new(Box::new(Pool::new("target")));

        simulation.add_process(source)?;
        simulation.add_process(target)?;

        let connection_id = "1";
        let connection = Connection::new(
            connection_id.to_string(),
            "source".to_string(),
            Some("out".to_string()),
            "target".to_string(),
            Some("in".to_string()),
            Some(1.0),
        );

        simulation.add_connection(connection).unwrap();
        simulation.remove_connection(&connection_id)?;

        Ok(())
    }
}
