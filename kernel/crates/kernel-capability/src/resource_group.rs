//! Per-agent resource groups

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Resource group
/// 
/// Groups resources for an agent (cgroups-like)
#[derive(Clone)]
pub struct ResourceGroup {
    pub agent_id: u64,
    pub cpu_quota: u64,
    pub memory_quota: u64,
    pub io_quota: u64,
    pub network_quota: u64,
    pub fs_quota: u64,
    pub children: Vec<u64>, // Child agent IDs
}

impl ResourceGroup {
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            cpu_quota: 100,
            memory_quota: 1024 * 1024 * 1024, // 1GB
            io_quota: 1000,
            network_quota: 100 * 1024 * 1024, // 100MB/s
            fs_quota: 10 * 1024 * 1024 * 1024, // 10GB
            children: Vec::new(),
        }
    }
}

