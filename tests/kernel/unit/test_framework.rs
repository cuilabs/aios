//! Kernel Unit Test Framework
//! 
//! Provides testing utilities for kernel subsystems in no_std environment

#![no_std]
#![feature(test)]

#[cfg(test)]
extern crate alloc;

#[cfg(test)]
use alloc::vec::Vec;

/// Test result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    Pass,
    Fail,
    Skip,
}

/// Test case
pub struct TestCase {
    pub name: &'static str,
    pub test_fn: fn() -> TestResult,
}

/// Test suite
pub struct TestSuite {
    pub name: &'static str,
    pub tests: &'static [TestCase],
}

/// Assert macro for tests
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return $crate::TestResult::Fail;
        }
    };
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {
        if $left == $right {
            return $crate::TestResult::Fail;
        }
    };
}

#[macro_export]
macro_rules! assert_true {
    ($condition:expr) => {
        if !$condition {
            return $crate::TestResult::Fail;
        }
    };
}

#[macro_export]
macro_rules! assert_false {
    ($condition:expr) => {
        if $condition {
            return $crate::TestResult::Fail;
        }
    };
}

/// Run test suite
pub fn run_test_suite(suite: &TestSuite) -> (usize, usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    
    for test in suite.tests {
        match (test.test_fn)() {
            TestResult::Pass => passed += 1,
            TestResult::Fail => failed += 1,
            TestResult::Skip => skipped += 1,
        }
    }
    
    (passed, failed, skipped)
}

