use crate::model::ProcessContext;
use crate::simulator::event::{Event, EventPayload};
use crate::utils::errors::SimulationError;

/// Priority levels for different event types
pub const STEP_PRIORITY: u8 = 0;
pub const PULL_PRIORITY: u8 = 1;
pub const RESOURCE_PRIORITY: u8 = 2;
pub const RESOURCE_ACK_PRIORITY: u8 = 3;
pub const OTHER_PRIORITY: u8 = 4;

/// Get the priority level for a given event payload
pub fn get_event_priority(payload: &EventPayload) -> u8 {
    match payload {
        EventPayload::Step => STEP_PRIORITY,
        EventPayload::PullRequest | EventPayload::PullAllRequest => PULL_PRIORITY,
        EventPayload::Resource(_) => RESOURCE_PRIORITY,
        EventPayload::ResourceAccepted(_) | EventPayload::ResourceRejected(_) => {
            RESOURCE_ACK_PRIORITY
        }
        _ => OTHER_PRIORITY,
    }
}

/// Process a batch of events with priority ordering
pub fn process_events_with_priority<F>(
    events: &[Event],
    context: &ProcessContext,
    mut process_event: F,
) -> Result<Vec<Event>, SimulationError>
where
    F: FnMut(&Event, &ProcessContext) -> Result<Vec<Event>, SimulationError>,
{
    // Group events by priority using shared priority function
    let mut events_by_priority: Vec<Vec<Event>> = vec![vec![]; 5];
    for event in events {
        let priority = get_event_priority(&event.payload);
        events_by_priority[priority as usize].push(event.clone());
    }

    // Sort events within each priority group by timestamp
    for events in &mut events_by_priority {
        events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    let mut new_events = Vec::new();

    // First pass: Process only Step events (priority 0)
    if !events_by_priority[0].is_empty() {
        for event in &events_by_priority[0] {
            new_events.extend(process_event(event, context)?);
        }
        // Return both new events and all non-Step events for the next pass
        for priority in 1..5 {
            new_events.extend(events_by_priority[priority].iter().cloned());
        }
        return Ok(new_events);
    }

    // Second pass: Process all other events in priority and time order
    for priority in 1..5 {
        for event in &events_by_priority[priority] {
            new_events.extend(process_event(event, context)?);
        }
    }

    Ok(new_events)
}
