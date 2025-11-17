//! Metrics collection

use super::SystemMetrics;

/// Metrics collector
pub struct MetricsCollector;

impl MetricsCollector {
    pub fn new() -> Self {
        Self
    }

    /// Collect system metrics
    pub fn collect(&self) -> SystemMetrics {
        // TODO: Collect actual metrics
        SystemMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_throughput: 0.0,
            io_throughput: 0.0,
            active_agents: 0,
        }
    }
}

