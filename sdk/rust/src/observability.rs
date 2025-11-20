//! Observability API

use crate::kernel::KernelClient;

/// Observability client
pub struct ObservabilityClient {
    kernel: KernelClient,
}

impl ObservabilityClient {
    /// Get metrics
    pub fn metrics(&self) -> Result<SystemMetrics, ObservabilityError> {
        // TODO: Get system metrics
        Ok(SystemMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
        })
    }
}

/// System metrics
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

/// Observability error
#[derive(Debug)]
pub enum ObservabilityError {
    NotAvailable,
}

