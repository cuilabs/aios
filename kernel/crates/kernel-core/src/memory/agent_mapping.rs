//! Agent memory mapping
//! 
//! Maps virtual addresses to agent IDs for exception handling

use x86_64::VirtAddr;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

/// Agent memory region
#[derive(Clone, Copy, Debug)]
struct AgentRegion {
    agent_id: u64,
    start: VirtAddr,
    end: VirtAddr,
}

/// Agent memory mapping
pub struct AgentMemoryMapping {
    #[cfg(feature = "alloc")]
    regions: Mutex<BTreeMap<u64, AgentRegion>>, // Agent ID -> Region
    #[cfg(feature = "alloc")]
    address_map: Mutex<BTreeMap<u64, u64>>, // Address (as u64) -> Agent ID
    #[cfg(not(feature = "alloc"))]
    regions: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    address_map: Mutex<()>,
}

static AGENT_MAPPING: Mutex<Option<AgentMemoryMapping>> = Mutex::new(None);

/// Initialize agent memory mapping
pub fn init() {
    let mut mapping = AGENT_MAPPING.lock();
    *mapping = Some(AgentMemoryMapping {
        #[cfg(feature = "alloc")]
        regions: Mutex::new(BTreeMap::new()),
        #[cfg(not(feature = "alloc"))]
        regions: Mutex::new(()),
        #[cfg(feature = "alloc")]
        address_map: Mutex::new(BTreeMap::new()),
        #[cfg(not(feature = "alloc"))]
        address_map: Mutex::new(()),
    });
}

/// Register agent memory region
#[cfg(feature = "alloc")]
pub fn register_agent_region(agent_id: u64, start: VirtAddr, end: VirtAddr) {
    let mapping = AGENT_MAPPING.lock();
    if let Some(ref mapping) = *mapping {
        let region = AgentRegion {
            agent_id,
            start,
            end,
        };
        
        let mut regions = mapping.regions.lock();
        regions.insert(agent_id, region);
        
        // Build address map (could use interval tree for optimization)
        let mut address_map = mapping.address_map.lock();
        let mut addr = start.as_u64();
        let end_addr = end.as_u64();
        while addr < end_addr {
            address_map.insert(addr, agent_id);
            addr += 4096; // Page size
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub fn register_agent_region(_agent_id: u64, _start: VirtAddr, _end: VirtAddr) {
    // No-op without alloc
}

/// Unregister agent memory region
#[cfg(feature = "alloc")]
pub fn unregister_agent_region(agent_id: u64) {
    let mapping = AGENT_MAPPING.lock();
    if let Some(ref mapping) = *mapping {
        let mut regions = mapping.regions.lock();
        if let Some(region) = regions.remove(&agent_id) {
            // Remove from address map
            let mut address_map = mapping.address_map.lock();
            let mut addr = region.start.as_u64();
            let end_addr = region.end.as_u64();
            while addr < end_addr {
                address_map.remove(&addr);
                addr += 4096;
            }
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub fn unregister_agent_region(_agent_id: u64) {
    // No-op without alloc
}

/// Get agent ID for address
pub fn get_agent_for_address(address: VirtAddr) -> Option<u64> {
    #[cfg(feature = "alloc")]
    {
        let mapping = AGENT_MAPPING.lock();
        if let Some(ref mapping) = *mapping {
            let address_map = mapping.address_map.lock();
            // Check exact address first
            if let Some(agent_id) = address_map.get(&address.as_u64()) {
                return Some(*agent_id);
            }
            
            // Check page-aligned address
            let page_addr = address.as_u64() & !0xFFF;
            address_map.get(&page_addr).copied()
        } else {
            None
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        None
    }
}

