//! Scheduler unit tests

#![no_std]
#![cfg(test)]

extern crate alloc;

use aios_kernel_agent::scheduler::AgentScheduler;
use test_framework::{TestResult, TestCase, TestSuite};

/// Test agent scheduling
fn test_agent_scheduling() -> TestResult {
    let scheduler = AgentScheduler::new();
    
    // Add agent
    scheduler.add_agent(1, 100, 10);
    
    // Get next agent
    let next = scheduler.next();
    assert_eq!(next, Some(1));
    
    TestResult::Pass
}

/// Test deadline scheduling
fn test_deadline_scheduling() -> TestResult {
    let scheduler = AgentScheduler::new();
    
    scheduler.add_agent(1, 100, 10);
    scheduler.add_agent(2, 100, 10);
    
    // Set deadline for agent 2
    scheduler.set_deadline(2, 1000);
    
    // Next should prioritize agent with deadline
    let next = scheduler.next();
    assert_eq!(next, Some(2));
    
    TestResult::Pass
}

/// Test priority inheritance
fn test_priority_inheritance() -> TestResult {
    let scheduler = AgentScheduler::new();
    
    scheduler.add_agent(1, 50, 5);  // Lower priority
    scheduler.add_agent(2, 100, 10); // Higher priority
    
    // Inherit priority from agent 2 to agent 1
    scheduler.inherit_priority(1, 2);
    
    // Agent 1 should now have higher effective priority
    let next = scheduler.next();
    assert_eq!(next, Some(1));
    
    // Restore priority
    scheduler.restore_priority(1);
    
    TestResult::Pass
}

/// Scheduler test suite
pub const SCHEDULER_TEST_SUITE: TestSuite = TestSuite {
    name: "Scheduler Subsystem",
    tests: &[
        TestCase {
            name: "agent_scheduling",
            test_fn: test_agent_scheduling,
        },
        TestCase {
            name: "deadline_scheduling",
            test_fn: test_deadline_scheduling,
        },
        TestCase {
            name: "priority_inheritance",
            test_fn: test_priority_inheritance,
        },
    ],
};

