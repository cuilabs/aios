#![no_std]

//! Kernel Event Bus
//! 
//! System-wide event bus for kernel events:
//! - Interrupt events
//! - Device events
//! - Agent lifecycle events
//! - FS/NET/VM events
//! - Security violations
//! - Performance alerts

extern crate alloc;

pub mod event;
pub mod bus;
pub mod subscription;

use event::KernelEvent;
use bus::EventBus;
use subscription::EventSubscription;

/// Global kernel event system instance
static EVENT_SYSTEM: spin::Once<KernelEventSystem> = spin::Once::new();

/// Kernel event system
pub struct KernelEventSystem {
    bus: spin::Mutex<EventBus>,
}

impl KernelEventSystem {
    pub fn new() -> Self {
        Self {
            bus: spin::Mutex::new(EventBus::new()),
        }
    }

    /// Initialize global event system
    pub fn init() {
        EVENT_SYSTEM.call_once(|| KernelEventSystem::new());
    }

    /// Get global event system instance
    pub fn get() -> Option<&'static KernelEventSystem> {
        EVENT_SYSTEM.get()
    }

    /// Publish event
    pub fn publish(&self, event: KernelEvent) {
        let mut bus = self.bus.lock();
        bus.publish(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self, subscription: EventSubscription) -> u64 {
        let mut bus = self.bus.lock();
        bus.subscribe(subscription)
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, subscription_id: u64) {
        let mut bus = self.bus.lock();
        bus.unsubscribe(subscription_id);
    }
}

/// Publish event to global event system
pub fn publish_event(event: KernelEvent) {
    if let Some(system) = KernelEventSystem::get() {
        system.publish(event);
    }
}

/// Subscribe to events (convenience function)
pub fn subscribe(event_types: alloc::vec::Vec<EventType>, agent_id_filter: Option<u64>, callback: fn(&KernelEvent)) -> u64 {
    if let Some(system) = KernelEventSystem::get() {
        let subscription = EventSubscription::new(event_types, agent_id_filter, callback);
        system.subscribe(subscription)
    } else {
        0
    }
}

