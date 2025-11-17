//! Capability revocation

use spin::Mutex;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeSet;

/// Revocation manager
/// 
/// Tracks revoked capability tokens
pub struct RevocationManager {
    revoked: Mutex<BTreeSet<u64>>,
}

impl RevocationManager {
    pub fn new() -> Self {
        Self {
            revoked: Mutex::new(BTreeSet::new()),
        }
    }

    /// Revoke capability token
    pub fn revoke(&self, token_id: u64) {
        let mut revoked = self.revoked.lock();
        revoked.insert(token_id);
    }

    /// Check if token is revoked
    pub fn is_revoked(&self, token_id: u64) -> bool {
        let revoked = self.revoked.lock();
        revoked.contains(&token_id)
    }

    /// Unrevoke capability token (for testing/recovery)
    pub fn unrevoke(&self, token_id: u64) {
        let mut revoked = self.revoked.lock();
        revoked.remove(&token_id);
    }
}

