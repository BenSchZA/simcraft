use crate::model::ProcessContext;
use crate::simulator::event::{Event, EventPayload};
use crate::utils::errors::SimulationError;

/// Process a batch of events with priority ordering
pub fn process_events_with_priority<F>(
    events: &[Event],
    context: &ProcessContext,
    mut process_event: F,
) -> Result<Vec<Event>, SimulationError>
where
    F: FnMut(&Event, &ProcessContext) -> Result<Vec<Event>, SimulationError>,
{
    let mut new_events = Vec::new();

    // Group events by type
    let mut step_events = Vec::new();
    let mut trigger_events = Vec::new();
    let mut pull_request_events = Vec::new();
    let mut pull_all_request_events = Vec::new();
    let mut resource_events = Vec::new();
    let mut other_events = Vec::new();

    // Sort events into groups
    for event in events {
        match event.payload {
            EventPayload::Step => step_events.push(event),
            EventPayload::Trigger => trigger_events.push(event),
            EventPayload::PullRequest => pull_request_events.push(event),
            EventPayload::PullAllRequest => pull_all_request_events.push(event),
            EventPayload::Resource(_) => resource_events.push(event),
            _ => other_events.push(event),
        }
    }

    // Sort pull request events by connection sequence number
    pull_request_events.sort_by_key(|event| {
        context
            .inputs_for_port(Some("in"))
            .find(|conn| conn.source_id == event.source_id)
            .map(|conn| conn.sequence_number)
            .unwrap_or(u64::MAX)
    });

    // Sort pull all request events by connection sequence number
    pull_all_request_events.sort_by_key(|event| {
        context
            .inputs_for_port(Some("in"))
            .find(|conn| conn.source_id == event.source_id)
            .map(|conn| conn.sequence_number)
            .unwrap_or(u64::MAX)
    });

    // Sort resource events by connection sequence number
    resource_events.sort_by_key(|event| {
        context
            .inputs_for_port(Some("in"))
            .find(|conn| conn.source_id == event.source_id)
            .map(|conn| conn.sequence_number)
            .unwrap_or(u64::MAX)
    });

    // Process events in priority order
    for event in step_events
        .into_iter()
        .chain(trigger_events)
        .chain(pull_request_events)
        .chain(pull_all_request_events)
        .chain(resource_events)
        .chain(other_events)
    {
        new_events.extend(process_event(event, context)?);
    }

    Ok(new_events)
}
