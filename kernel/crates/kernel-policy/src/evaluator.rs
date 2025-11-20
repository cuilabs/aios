//! Policy evaluator

use super::policy::Policy;
use super::{PolicyContext, PolicyDecision, PolicyConstraint};
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Policy evaluator
pub struct PolicyEvaluator;

impl PolicyEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate policies against context
    pub fn evaluate(
        &self,
        policies: &BTreeMap<u64, Policy>,
        context: &PolicyContext,
    ) -> PolicyDecision {
        // Sort policies by priority
        let mut sorted_policies: Vec<&Policy> = policies.values().collect();
        sorted_policies.sort_by_key(|p| p.priority);

        // Evaluate each policy
        for policy in sorted_policies {
            for rule in &policy.rules {
                if self.matches_condition(&rule.condition, context) {
                    return self.execute_action(&rule.action, context);
                }
            }
        }

        // Default: deny
        PolicyDecision {
            allow: false,
            reason: String::from("No matching policy"),
            constraints: Vec::new(),
        }
    }

    /// Check if condition matches context
    fn matches_condition(&self, condition: &super::policy::PolicyCondition, context: &PolicyContext) -> bool {
        // Match based on field and operator
        match condition.field.as_str() {
            "agent_id" => {
                let condition_value = u64::from_le_bytes([
                    condition.value.get(0).copied().unwrap_or(0),
                    condition.value.get(1).copied().unwrap_or(0),
                    condition.value.get(2).copied().unwrap_or(0),
                    condition.value.get(3).copied().unwrap_or(0),
                    condition.value.get(4).copied().unwrap_or(0),
                    condition.value.get(5).copied().unwrap_or(0),
                    condition.value.get(6).copied().unwrap_or(0),
                    condition.value.get(7).copied().unwrap_or(0),
                ]);
                self.compare_values(context.agent_id, condition_value, &condition.operator)
            }
            "operation" => {
                let condition_op = String::from_utf8_lossy(&condition.value);
                self.compare_strings(&context.operation, &condition_op, &condition.operator)
            }
            "resource" => {
                if let Some(ref resource) = context.resource {
                    let condition_resource = String::from_utf8_lossy(&condition.value);
                    self.compare_strings(resource, &condition_resource, &condition.operator)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Compare numeric values
    fn compare_values(&self, value: u64, condition_value: u64, operator: &str) -> bool {
        match operator {
            "eq" => value == condition_value,
            "ne" => value != condition_value,
            "gt" => value > condition_value,
            "lt" => value < condition_value,
            "ge" => value >= condition_value,
            "le" => value <= condition_value,
            _ => false,
        }
    }

    /// Compare string values
    fn compare_strings(&self, value: &str, condition_value: &str, operator: &str) -> bool {
        match operator {
            "eq" => value == condition_value,
            "ne" => value != condition_value,
            "contains" => value.contains(condition_value),
            "starts_with" => value.starts_with(condition_value),
            "ends_with" => value.ends_with(condition_value),
            _ => false,
        }
    }

    /// Execute policy action
    fn execute_action(
        &self,
        action: &super::policy::PolicyAction,
        context: &PolicyContext,
    ) -> PolicyDecision {
        match action.action_type.as_str() {
            "allow" => PolicyDecision {
                allow: true,
                reason: String::from("Policy allows"),
                constraints: Vec::new(),
            },
            "deny" => PolicyDecision {
                allow: false,
                reason: String::from("Policy denies"),
                constraints: Vec::new(),
            },
            "limit" => PolicyDecision {
                allow: true,
                reason: String::from("Policy limits"),
                constraints: vec![PolicyConstraint {
                    constraint_type: String::from("limit"),
                    value: action.parameters.clone(),
                }],
            },
            _ => PolicyDecision {
                allow: false,
                reason: String::from("Unknown action"),
                constraints: Vec::new(),
            },
        }
    }
}

