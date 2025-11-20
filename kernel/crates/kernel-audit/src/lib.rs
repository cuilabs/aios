#![no_std]

//! Audit & Attestation Manager
//! 
//! Record all agent/system critical actions, support TPM attestation

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod audit;
pub mod attestation;
pub mod log;

use audit::{AuditManager, AuditEvent, AuditEventType, AuditResult};
use attestation::AttestationManager;
use log::AuditLog;

/// Global audit and attestation system instance
static AUDIT_SYSTEM: spin::Once<AuditAttestationSystem> = spin::Once::new();

/// Audit and attestation system
pub struct AuditAttestationSystem {
    audit: spin::Mutex<AuditManager>,
    attestation: spin::Mutex<AttestationManager>,
    log: spin::Mutex<AuditLog>,
}

impl AuditAttestationSystem {
    pub fn new() -> Self {
        Self {
            audit: spin::Mutex::new(AuditManager::new()),
            attestation: spin::Mutex::new(AttestationManager::new()),
            log: spin::Mutex::new(AuditLog::new()),
        }
    }

    /// Initialize global audit system
    pub fn init() {
        AUDIT_SYSTEM.call_once(|| {
            let system = AuditAttestationSystem::new();
            // Subscribe to kernel events
            #[cfg(feature = "alloc")]
            {
                system.subscribe_to_events();
            }
            system
        });
    }

    /// Get global audit system instance
    pub fn get() -> Option<&'static AuditAttestationSystem> {
        AUDIT_SYSTEM.get()
    }

    /// Subscribe to kernel events
    #[cfg(feature = "alloc")]
    fn subscribe_to_events(&self) {
        use aios_kernel_events::{KernelEventSystem, event::EventType, subscription::EventSubscription};
        use aios_kernel_core::time;
        
        if let Some(event_system) = KernelEventSystem::get() {
            // Subscribe to security events
            let security_sub = EventSubscription {
                event_types: alloc::vec![EventType::Security],
                agent_id_filter: None,
                callback: |event: &aios_kernel_events::event::KernelEvent| {
                    if let Some(system) = AuditAttestationSystem::get() {
                        let audit_event = AuditEvent {
                            timestamp: event.timestamp,
                            event_type: AuditEventType::SecurityViolation,
                            agent_id: event.agent_id,
                            action: alloc::string::String::from("Security violation"),
                            result: AuditResult::Denied,
                            metadata: event.data.clone(),
                        };
                        system.record_event(audit_event);
                    }
                },
            };
            event_system.subscribe(security_sub);

            // Subscribe to agent lifecycle events
            let lifecycle_sub = EventSubscription {
                event_types: alloc::vec![EventType::AgentLifecycle],
                agent_id_filter: None,
                callback: |event: &aios_kernel_events::event::KernelEvent| {
                    if let Some(system) = AuditAttestationSystem::get() {
                        let audit_event = AuditEvent {
                            timestamp: event.timestamp,
                            event_type: AuditEventType::AgentSpawn, // Will be refined based on event data
                            agent_id: event.agent_id,
                            action: alloc::string::String::from("Agent lifecycle event"),
                            result: AuditResult::Success,
                            metadata: event.data.clone(),
                        };
                        system.record_event(audit_event);
                    }
                },
            };
            event_system.subscribe(lifecycle_sub);
        }
    }

    /// Record audit event
    pub fn record_event(&self, event: AuditEvent) {
        let mut audit = self.audit.lock();
        audit.record(event.clone());

        // Also log
        let mut log = self.log.lock();
        log.append(event);
    }

    /// Generate attestation evidence
    pub fn generate_attestation(&self) -> attestation::AttestationEvidence {
        let mut attestation = self.attestation.lock();
        attestation.generate_evidence()
    }

    /// Verify attestation
    pub fn verify_attestation(&self, evidence: &attestation::AttestationEvidence) -> bool {
        let attestation = self.attestation.lock();
        attestation.verify(evidence)
    }
}

/// Record audit event (helper function)
#[cfg(feature = "alloc")]
pub fn record_audit_event(
    event_type: AuditEventType,
    agent_id: Option<u64>,
    action: &str,
    result: AuditResult,
    metadata: alloc::vec::Vec<u8>,
) {
    if let Some(system) = AuditAttestationSystem::get() {
        use aios_kernel_core::time;
        let event = AuditEvent {
            timestamp: time::now(),
            event_type,
            agent_id,
            action: alloc::string::String::from(action),
            result,
            metadata,
        };
        system.record_event(event);
    }
}

