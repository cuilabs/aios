//! Per-agent quota management

use spin::Mutex;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

/// Quota manager
/// 
/// Enforces per-agent quotas across all subsystems
pub struct QuotaManager {
    quotas: Mutex<BTreeMap<u64, AgentQuotas>>,
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
            quotas: Mutex::new(BTreeMap::new()),
        }
    }

    /// Check if agent can use resource
    pub fn check(&self, agent_id: u64, resource: &str, amount: u64) -> bool {
        let mut quotas = self.quotas.lock();
        let agent_quotas = quotas.entry(agent_id).or_insert_with(|| AgentQuotas::default());

        match resource {
            "memory" => agent_quotas.memory.check(amount),
            "cpu" => agent_quotas.cpu.check(amount),
            "network" => agent_quotas.network.check(amount),
            "io" => agent_quotas.io.check(amount),
            "fs" => agent_quotas.fs.check(amount),
            _ => false,
        }
    }

    /// Allocate resource
    pub fn allocate(&self, agent_id: u64, resource: &str, amount: u64) -> bool {
        if !self.check(agent_id, resource, amount) {
            return false;
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

    /// Release resource
    pub fn release(&self, agent_id: u64, resource: &str, amount: u64) {
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

