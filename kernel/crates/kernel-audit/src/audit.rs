//! Audit manager

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Audit event
#[derive(Clone)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub agent_id: Option<u64>,
    pub action: alloc::string::String,
    pub result: AuditResult,
    pub metadata: Vec<u8>,
}

/// Audit event type
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum AuditEventType {
    AgentSpawn = 1,
    AgentKill = 2,
    CapabilityGrant = 3,
    CapabilityRevoke = 4,
    SecurityViolation = 5,
    ResourceExceeded = 6,
    SystemEvent = 7,
}

/// Audit result
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum AuditResult {
    Success = 1,
    Failure = 2,
    Denied = 3,
}

/// Audit manager
pub struct AuditManager {
    events: alloc::vec::Vec<AuditEvent>,
}

impl AuditManager {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Record audit event
    pub fn record(&mut self, event: AuditEvent) {
        // Append-only log
        self.events.push(event);
    }

    /// Get audit events
    pub fn get_events(&self, filter: Option<AuditFilter>) -> Vec<&AuditEvent> {
        if let Some(filter) = filter {
            self.events
                .iter()
                .filter(|e| {
                    if let Some(agent_id) = filter.agent_id {
                        e.agent_id == Some(agent_id)
                    } else {
                        true
                    }
                })
                .collect()
        } else {
            self.events.iter().collect()
        }
    }
}

/// Audit filter
pub struct AuditFilter {
    pub agent_id: Option<u64>,
    pub event_type: Option<AuditEventType>,
}

