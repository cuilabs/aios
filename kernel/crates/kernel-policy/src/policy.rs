//! Policy definition

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Policy
pub struct Policy {
    pub policy_id: u64,
    pub name: String,
    pub policy_type: PolicyType,
    pub rules: Vec<PolicyRule>,
    pub priority: u32,
}

/// Policy type
#[repr(u32)]
pub enum PolicyType {
    Security = 1,
    ResourceQuota = 2,
    Scheduling = 3,
    Network = 4,
    FileSystem = 5,
}

/// Policy rule
pub struct PolicyRule {
    pub condition: PolicyCondition,
    pub action: PolicyAction,
}

/// Policy condition
pub struct PolicyCondition {
    pub field: String,
    pub operator: String, // "eq", "gt", "lt", "contains", etc.
    pub value: Vec<u8>,
}

/// Policy action
pub struct PolicyAction {
    pub action_type: String, // "allow", "deny", "limit", "audit"
    pub parameters: Vec<u8>,
}

