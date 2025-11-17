#![no_std]

//! Audit & Attestation Manager
//! 
//! Record all agent/system critical actions, support TPM attestation

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod audit;
pub mod attestation;
pub mod log;

use audit::AuditManager;
use attestation::AttestationManager;
use log::AuditLog;

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

    /// Record audit event
    pub fn record_event(&self, event: audit::AuditEvent) {
        let mut audit = self.audit.lock();
        audit.record(event);

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

