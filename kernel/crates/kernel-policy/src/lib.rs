#![no_std]

//! System-wide Policy Engine
//! 
//! Enforcement of security, resource quotas, scheduling, network, FS policies

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod policy;
pub mod evaluator;
pub mod enforcement;

use policy::Policy;
use evaluator::PolicyEvaluator;
use enforcement::PolicyEnforcer;

/// Policy audit entry
#[cfg(feature = "alloc")]
struct PolicyAuditEntry {
    timestamp: u64,
    policy_id: u64,
    action: alloc::string::String,
    context: PolicyContext,
    decision: PolicyDecision,
}

/// Policy engine
pub struct PolicyEngine {
    policies: spin::Mutex<alloc::collections::BTreeMap<u64, Policy>>,
    evaluator: spin::Mutex<PolicyEvaluator>,
    enforcer: spin::Mutex<PolicyEnforcer>,
    #[cfg(feature = "alloc")]
    audit_trail: spin::Mutex<alloc::vec::Vec<PolicyAuditEntry>>,
    #[cfg(not(feature = "alloc"))]
    audit_trail: spin::Mutex<()>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: spin::Mutex::new(alloc::collections::BTreeMap::new()),
            evaluator: spin::Mutex::new(PolicyEvaluator::new()),
            enforcer: spin::Mutex::new(PolicyEnforcer::new()),
            #[cfg(feature = "alloc")]
            audit_trail: spin::Mutex::new(alloc::vec::Vec::new()),
            #[cfg(not(feature = "alloc"))]
            audit_trail: spin::Mutex::new(()),
        }
    }

    /// Add policy
    pub fn add_policy(&self, mut policy: Policy) -> u64 {
        // Time dependency removed to break circular dependency
        
        // Generate unique policy ID using timestamp and counter
        static COUNTER: spin::Mutex<u64> = spin::Mutex::new(1);
        let mut counter = COUNTER.lock();
        let policy_id = *counter;
        *counter += 1;

        // Set policy metadata
        policy.policy_id = policy_id;
        policy.version = 1;
        policy.enabled = true;
        policy.created_at = 0; // Timestamp should be set by caller
        policy.updated_at = 0; // Timestamp should be set by caller

        let mut policies = self.policies.lock();
        policies.insert(policy_id, policy);

        policy_id
    }

    /// Update policy (creates new version)
    pub fn update_policy(&self, policy_id: u64, mut updated_policy: Policy) -> Result<u32, PolicyError> {
        // Time dependency removed to break circular dependency
        
        let mut policies = self.policies.lock();
        if let Some(existing_policy) = policies.get(&policy_id) {
            // Create new version
            updated_policy.policy_id = policy_id;
            updated_policy.version = existing_policy.version + 1;
            updated_policy.enabled = existing_policy.enabled;
            updated_policy.created_at = existing_policy.created_at;
            updated_policy.updated_at = 0; // Timestamp should be set by caller
            
            policies.insert(policy_id, updated_policy);
            Ok(policies.get(&policy_id).unwrap().version)
        } else {
            Err(PolicyError::PolicyNotFound)
        }
    }

    /// Enable/disable policy
    pub fn set_policy_enabled(&self, policy_id: u64, enabled: bool) -> Result<(), PolicyError> {
        let mut policies = self.policies.lock();
        if let Some(policy) = policies.get_mut(&policy_id) {
            policy.enabled = enabled;
            Ok(())
        } else {
            Err(PolicyError::PolicyNotFound)
        }
    }

    /// Get policy version
    pub fn get_policy_version(&self, policy_id: u64) -> Option<u32> {
        let policies = self.policies.lock();
        policies.get(&policy_id).map(|p| p.version)
    }

    /// Evaluate policy
    pub fn evaluate(&self, context: &PolicyContext) -> PolicyDecision {
        // Time dependency removed to break circular dependency
        
        let policies = self.policies.lock();
        // Filter to only enabled policies
        let enabled_policies: alloc::collections::BTreeMap<u64, Policy> = policies
            .iter()
            .filter(|(_, p)| p.enabled)
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        
        let evaluator = self.evaluator.lock();
        let decision = evaluator.evaluate(&enabled_policies, context);
        
        // Audit policy evaluation
        #[cfg(feature = "alloc")]
        {
            let mut audit = self.audit_trail.lock();
            audit.push(PolicyAuditEntry {
                timestamp: 0, // Timestamp should be set by caller
                policy_id: 0, // Will be set if policy matched
                action: context.operation.clone(),
                context: PolicyContext {
                    agent_id: context.agent_id,
                    operation: context.operation.clone(),
                    resource: context.resource.clone(),
                    metadata: context.metadata.clone(),
                },
                decision: PolicyDecision {
                    allow: decision.allow,
                    reason: decision.reason.clone(),
                    constraints: decision.constraints.clone(),
                },
            });
            
            // Keep only last 10000 audit entries
            if audit.len() > 10000 {
                audit.remove(0);
            }
        }
        
        decision
    }

    /// Enforce policy decision
    pub fn enforce(&self, decision: &PolicyDecision) -> bool {
        let mut enforcer = self.enforcer.lock();
        enforcer.enforce(decision)
    }

    /// Get policy audit trail
    #[cfg(feature = "alloc")]
    pub fn get_audit_trail(&self, limit: usize) -> alloc::vec::Vec<&PolicyAuditEntry> {
        let audit = self.audit_trail.lock();
        let start = if audit.len() > limit {
            audit.len() - limit
        } else {
            0
        };
        audit[start..].iter().collect()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_audit_trail(&self, _limit: usize) -> alloc::vec::Vec<&PolicyAuditEntry> {
        Vec::new()
    }
}

/// Policy context
pub struct PolicyContext {
    pub agent_id: u64,
    pub operation: alloc::string::String,
    pub resource: Option<alloc::string::String>,
    pub metadata: alloc::vec::Vec<u8>,
}

/// Policy decision
pub struct PolicyDecision {
    pub allow: bool,
    pub reason: alloc::string::String,
    pub constraints: alloc::vec::Vec<PolicyConstraint>,
}

/// Policy error
#[derive(Debug, Clone, Copy)]
pub enum PolicyError {
    PolicyNotFound,
    InvalidPolicy,
    EvaluationFailed,
}

/// Policy constraint
pub struct PolicyConstraint {
    pub constraint_type: alloc::string::String,
    pub value: alloc::vec::Vec<u8>,
}

/// Global policy engine instance
static POLICY_ENGINE: spin::Once<PolicyEngine> = spin::Once::new();

impl PolicyEngine {
    /// Initialize global policy engine
    pub fn init() {
        POLICY_ENGINE.call_once(|| PolicyEngine::new());
    }

    /// Get global policy engine instance
    pub fn get() -> Option<&'static PolicyEngine> {
        POLICY_ENGINE.get()
    }
}

/// Check policy for operation (helper function)
#[cfg(feature = "alloc")]
pub fn check_policy(
    agent_id: u64,
    operation: &str,
    resource: Option<&str>,
) -> bool {
    if let Some(engine) = PolicyEngine::get() {
        let context = PolicyContext {
            agent_id,
            operation: alloc::string::String::from(operation),
            resource: resource.map(|s| alloc::string::String::from(s)),
            metadata: alloc::vec::Vec::new(),
        };
        let decision = engine.evaluate(&context);
        if decision.allow {
            engine.enforce(&decision);
            true
        } else {
            false
        }
    } else {
        // If policy engine not initialized, default to allow
        true
    }
}

