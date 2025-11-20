//! Test Harness
//! 
//! Common test harness utilities for running tests and collecting results.

use crate::TestStatus;
use anyhow::Result;
use chrono::Utc;
use std::time::Instant;

pub struct TestHarness {
    verbose: bool,
}

impl TestHarness {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
    
    pub fn run_test<F>(&self, name: &str, test_fn: F) -> Result<crate::TestResult>
    where
        F: FnOnce() -> Result<()>,
    {
        if self.verbose {
            println!("Running test: {}", name);
        }
        
        let start = Instant::now();
        let timestamp = Utc::now();
        
        let (status, details) = match test_fn() {
            Ok(()) => {
                if self.verbose {
                    println!("  ✓ Passed");
                }
                (TestStatus::Pass, None)
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if self.verbose {
                    println!("  ✗ Failed: {}", error_msg);
                }
                (TestStatus::Fail(error_msg), None)
            }
        };
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        Ok(crate::TestResult {
            name: name.to_string(),
            status,
            duration_ms,
            details,
            timestamp,
        })
    }
    
    pub fn skip_test(&self, name: &str, reason: &str) -> crate::TestResult {
        if self.verbose {
            println!("Skipping test: {} ({})", name, reason);
        }
        
        crate::TestResult {
            name: name.to_string(),
            status: TestStatus::Skipped(reason.to_string()),
            duration_ms: 0,
            details: None,
            timestamp: Utc::now(),
        }
    }
}

