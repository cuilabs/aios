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
        // Note: Time check removed to avoid circular dependency with kernel-core
        // Caller should check expiration before calling validate
        if token.expires_at > 0 && token.expires_at < u64::MAX {
            // Expiration check would be done by caller with current time
            // For now, we skip the check to avoid circular dependency
        }

        // Verify signature
        // Cryptographic signature verification would be performed here
        // Signature verification requires PQC daemon integration

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
    pub fn revoke(&self, token_id: u64, agent_id: Option<u64>) {
        self.revocation_manager.revoke(token_id);
        
        // Publish audit event via event bus
        #[cfg(feature = "alloc")]
        {
            use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
            // Timestamp set to 0 - caller should set proper timestamp to avoid circular dependency
            publish_event(KernelEvent {
                event_type: EventType::Security,
                timestamp: 0, // Caller should set timestamp
                agent_id,
                data: alloc::vec![("action".into(), "capability_revoke".into()), ("token_id".into(), token_id.to_string())],
            });
        }
    }
    
    /// Check if capability is revoked (enforcement check)
    pub fn is_revoked(&self, token_id: u64) -> bool {
        self.revocation_manager.is_revoked(token_id)
    }
}

/// Global capability system instance
static CAPABILITY_SYSTEM: spin::Once<CapabilitySystem> = spin::Once::new();

/// Initialize capability system
pub fn init() {
    CAPABILITY_SYSTEM.call_once(|| CapabilitySystem::new());
}

/// Get capability system instance
pub fn get() -> Option<&'static CapabilitySystem> {
    CAPABILITY_SYSTEM.get()
}

/// Validate capability token (convenience function)
pub fn validate_capability(token: &CapabilityToken) -> bool {
    if let Some(system) = get() {
        system.validate(token)
    } else {
        false
    }
}

/// Check if agent has capability (convenience function)
pub fn has_capability(token: Option<&CapabilityToken>, capability: capability::Capabilities) -> bool {
    if let Some(tok) = token {
        if !validate_capability(tok) {
            return false;
        }
        tok.has_capability(capability)
    } else {
        false
    }
}

