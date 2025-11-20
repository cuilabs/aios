//! Policy engine unit tests

#![no_std]
#![cfg(test)]

extern crate alloc;

use aios_kernel_policy::{PolicyEngine, Policy, PolicyType, PolicyRule, PolicyCondition, PolicyAction};
use test_framework::{TestResult, TestCase, TestSuite};

/// Test policy evaluation
fn test_policy_evaluation() -> TestResult {
    let engine = PolicyEngine::new();
    
    // Create test policy
    let policy = Policy {
        policy_id: 0,
        name: alloc::string::String::from("test_policy"),
        policy_type: PolicyType::Security,
        rules: alloc::vec![PolicyRule {
            condition: PolicyCondition {
                field: alloc::string::String::from("agent_id"),
                operator: alloc::string::String::from("eq"),
                value: 1u64.to_le_bytes().to_vec(),
            },
            action: PolicyAction {
                action_type: alloc::string::String::from("allow"),
                parameters: alloc::vec::Vec::new(),
            },
        }],
        priority: 1,
        version: 1,
        enabled: true,
        created_at: 0,
        updated_at: 0,
    };
    
    let policy_id = engine.add_policy(policy);
    
    // Evaluate policy
    let context = aios_kernel_policy::PolicyContext {
        agent_id: 1,
        operation: alloc::string::String::from("test_operation"),
        resource: None,
        metadata: alloc::vec::Vec::new(),
    };
    
    let decision = engine.evaluate(&context);
    assert_true!(decision.allow);
    
    TestResult::Pass
}

/// Test policy versioning
fn test_policy_versioning() -> TestResult {
    let engine = PolicyEngine::new();
    
    let policy = Policy {
        policy_id: 0,
        name: alloc::string::String::from("test_policy"),
        policy_type: PolicyType::Security,
        rules: alloc::vec![],
        priority: 1,
        version: 1,
        enabled: true,
        created_at: 0,
        updated_at: 0,
    };
    
    let policy_id = engine.add_policy(policy);
    
    // Check initial version
    let version1 = engine.get_policy_version(policy_id);
    assert_eq!(version1, Some(1));
    
    // Update policy
    let updated_policy = Policy {
        policy_id: 0,
        name: alloc::string::String::from("test_policy_v2"),
        policy_type: PolicyType::Security,
        rules: alloc::vec![],
        priority: 1,
        version: 1,
        enabled: true,
        created_at: 0,
        updated_at: 0,
    };
    
    let version2 = engine.update_policy(policy_id, updated_policy);
    assert_true!(version2.is_ok());
    assert_eq!(version2.unwrap(), 2);
    
    TestResult::Pass
}

/// Policy test suite
pub const POLICY_TEST_SUITE: TestSuite = TestSuite {
    name: "Policy Engine",
    tests: &[
        TestCase {
            name: "policy_evaluation",
            test_fn: test_policy_evaluation,
        },
        TestCase {
            name: "policy_versioning",
            test_fn: test_policy_versioning,
        },
    ],
};

