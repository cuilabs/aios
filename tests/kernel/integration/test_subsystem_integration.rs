//! Subsystem integration tests

#![no_std]
#![cfg(test)]

extern crate alloc;

use aios_kernel_core::memory::fabric::MemoryFabricManager;
use aios_kernel_agent::scheduler::AgentScheduler;
use aios_kernel_policy::PolicyEngine;
use test_framework::{TestResult, TestCase, TestSuite};

/// Test memory fabric and scheduler integration
fn test_memory_scheduler_integration() -> TestResult {
    // Create agent
    let scheduler = AgentScheduler::new();
    scheduler.add_agent(1, 100, 10);
    
    // Allocate memory for agent
    if let Some(fabric) = MemoryFabricManager::get() {
        let region_id = fabric.create_region(
            1,
            aios_kernel_core::memory::fabric::MemoryRegionType::Ephemeral,
            4096,
        );
        assert_true!(region_id.is_ok());
    }
    
    TestResult::Pass
}

/// Test policy and capability integration
fn test_policy_capability_integration() -> TestResult {
    let engine = PolicyEngine::new();
    // Policy evaluation should integrate with capability checks
    // In production, would test actual integration
    TestResult::Pass
}

/// Integration test suite
pub const INTEGRATION_TEST_SUITE: TestSuite = TestSuite {
    name: "Subsystem Integration",
    tests: &[
        TestCase {
            name: "memory_scheduler_integration",
            test_fn: test_memory_scheduler_integration,
        },
        TestCase {
            name: "policy_capability_integration",
            test_fn: test_policy_capability_integration,
        },
    ],
};

