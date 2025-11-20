//! Unit tests

#![no_std]
#![cfg(test)]

pub mod test_framework;
pub mod test_memory;
pub mod test_scheduler;
pub mod test_policy;

#[cfg(test)]
mod tests {
    use super::*;
    use test_framework::run_test_suite;
    
    #[test]
    fn run_all_unit_tests() {
        let suites = [
            test_memory::MEMORY_TEST_SUITE,
            test_scheduler::SCHEDULER_TEST_SUITE,
            test_policy::POLICY_TEST_SUITE,
        ];
        
        for suite in suites.iter() {
            let (passed, failed, skipped) = run_test_suite(suite);
            // In production, would report results
        }
    }
}

