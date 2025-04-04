use crate::simulator::event::{Event, EventPayload};

pub struct ResourceDeliveryProtocol;

impl ResourceDeliveryProtocol {
    /// Create a resource delivery attempt from sender to receiver
    pub fn send_resource(
        time: f64,
        sender_id: &str,
        sender_port: Option<&str>,
        receiver_id: &str,
        receiver_port: Option<&str>,
        amount: f64,
    ) -> Event {
        Event::new(sender_id, receiver_id, time, EventPayload::Resource(amount))
            .with_source_port(sender_port.unwrap_or_default())
            .with_target_port(receiver_port.unwrap_or_default())
    }

    /// Acknowledge that a resource was accepted, sent by receiver to sender
    pub fn accept_resource(
        time: f64,
        receiver_id: &str,
        receiver_port: Option<&str>,
        sender_id: &str,
        sender_port: Option<&str>,
        amount: f64,
    ) -> Event {
        Event::new(
            receiver_id,
            sender_id,
            time,
            EventPayload::ResourceAccepted(amount),
        )
        .with_source_port(receiver_port.unwrap_or_default())
        .with_target_port(sender_port.unwrap_or_default())
    }

    /// Inform the sender that the resource was rejected or only partially accepted
    pub fn reject_resource(
        time: f64,
        receiver_id: &str,
        receiver_port: Option<&str>,
        sender_id: &str,
        sender_port: Option<&str>,
        amount: f64,
    ) -> Event {
        Event::new(
            receiver_id,
            sender_id,
            time,
            EventPayload::ResourceRejected(amount),
        )
        .with_source_port(receiver_port.unwrap_or_default())
        .with_target_port(sender_port.unwrap_or_default())
    }

    /// Utility to emit either accept or reject based on amounts
    pub fn respond_to_delivery(
        time: f64,
        receiver_id: &str,
        receiver_port: Option<&str>,
        sender_id: &str,
        sender_port: Option<&str>,
        accepted: f64,
        rejected: f64,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        if accepted > 0.0 {
            events.push(Self::accept_resource(
                time,
                receiver_id,
                receiver_port,
                sender_id,
                sender_port,
                accepted,
            ));
        }
        if rejected > 0.0 {
            events.push(Self::reject_resource(
                time,
                receiver_id,
                receiver_port,
                sender_id,
                sender_port,
                rejected,
            ));
        }
        events
    }
}
