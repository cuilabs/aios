//! Event bus implementation

use super::event::KernelEvent;
use super::subscription::EventSubscription;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Event bus
/// 
/// Routes events to subscribers
pub struct EventBus {
    subscriptions: Mutex<BTreeMap<u64, EventSubscription>>,
    next_id: Mutex<u64>,
    event_queue: Mutex<Vec<KernelEvent>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(BTreeMap::new()),
            next_id: Mutex::new(1),
            event_queue: Mutex::new(Vec::new()),
        }
    }

    /// Publish event
    pub fn publish(&mut self, event: KernelEvent) {
        // Add to queue
        let mut queue = self.event_queue.lock();
        queue.push(event.clone());

        // Route to subscribers
        let subscriptions = self.subscriptions.lock();
        for subscription in subscriptions.values() {
            // Check if subscription matches event type
            let matches_type = subscription.event_types.iter().any(|&et| {
                core::mem::discriminant(&et) == core::mem::discriminant(&event.event_type)
            });
            
            if matches_type {
                // Check agent filter
                if let Some(filter_agent_id) = subscription.agent_id_filter {
                    if event.agent_id != Some(filter_agent_id) {
                        continue;
                    }
                }
                
                // Call callback
                (subscription.callback)(&event);
            }
        }
    }

    /// Subscribe to events
    pub fn subscribe(&mut self, subscription: EventSubscription) -> u64 {
        let mut next_id = self.next_id.lock();
        let id = *next_id;
        *next_id = next_id.wrapping_add(1);

        let mut subscriptions = self.subscriptions.lock();
        subscriptions.insert(id, subscription);

        id
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&mut self, subscription_id: u64) {
        let mut subscriptions = self.subscriptions.lock();
        subscriptions.remove(&subscription_id);
    }
}

