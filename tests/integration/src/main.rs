//! AIOS Test Runner
//! 
//! Comprehensive test runner for AIOS integration, performance, and chaos tests.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use clap::{Parser, Subcommand};

mod integration;
mod performance;
mod chaos;
mod e2e_workflows;
mod harness;

use integration::IntegrationTestSuite;
use performance::PerformanceTestSuite;
use chaos::ChaosTestSuite;
use e2e_workflows::E2EWorkflowTestSuite;

#[derive(Debug, Parser)]
#[command(name = "aios-test-runner", about = "AIOS comprehensive test runner")]
struct Opt {
    /// Test suite to run
    #[command(subcommand)]
    suite: TestSuite,
    
    /// Output directory for test results
    #[arg(long, default_value = "tests/artifacts")]
    output: PathBuf,
    
    /// Report file path
    #[arg(long)]
    report: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum TestSuite {
    /// Run integration tests
    Integration {
        /// Specific test category to run
        #[arg(long)]
        category: Option<String>,
    },
    /// Run performance benchmarks
    Performance {
        /// Specific benchmark to run
        #[arg(long)]
        benchmark: Option<String>,
    },
    /// Run chaos/resilience tests
    Chaos {
        /// Specific chaos test to run
        #[arg(long)]
        test: Option<String>,
    },
    /// Run end-to-end workflow tests
    E2EWorkflows {
        /// Specific workflow to run
        #[arg(long)]
        workflow: Option<String>,
    },
    /// Run all test suites
    All,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TestStatus {
    Pass,
    Fail(String),
    Skipped(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub timestamp: DateTime<Utc>,
    pub suite: String,
    pub tests: Vec<TestResult>,
    pub summary: TestSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    
    let start_time = Instant::now();
    let report = match opt.suite {
        TestSuite::Integration { category } => {
            let suite = IntegrationTestSuite::new(opt.output.clone());
            suite.run(category.as_deref(), opt.verbose)?
        }
        TestSuite::Performance { benchmark } => {
            let suite = PerformanceTestSuite::new(opt.output.clone());
            suite.run(benchmark.as_deref(), opt.verbose)?
        }
        TestSuite::Chaos { test } => {
            let suite = ChaosTestSuite::new(opt.output.clone());
            suite.run(test.as_deref(), opt.verbose)?
        }
        TestSuite::E2EWorkflows { workflow } => {
            let suite = E2EWorkflowTestSuite::new(opt.output.clone());
            suite.run(workflow.as_deref(), opt.verbose)?
        }
        TestSuite::All => {
            // Run all suites
            let integration_suite = IntegrationTestSuite::new(opt.output.join("integration"));
            let perf_suite = PerformanceTestSuite::new(opt.output.join("performance"));
            let chaos_suite = ChaosTestSuite::new(opt.output.join("chaos"));
            let e2e_suite = E2EWorkflowTestSuite::new(opt.output.join("e2e_workflows"));
            
            let integration_report = integration_suite.run(None, opt.verbose)?;
            let perf_report = perf_suite.run(None, opt.verbose)?;
            let chaos_report = chaos_suite.run(None, opt.verbose)?;
            let e2e_report = e2e_suite.run(None, opt.verbose)?;
            
            // Combine reports
            let mut all_tests = integration_report.tests;
            all_tests.extend(perf_report.tests);
            all_tests.extend(chaos_report.tests);
            all_tests.extend(e2e_report.tests);
            
            let total = all_tests.len();
            let passed = all_tests.iter().filter(|t| matches!(t.status, TestStatus::Pass)).count();
            let failed = all_tests.iter().filter(|t| matches!(t.status, TestStatus::Fail(_))).count();
            let skipped = all_tests.iter().filter(|t| matches!(t.status, TestStatus::Skipped(_))).count();
            
            TestReport {
                timestamp: Utc::now(),
                suite: "all".to_string(),
                tests: all_tests,
                summary: TestSummary {
                    total,
                    passed,
                    failed,
                    skipped,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                },
            }
        }
    };
    
    // Write report
    let report_path = opt.report.unwrap_or_else(|| {
        opt.output.join(format!("{}-report.json", report.suite))
    });
    
    std::fs::create_dir_all(report_path.parent().unwrap())?;
    let report_json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&report_path, report_json)
        .context("Failed to write test report")?;
    
    println!("Test suite '{}' completed:", report.suite);
    println!("  Total: {}", report.summary.total);
    println!("  Passed: {}", report.summary.passed);
    println!("  Failed: {}", report.summary.failed);
    println!("  Skipped: {}", report.summary.skipped);
    println!("  Duration: {}ms", report.summary.duration_ms);
    println!("  Report: {}", report_path.display());
    
    if report.summary.failed > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

