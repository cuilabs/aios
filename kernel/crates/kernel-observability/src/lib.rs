#![no_std]

//! Performance Counters / Observability Hooks
//! 
//! CPU, memory, network, FS, scheduler metrics; trace agent behavior

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod counters;
pub mod metrics;
pub mod tracing;

use counters::PerformanceCounters;
use metrics::MetricsCollector;
use tracing::TracingSystem;

/// Observability system
pub struct ObservabilitySystem {
    counters: spin::Mutex<PerformanceCounters>,
    metrics: spin::Mutex<MetricsCollector>,
    tracing: spin::Mutex<TracingSystem>,
}

impl ObservabilitySystem {
    pub fn new() -> Self {
        Self {
            counters: spin::Mutex::new(PerformanceCounters::new()),
            metrics: spin::Mutex::new(MetricsCollector::new()),
            tracing: spin::Mutex::new(TracingSystem::new()),
        }
    }

    /// Get performance counters for agent
    pub fn get_agent_counters(&self, agent_id: u64) -> Option<AgentCounters> {
        let counters = self.counters.lock();
        counters.get_agent_counters(agent_id)
    }

    /// Collect metrics
    pub fn collect_metrics(&self) -> SystemMetrics {
        let metrics = self.metrics.lock();
        metrics.collect()
    }

    /// Start tracing
    pub fn start_trace(&self, agent_id: u64) {
        let mut tracing = self.tracing.lock();
        tracing.start_trace(agent_id);
    }

    /// Stop tracing
    pub fn stop_trace(&self, agent_id: u64) -> TraceData {
        let mut tracing = self.tracing.lock();
        tracing.stop_trace(agent_id)
    }
}

/// Agent performance counters
pub struct AgentCounters {
    pub cpu_cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub memory_allocated: u64,
    pub memory_freed: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub io_operations: u64,
}

/// System metrics
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_throughput: f64,
    pub io_throughput: f64,
    pub active_agents: u32,
}

/// Trace data
pub struct TraceData {
    pub agent_id: u64,
    pub events: alloc::vec::Vec<TraceEvent>,
}

/// Trace event
pub struct TraceEvent {
    pub timestamp: u64,
    pub event_type: alloc::string::String,
    pub data: alloc::vec::Vec<u8>,
}

