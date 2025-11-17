//! Event subscription

use super::event::EventType;

/// Event subscription
/// 
/// Defines what events a subscriber wants to receive
pub struct EventSubscription {
    pub event_types: alloc::vec::Vec<EventType>,
    pub agent_id_filter: Option<u64>,
    pub callback: fn(&super::event::KernelEvent),
}

impl EventSubscription {
    pub fn new(
        event_types: alloc::vec::Vec<EventType>,
        agent_id_filter: Option<u64>,
        callback: fn(&super::event::KernelEvent),
    ) -> Self {
        Self {
            event_types,
            agent_id_filter,
            callback,
        }
    }
}

