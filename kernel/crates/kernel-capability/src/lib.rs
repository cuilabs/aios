#![no_std]

//! Kernel Capability Model
//! 
//! Uniform capability model across all kernel subsystems:
//! - Per-agent quotas
//! - Per-agent capability tokens
//! - Per-agent resource groups
//! - Capability revocation
//! - Isolation

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod capability;
pub mod quota;
pub mod resource_group;
pub mod revocation;

use capability::CapabilityToken;
use quota::QuotaManager;
use resource_group::ResourceGroup;
use revocation::RevocationManager;

/// Capability system
/// 
/// Provides uniform capability model across all kernel subsystems
pub struct CapabilitySystem {
    quota_manager: QuotaManager,
    revocation_manager: RevocationManager,
    resource_groups: spin::Mutex<alloc::collections::BTreeMap<u64, ResourceGroup>>,
}

impl CapabilitySystem {
    pub fn new() -> Self {
        Self {
            quota_manager: QuotaManager::new(),
            revocation_manager: RevocationManager::new(),
            resource_groups: spin::Mutex::new(alloc::collections::BTreeMap::new()),
        }
    }

    /// Validate capability token
    pub fn validate(&self, token: &CapabilityToken) -> bool {
        // Check revocation
        if self.revocation_manager.is_revoked(token.token_id) {
            return false;
        }

        // Check expiration
        let now = 0; // TODO: Get from kernel time
        if token.expires_at < now {
            return false;
        }

        // Verify signature
        // TODO: Verify cryptographic signature

        true
    }

    /// Check quota for agent
    pub fn check_quota(&self, agent_id: u64, resource: &str, amount: u64) -> bool {
        self.quota_manager.check(agent_id, resource, amount)
    }

    /// Get resource group for agent
    pub fn get_resource_group(&self, agent_id: u64) -> Option<ResourceGroup> {
        let groups = self.resource_groups.lock();
        groups.get(&agent_id).cloned()
    }

    /// Revoke capability
    pub fn revoke(&self, token_id: u64) {
        self.revocation_manager.revoke(token_id);
    }
}

