//! Kernel event types

use alloc::vec::Vec;

/// Event type
#[repr(u32)]
pub enum EventType {
    Interrupt = 1,
    Device = 2,
    AgentLifecycle = 3,
    FileSystem = 4,
    Network = 5,
    VirtualMemory = 6,
    Security = 7,
    Performance = 8,
}

/// Kernel event
pub struct KernelEvent {
    pub event_type: EventType,
    pub timestamp: u64,
    pub agent_id: Option<u64>,
    pub data: Vec<u8>,
}

/// Agent lifecycle events
#[repr(u32)]
pub enum AgentLifecycleEvent {
    Spawned = 1,
    Killed = 2,
    Cloned = 3,
    Merged = 4,
    Split = 5,
    Upgraded = 6,
    Specialized = 7,
}

/// Security violation events
#[repr(u32)]
pub enum SecurityEvent {
    CapabilityViolation = 1,
    QuotaExceeded = 2,
    UnauthorizedAccess = 3,
    BehavioralAnomaly = 4,
}

