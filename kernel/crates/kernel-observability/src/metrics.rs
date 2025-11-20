//! Metrics collection

use super::SystemMetrics;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Metrics snapshot
#[cfg(feature = "alloc")]
struct MetricsSnapshot {
    timestamp: u64,
    cpu_usage: f64,
    memory_usage: f64,
    network_throughput: f64,
    io_throughput: f64,
    active_agents: u32,
}

/// Metrics collector
pub struct MetricsCollector {
    #[cfg(feature = "alloc")]
    snapshots: Mutex<Vec<MetricsSnapshot>>,
    #[cfg(feature = "alloc")]
    current_metrics: Mutex<SystemMetrics>,
    #[cfg(feature = "alloc")]
    agent_metrics: Mutex<BTreeMap<u64, AgentMetrics>>,
    #[cfg(not(feature = "alloc"))]
    snapshots: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    current_metrics: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    agent_metrics: Mutex<()>,
}

/// Agent-specific metrics
#[cfg(feature = "alloc")]
struct AgentMetrics {
    cpu_time: u64,
    memory_usage: u64,
    network_bytes: u64,
    io_operations: u64,
    last_update: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            snapshots: Mutex::new(Vec::new()),
            #[cfg(feature = "alloc")]
            current_metrics: Mutex::new(SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_throughput: 0.0,
                io_throughput: 0.0,
                active_agents: 0,
            }),
            #[cfg(feature = "alloc")]
            agent_metrics: Mutex::new(BTreeMap::new()),
            #[cfg(not(feature = "alloc"))]
            snapshots: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            current_metrics: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            agent_metrics: Mutex::new(()),
        }
    }

    /// Collect system metrics
    pub fn collect(&self) -> SystemMetrics {
        #[cfg(feature = "alloc")]
        {
            use aios_kernel_hal::timer;
            
            let mut current = self.current_metrics.lock();
            let agent_metrics = self.agent_metrics.lock();
            
            // Calculate CPU usage from agent metrics
            let total_cpu_time: u64 = agent_metrics.values().map(|m| m.cpu_time).sum();
            current.cpu_usage = (total_cpu_time as f64) / 1_000_000_000.0; // Convert to seconds
            
            // Calculate memory usage
            let total_memory: u64 = agent_metrics.values().map(|m| m.memory_usage).sum();
            current.memory_usage = total_memory as f64;
            
            // Calculate network throughput
            let total_network: u64 = agent_metrics.values().map(|m| m.network_bytes).sum();
            current.network_throughput = total_network as f64;
            
            // Calculate I/O throughput
            let total_io: u64 = agent_metrics.values().map(|m| m.io_operations).sum();
            current.io_throughput = total_io as f64;
            
            // Count active agents
            current.active_agents = agent_metrics.len() as u32;
            
            // Save snapshot
            let mut snapshots = self.snapshots.lock();
            snapshots.push(MetricsSnapshot {
                timestamp: timer::now_nanos() / 1_000_000_000, // Convert to seconds
                cpu_usage: current.cpu_usage,
                memory_usage: current.memory_usage,
                network_throughput: current.network_throughput,
                io_throughput: current.io_throughput,
                active_agents: current.active_agents,
            });
            
            // Keep only last 1000 snapshots
            if snapshots.len() > 1000 {
                snapshots.remove(0);
            }
            
            *current
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_throughput: 0.0,
                io_throughput: 0.0,
                active_agents: 0,
            }
        }
    }

    /// Update agent metrics
    #[cfg(feature = "alloc")]
    pub fn update_agent_metrics(&self, agent_id: u64, cpu_time: u64, memory: u64, network: u64, io: u64) {
        use aios_kernel_core::time;
        
        let mut agent_metrics = self.agent_metrics.lock();
        let metrics = agent_metrics.entry(agent_id).or_insert_with(|| AgentMetrics {
            cpu_time: 0,
            memory_usage: 0,
            network_bytes: 0,
            io_operations: 0,
            last_update: timer::now_nanos() / 1_000_000_000, // Convert to seconds
        });
        
        metrics.cpu_time += cpu_time;
        metrics.memory_usage = memory;
        metrics.network_bytes += network;
        metrics.io_operations += io;
        metrics.last_update = time::now();
    }

    #[cfg(not(feature = "alloc"))]
    pub fn update_agent_metrics(&self, _agent_id: u64, _cpu_time: u64, _memory: u64, _network: u64, _io: u64) {
        // No-op
    }

    /// Get metrics history
    #[cfg(feature = "alloc")]
    pub fn get_history(&self, limit: usize) -> Vec<SystemMetrics> {
        let snapshots = self.snapshots.lock();
        let start = if snapshots.len() > limit {
            snapshots.len() - limit
        } else {
            0
        };
        
        snapshots[start..]
            .iter()
            .map(|s| SystemMetrics {
                cpu_usage: s.cpu_usage,
                memory_usage: s.memory_usage,
                network_throughput: s.network_throughput,
                io_throughput: s.io_throughput,
                active_agents: s.active_agents,
            })
            .collect()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_history(&self, _limit: usize) -> Vec<SystemMetrics> {
        Vec::new()
    }
}

