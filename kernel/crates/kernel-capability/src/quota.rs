//! Per-agent quota management

use spin::Mutex;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};

/// Quota manager
/// 
/// Enforces per-agent quotas across all subsystems
pub struct QuotaManager {
    #[cfg(feature = "alloc")]
    quotas: Mutex<BTreeMap<u64, AgentQuotas>>,
    #[cfg(feature = "alloc")]
    statistics: Mutex<BTreeMap<u64, QuotaStatistics>>, // Agent ID -> Statistics
    #[cfg(not(feature = "alloc"))]
    quotas: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    statistics: Mutex<()>,
}

/// Quota statistics
#[cfg(feature = "alloc")]
struct QuotaStatistics {
    total_checks: u64,
    total_allocations: u64,
    total_releases: u64,
    quota_exceeded_count: u64,
    last_exceeded: u64,
}

struct AgentQuotas {
    memory: Quota,
    cpu: Quota,
    network: Quota,
    io: Quota,
    fs: Quota,
}

struct Quota {
    limit: u64,
    used: u64,
}

impl QuotaManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            quotas: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            statistics: Mutex::new(BTreeMap::new()),
            #[cfg(not(feature = "alloc"))]
            quotas: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            statistics: Mutex::new(()),
        }
    }

    /// Check if agent can use resource
    pub fn check(&self, agent_id: u64, resource: &str, amount: u64) -> bool {
        #[cfg(feature = "alloc")]
        {
            // Update statistics
            let mut stats = self.statistics.lock();
            let stat = stats.entry(agent_id).or_insert_with(|| QuotaStatistics {
                total_checks: 0,
                total_allocations: 0,
                total_releases: 0,
                quota_exceeded_count: 0,
                last_exceeded: 0,
            });
            stat.total_checks += 1;

            let mut quotas = self.quotas.lock();
            let agent_quotas = quotas.entry(agent_id).or_insert_with(|| AgentQuotas::default());

            let result = match resource {
                "memory" => agent_quotas.memory.check(amount),
                "cpu" => agent_quotas.cpu.check(amount),
                "network" => agent_quotas.network.check(amount),
                "io" => agent_quotas.io.check(amount),
                "fs" => agent_quotas.fs.check(amount),
                _ => false,
            };

            if !result {
                stat.quota_exceeded_count += 1;
                stat.last_exceeded = 0; // Timestamp should be set by caller to avoid circular dependency
                
                // Publish quota exceeded event
                publish_event(KernelEvent {
                    event_type: EventType::Security,
                    timestamp: 0, // Timestamp should be set by caller
                    agent_id: Some(agent_id),
                    data: Vec::new(),
                });
            }

            result
        }

        #[cfg(not(feature = "alloc"))]
        {
            true
        }
    }

    /// Allocate resource
    pub fn allocate(&self, agent_id: u64, resource: &str, amount: u64) -> bool {
        if !self.check(agent_id, resource, amount) {
            return false;
        }

        #[cfg(feature = "alloc")]
        {
            // Update statistics
            let mut stats = self.statistics.lock();
            if let Some(stat) = stats.get_mut(&agent_id) {
                stat.total_allocations += 1;
            }

            let mut quotas = self.quotas.lock();
            let agent_quotas = quotas.get_mut(&agent_id).unwrap();

            match resource {
                "memory" => agent_quotas.memory.allocate(amount),
                "cpu" => agent_quotas.cpu.allocate(amount),
                "network" => agent_quotas.network.allocate(amount),
                "io" => agent_quotas.io.allocate(amount),
                "fs" => agent_quotas.fs.allocate(amount),
                _ => false,
            }
        }

        #[cfg(not(feature = "alloc"))]
        {
            true
        }
    }

    /// Release resource
    pub fn release(&self, agent_id: u64, resource: &str, amount: u64) {
        #[cfg(feature = "alloc")]
        {
            // Update statistics
            let mut stats = self.statistics.lock();
            if let Some(stat) = stats.get_mut(&agent_id) {
                stat.total_releases += 1;
            }

            let mut quotas = self.quotas.lock();
            if let Some(agent_quotas) = quotas.get_mut(&agent_id) {
                match resource {
                    "memory" => agent_quotas.memory.release(amount),
                    "cpu" => agent_quotas.cpu.release(amount),
                    "network" => agent_quotas.network.release(amount),
                    "io" => agent_quotas.io.release(amount),
                    "fs" => agent_quotas.fs.release(amount),
                    _ => {},
                }
            }
        }
    }

    /// Get quota usage for agent
    #[cfg(feature = "alloc")]
    pub fn get_usage(&self, agent_id: u64, resource: &str) -> Option<(u64, u64)> {
        let quotas = self.quotas.lock();
        if let Some(agent_quotas) = quotas.get(&agent_id) {
            match resource {
                "memory" => Some((agent_quotas.memory.used, agent_quotas.memory.limit)),
                "cpu" => Some((agent_quotas.cpu.used, agent_quotas.cpu.limit)),
                "network" => Some((agent_quotas.network.used, agent_quotas.network.limit)),
                "io" => Some((agent_quotas.io.used, agent_quotas.io.limit)),
                "fs" => Some((agent_quotas.fs.used, agent_quotas.fs.limit)),
                _ => None,
            }
        } else {
            None
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_usage(&self, _agent_id: u64, _resource: &str) -> Option<(u64, u64)> {
        None
    }

    /// Set quota limit for agent
    #[cfg(feature = "alloc")]
    pub fn set_limit(&self, agent_id: u64, resource: &str, limit: u64) -> bool {
        let mut quotas = self.quotas.lock();
        let agent_quotas = quotas.entry(agent_id).or_insert_with(|| AgentQuotas::default());

        match resource {
            "memory" => {
                agent_quotas.memory.limit = limit;
                true
            }
            "cpu" => {
                agent_quotas.cpu.limit = limit;
                true
            }
            "network" => {
                agent_quotas.network.limit = limit;
                true
            }
            "io" => {
                agent_quotas.io.limit = limit;
                true
            }
            "fs" => {
                agent_quotas.fs.limit = limit;
                true
            }
            _ => false,
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn set_limit(&self, _agent_id: u64, _resource: &str, _limit: u64) -> bool {
        false
    }

    /// Get statistics for agent
    #[cfg(feature = "alloc")]
    pub fn get_statistics(&self, agent_id: u64) -> Option<(u64, u64, u64, u64)> {
        let stats = self.statistics.lock();
        if let Some(stat) = stats.get(&agent_id) {
            Some((
                stat.total_checks,
                stat.total_allocations,
                stat.total_releases,
                stat.quota_exceeded_count,
            ))
        } else {
            None
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_statistics(&self, _agent_id: u64) -> Option<(u64, u64, u64, u64)> {
        None
    }
}

impl Quota {
    fn check(&self, amount: u64) -> bool {
        self.used + amount <= self.limit
    }

    fn allocate(&mut self, amount: u64) -> bool {
        if self.check(amount) {
            self.used += amount;
            true
        } else {
            false
        }
    }

    fn release(&mut self, amount: u64) {
        self.used = self.used.saturating_sub(amount);
    }
}

impl Default for AgentQuotas {
    fn default() -> Self {
        Self {
            memory: Quota { limit: 1024 * 1024 * 1024, used: 0 }, // 1GB
            cpu: Quota { limit: 100, used: 0 }, // 100%
            network: Quota { limit: 100 * 1024 * 1024, used: 0 }, // 100MB/s
            io: Quota { limit: 1000, used: 0 }, // 1000 IOPS
            fs: Quota { limit: 10 * 1024 * 1024 * 1024, used: 0 }, // 10GB
        }
    }
}

