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

