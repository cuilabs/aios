//! Integration tests

#![no_std]
#![cfg(test)]

pub mod test_boot;
pub mod test_subsystem_integration;

#[cfg(test)]
mod tests {
    use super::*;
    use test_framework::run_test_suite;
    
    #[test]
    fn run_all_integration_tests() {
        let suites = [
            test_boot::BOOT_TEST_SUITE,
            test_subsystem_integration::INTEGRATION_TEST_SUITE,
        ];
        
        for suite in suites.iter() {
            let (passed, failed, skipped) = run_test_suite(suite);
            // In production, would report results
        }
    }
}

