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
    pub version: u32,
    pub enabled: bool,
    pub created_at: u64,
    pub updated_at: u64,
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

#[cfg(feature = "alloc")]
impl Clone for Policy {
    fn clone(&self) -> Self {
        Self {
            policy_id: self.policy_id,
            name: self.name.clone(),
            policy_type: self.policy_type,
            rules: self.rules.clone(),
            priority: self.priority,
            version: self.version,
            enabled: self.enabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(feature = "alloc")]
impl Clone for PolicyRule {
    fn clone(&self) -> Self {
        Self {
            condition: PolicyCondition {
                field: self.condition.field.clone(),
                operator: self.condition.operator.clone(),
                value: self.condition.value.clone(),
            },
            action: PolicyAction {
                action_type: self.action.action_type.clone(),
                parameters: self.action.parameters.clone(),
            },
        }
    }
}

