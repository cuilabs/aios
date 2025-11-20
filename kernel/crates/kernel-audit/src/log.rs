//! Immutable audit log

use super::audit::AuditEvent;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use spin::Mutex;

/// Immutable audit log
pub struct AuditLog {
    entries: Mutex<Vec<AuditLogEntry>>,
}

struct AuditLogEntry {
    sequence: u64,
    event: AuditEvent,
    hash: Vec<u8>, // Hash of previous entry + this entry
    signature: Vec<u8>, // Cryptographic signature
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Append entry (append-only)
    pub fn append(&mut self, event: AuditEvent) {
        let mut entries = self.entries.lock();
        let sequence = entries.len() as u64;

        // Calculate hash (previous hash + event)
        let previous_hash = entries.last().map(|e| e.hash.clone()).unwrap_or_default();
        let hash = self.calculate_hash(&previous_hash, &event);

        // Sign entry
        let signature = self.sign(&event, &hash);

        entries.push(AuditLogEntry {
            sequence,
            event,
            hash,
            signature,
        });
    }

    /// Verify log integrity
    pub fn verify(&self) -> bool {
        let entries = self.entries.lock();
        let mut previous_hash = Vec::new();

        for entry in entries.iter() {
            // Verify hash chain
            let expected_hash = self.calculate_hash(&previous_hash, &entry.event);
            if expected_hash != entry.hash {
                return false;
            }

            // Verify signature
            if !self.verify_signature(&entry.event, &entry.hash, &entry.signature) {
                return false;
            }

            previous_hash = entry.hash.clone();
        }

        true
    }

    /// Calculate hash (SHA-256)
    fn calculate_hash(&self, previous: &[u8], event: &AuditEvent) -> Vec<u8> {
        use aios_kernel_crypto::hash;
        
        // Combine previous hash + event data for chain
        let mut data = Vec::new();
        data.extend_from_slice(previous);
        data.extend_from_slice(&event.timestamp.to_le_bytes());
        data.push(event.event_type as u8);
        if let Some(agent_id) = event.agent_id {
            data.extend_from_slice(&agent_id.to_le_bytes());
        }
        data.extend_from_slice(event.action.as_bytes());
        data.push(event.result as u8);
        data.extend_from_slice(&event.metadata);
        
        // Calculate SHA-256 hash
        hash::sha256(&data)
    }

    /// Sign entry (using kernel crypto)
    fn sign(&self, _event: &AuditEvent, hash: &[u8]) -> Vec<u8> {
        use aios_kernel_crypto::signature;
        
        // Sign hash with kernel signing key
        // Use TPM or hardware key for signing
        signature::sign(hash).unwrap_or_default()
    }

    /// Verify signature
    fn verify_signature(&self, _event: &AuditEvent, hash: &[u8], signature: &[u8]) -> bool {
        use aios_kernel_crypto::signature;
        
        // Verify signature
        signature::verify(hash, signature).unwrap_or(false)
    }
}

