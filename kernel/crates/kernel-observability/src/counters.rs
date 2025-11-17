//! Performance counters

use super::AgentCounters;
use spin::Mutex;

#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;
    use alloc::collections::BTreeMap;

    /// Performance counters
    pub struct PerformanceCounters {
        agent_counters: Mutex<BTreeMap<u64, AgentCounters>>,
    }

    impl PerformanceCounters {
        pub fn new() -> Self {
            Self {
                agent_counters: Mutex::new(BTreeMap::new()),
            }
        }

        pub fn get_agent_counters(&self, agent_id: u64) -> Option<AgentCounters> {
            let counters = self.agent_counters.lock();
            counters.get(&agent_id).cloned()
        }

        pub fn update_cpu_cycles(&self, agent_id: u64, cycles: u64) {
            let mut counters = self.agent_counters.lock();
            let counter = counters.entry(agent_id).or_insert_with(|| AgentCounters {
                cpu_cycles: 0,
                instructions: 0,
                cache_misses: 0,
                memory_allocated: 0,
                memory_freed: 0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                io_operations: 0,
            });
            counter.cpu_cycles += cycles;
        }

        pub fn update_memory_allocated(&self, agent_id: u64, bytes: u64) {
            let mut counters = self.agent_counters.lock();
            let counter = counters.entry(agent_id).or_insert_with(|| AgentCounters {
                cpu_cycles: 0,
                instructions: 0,
                cache_misses: 0,
                memory_allocated: 0,
                memory_freed: 0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                io_operations: 0,
            });
            counter.memory_allocated += bytes;
        }

        pub fn update_network_bytes(&self, agent_id: u64, sent: u64, received: u64) {
            let mut counters = self.agent_counters.lock();
            let counter = counters.entry(agent_id).or_insert_with(|| AgentCounters {
                cpu_cycles: 0,
                instructions: 0,
                cache_misses: 0,
                memory_allocated: 0,
                memory_freed: 0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                io_operations: 0,
            });
            counter.network_bytes_sent += sent;
            counter.network_bytes_received += received;
        }
    }
}

#[cfg(feature = "alloc")]
use alloc_impl::PerformanceCounters;

#[cfg(not(feature = "alloc"))]
struct PerformanceCounters;

#[cfg(not(feature = "alloc"))]
impl PerformanceCounters {
    pub fn new() -> Self {
        Self
    }
    pub fn get_agent_counters(&self, _agent_id: u64) -> Option<AgentCounters> {
        None
    }
    pub fn update_cpu_cycles(&self, _agent_id: u64, _cycles: u64) {}
    pub fn update_memory_allocated(&self, _agent_id: u64, _bytes: u64) {}
    pub fn update_network_bytes(&self, _agent_id: u64, _sent: u64, _received: u64) {}
}

