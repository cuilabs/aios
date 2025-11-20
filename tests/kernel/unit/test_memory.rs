//! Memory subsystem unit tests

#![no_std]
#![cfg(test)]

extern crate alloc;

use aios_kernel_core::memory::{allocate_agent_memory, deallocate_agent_memory};
use aios_kernel_core::memory::fabric::MemoryFabricManager;
use test_framework::{TestResult, TestCase, TestSuite};

/// Test agent memory allocation
fn test_agent_memory_allocation() -> TestResult {
    let memory = allocate_agent_memory(1024);
    assert_true!(memory.is_some());
    TestResult::Pass
}

/// Test memory fabric region creation
fn test_memory_fabric_region() -> TestResult {
    if let Some(fabric) = MemoryFabricManager::get() {
        let region_id = fabric.create_region(
            1,
            aios_kernel_core::memory::fabric::MemoryRegionType::Ephemeral,
            4096,
        );
        assert_true!(region_id.is_ok());
        TestResult::Pass
    } else {
        TestResult::Skip
    }
}

/// Test memory fabric versioning
fn test_memory_versioning() -> TestResult {
    if let Some(fabric) = MemoryFabricManager::get() {
        let region_id = fabric.create_region(
            1,
            aios_kernel_core::memory::fabric::MemoryRegionType::Ephemeral,
            4096,
        ).ok();
        
        if let Some(rid) = region_id {
            let version1 = fabric.get_version(rid);
            assert_true!(version1.is_some());
            
            let new_version = fabric.increment_version(rid);
            assert_true!(new_version.is_ok());
            
            let version2 = fabric.get_version(rid);
            assert_eq!(version2, Some(version1.unwrap() + 1));
        }
        
        TestResult::Pass
    } else {
        TestResult::Skip
    }
}

/// Memory test suite
pub const MEMORY_TEST_SUITE: TestSuite = TestSuite {
    name: "Memory Subsystem",
    tests: &[
        TestCase {
            name: "agent_memory_allocation",
            test_fn: test_agent_memory_allocation,
        },
        TestCase {
            name: "memory_fabric_region",
            test_fn: test_memory_fabric_region,
        },
        TestCase {
            name: "memory_versioning",
            test_fn: test_memory_versioning,
        },
    ],
};

