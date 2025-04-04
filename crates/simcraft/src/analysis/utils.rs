use crate::simulator::event::{Event, EventPayload};
use std::collections::HashMap;

/// Creates a visual representation of resource transfers from a batch of events
pub fn visualise_resource_transfers(events: &[Event]) -> String {
    let mut transfers = String::new();
    transfers.push_str(&format!("Time: {:.2}\n", events[0].time));
    transfers.push_str("Resource Transfers:\n");

    // Create a map of resource transfers and their outcomes
    let mut transfer_outcomes: HashMap<(String, String, String, String, String), (bool, f64)> =
        HashMap::new();

    // Track all resource events to determine outcomes
    for event in events {
        match &event.payload {
            EventPayload::Resource(amt) => {
                let key = (
                    event.source_id.clone(),
                    event.source_port.clone().unwrap_or_default(),
                    event.target_id.clone(),
                    event.target_port.clone().unwrap_or_default(),
                    format!("{:.2}", amt),
                );
                transfer_outcomes.insert(key, (false, *amt));
            }
            EventPayload::ResourceAccepted(amt) => {
                let key = (
                    event.target_id.clone(),
                    event.target_port.clone().unwrap_or_default(),
                    event.source_id.clone(),
                    event.source_port.clone().unwrap_or_default(),
                    format!("{:.2}", amt),
                );
                transfer_outcomes.insert(key, (true, *amt));
            }
            EventPayload::ResourceRejected(amt) => {
                let key = (
                    event.target_id.clone(),
                    event.target_port.clone().unwrap_or_default(),
                    event.source_id.clone(),
                    event.source_port.clone().unwrap_or_default(),
                    format!("{:.2}", amt),
                );
                transfer_outcomes.insert(key, (false, *amt));
            }
            _ => {}
        }
    }

    // Sort transfers by source ID for consistent output
    let mut sorted_transfers: Vec<_> = transfer_outcomes.into_iter().collect();
    sorted_transfers.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

    if sorted_transfers.is_empty() {
        transfers.push_str("No resource transfers in this timestep\n");
        return transfers;
    }

    // Display transfers
    for ((source_id, source_port, target_id, target_port, _), (success, amount)) in sorted_transfers
    {
        let symbol = if success { "→" } else { "✗" };
        transfers.push_str(&format!(
            "{}.{} {} {}.{} ({:.2} units)\n",
            source_id,
            if source_port.is_empty() {
                "*"
            } else {
                &source_port
            },
            symbol,
            target_id,
            if target_port.is_empty() {
                "*"
            } else {
                &target_port
            },
            amount
        ));
    }

    transfers
}
