//! Policy API

use crate::kernel::KernelClient;

/// Policy client
pub struct PolicyClient {
    kernel: KernelClient,
}

impl PolicyClient {
    /// Check policy
    pub fn check(&self, agent_id: u64, operation: &str) -> Result<bool, PolicyError> {
        // TODO: Check policy
        Ok(true)
    }
}

/// Policy error
#[derive(Debug)]
pub enum PolicyError {
    PolicyNotFound,
    EvaluationFailed,
}

