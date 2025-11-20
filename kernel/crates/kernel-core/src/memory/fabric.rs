//! Memory Fabric Extension
//! 
//! Cross-agent shared semantic memory with:
//! - Deterministic low-level shared memory pages
//! - Ephemeral vs persistent memory regions
//! - Memory tagging per agent
//! - Memory lease/expiration system

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use x86_64::VirtAddr;
use x86_64::PhysAddr;
use x86_64::structures::paging::PageTableFlags;

#[cfg(feature = "alloc")]
use crate::time;
#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};

/// Memory region type
#[repr(u32)]
pub enum MemoryRegionType {
    Ephemeral = 1,   // Temporary, cleared on agent termination
    Persistent = 2,  // Persists across agent lifecycle
    Shared = 3,      // Shared between agents
}

/// Memory tag
pub struct MemoryTag {
    pub agent_id: u64,
    pub tag: Vec<u8>,
    pub permissions: u64, // Read, write, execute flags
}

/// Memory lease
pub struct MemoryLease {
    pub agent_id: u64,
    pub region_id: u64,
    pub expires_at: u64,
    pub auto_renew: bool,
}

/// Memory fabric manager
pub struct MemoryFabricManager {
    regions: Mutex<BTreeMap<u64, MemoryRegion>>,
    shared_pages: Mutex<BTreeMap<u64, SharedMemoryPage>>,
    leases: Mutex<BTreeMap<u64, MemoryLease>>,
    tags: Mutex<BTreeMap<u64, Vec<MemoryTag>>>,
}

struct MemoryRegion {
    region_id: u64,
    agent_id: u64,
    region_type: MemoryRegionType,
    start: VirtAddr,
    size: u64,
    tags: Vec<u64>,
}

struct SharedMemoryPage {
    page_id: u64,
    physical_addr: PhysAddr,
    virtual_addrs: BTreeMap<u64, VirtAddr>, // Agent ID -> Virtual address
    agents: Vec<u64>,        // Agents with access
    version: u64,            // Memory version for versioning
    ref_count: usize,        // Reference count for COW
}

/// Global memory fabric manager instance
static MEMORY_FABRIC: spin::Once<MemoryFabricManager> = spin::Once::new();

impl MemoryFabricManager {
    pub fn new() -> Self {
        Self {
            regions: Mutex::new(BTreeMap::new()),
            shared_pages: Mutex::new(BTreeMap::new()),
            leases: Mutex::new(BTreeMap::new()),
            tags: Mutex::new(BTreeMap::new()),
        }
    }

    /// Initialize global memory fabric manager
    pub fn init() {
        MEMORY_FABRIC.call_once(|| MemoryFabricManager::new());
    }

    /// Get global memory fabric manager instance
    pub fn get() -> Option<&'static MemoryFabricManager> {
        MEMORY_FABRIC.get()
    }

    /// Create memory region
    pub fn create_region(
        &self,
        agent_id: u64,
        region_type: MemoryRegionType,
        size: u64,
    ) -> Result<u64, MemoryFabricError> {
        let region_id = self.generate_region_id();

        // Allocate virtual address space for region using virtual memory manager
        use super::virtual_mem;
        // Allocate virtual address range for agent (starts at 0x10000000, grows upward)
        let base_vaddr = 0x10000000u64;
        let region_offset = region_id as u64 * 0x1000000; // 16MB per region
        let start = VirtAddr::new(base_vaddr + region_offset);
        
        // Validate that address range doesn't overlap with existing regions
        let regions = self.regions.lock();
        for (_, existing_region) in regions.iter() {
            let existing_start = existing_region.start.as_u64();
            let existing_end = existing_start + existing_region.size;
            let new_start = start.as_u64();
            let new_end = new_start + size;
            
            if (new_start >= existing_start && new_start < existing_end) ||
               (new_end > existing_start && new_end <= existing_end) ||
               (new_start <= existing_start && new_end >= existing_end) {
                return Err(MemoryFabricError::OutOfMemory); // Overlap detected
            }
        }

        let region = MemoryRegion {
            region_id,
            agent_id,
            region_type,
            start,
            size,
            tags: Vec::new(),
        };

        let mut regions = self.regions.lock();
        regions.insert(region_id, region);

        // Publish event
        #[cfg(feature = "alloc")]
        {
            publish_event(KernelEvent {
                event_type: EventType::VirtualMemory,
                timestamp: time::now(),
                agent_id: Some(agent_id),
                data: alloc::vec::Vec::new(),
            });
        }

        Ok(region_id)
    }

    /// Create shared memory page
    pub fn create_shared_page(
        &self,
        agents: Vec<u64>,
    ) -> Result<u64, MemoryFabricError> {
        let page_id = self.generate_page_id();

        // Allocate physical page using virtual memory manager
        use super::virtual_mem;
        use super::physical;
        let frame = physical::allocate_frame().ok_or(MemoryFabricError::OutOfMemory)?;
        let physical_addr = frame.start_address();

        let shared_page = SharedMemoryPage {
            page_id,
            physical_addr,
            virtual_addrs: BTreeMap::new(),
            agents: agents.clone(),
        };

        // Map page into each agent's address space
        let mut virtual_addrs_map = BTreeMap::new();
        for agent_id in &agents {
            // Allocate virtual address for this agent
            let virtual_addr = VirtAddr::new(0x30000000 + (page_id as u64 * 4096) + (*agent_id * 0x1000000));
            
            // Map page (no capability check needed for shared pages - handled by policy)
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
            if let Err(_) = virtual_mem::map_page(virtual_addr, physical_addr, flags, *agent_id, true) {
                return Err(MemoryFabricError::OutOfMemory);
            }
            
            virtual_addrs_map.insert(*agent_id, virtual_addr);
        }

        // Mark as COW page for shared memory
        use super::virtual_mem;
        virtual_mem::mark_cow_page(physical_addr, 
            x86_64::structures::paging::PhysFrame::containing_address(physical_addr),
            agents.len());

        // Update shared page with virtual addresses
        let mut shared_page = SharedMemoryPage {
            page_id,
            physical_addr,
            virtual_addrs: virtual_addrs_map,
            agents,
            version: 1,
            ref_count: agents.len(),
        };

        let mut shared_pages = self.shared_pages.lock();
        shared_pages.insert(page_id, shared_page);

        // Publish event
        #[cfg(feature = "alloc")]
        {
            publish_event(KernelEvent {
                event_type: EventType::VirtualMemory,
                timestamp: time::now(),
                agent_id: None, // Shared across multiple agents
                data: alloc::vec::Vec::new(),
            });
        }

        Ok(page_id)
    }

    /// Tag memory region
    pub fn tag_region(&self, region_id: u64, tag: MemoryTag) -> Result<(), MemoryFabricError> {
        let mut tags = self.tags.lock();
        let region_tags = tags.entry(region_id).or_insert_with(Vec::new);
        region_tags.push(tag);
        Ok(())
    }

    /// Create memory lease
    pub fn create_lease(
        &self,
        agent_id: u64,
        region_id: u64,
        expires_at: u64,
        auto_renew: bool,
    ) -> Result<u64, MemoryFabricError> {
        let lease_id = self.generate_lease_id();

        let lease = MemoryLease {
            agent_id,
            region_id,
            expires_at,
            auto_renew,
        };

        let mut leases = self.leases.lock();
        leases.insert(lease_id, lease);

        Ok(lease_id)
    }

    /// Check and renew leases
    pub fn check_leases(&self, now: u64) {
        let mut leases = self.leases.lock();
        leases.retain(|_lease_id, lease| {
            if lease.expires_at < now {
                if lease.auto_renew {
                    // Auto-renew lease
                    lease.expires_at = now + 3600; // 1 hour default
                    true
                } else {
                    // Lease expired - remove
                    false
                }
            } else {
                true
            }
        });
    }

    /// Cleanup ephemeral regions for agent
    pub fn cleanup_agent_regions(&self, agent_id: u64) {
        let mut regions = self.regions.lock();
        regions.retain(|_region_id, region| {
            // Remove ephemeral regions only
            if region.agent_id == agent_id && matches!(region.region_type, MemoryRegionType::Ephemeral) {
                false
            } else {
                true
            }
        });
        
        // Also cleanup expired leases for this agent
        #[cfg(feature = "alloc")]
        {
            let now = time::now();
            let mut leases = self.leases.lock();
            leases.retain(|_lease_id, lease| {
                if lease.agent_id == agent_id && lease.expires_at < now && !lease.auto_renew {
                    false
                } else {
                    true
                }
            });
        }
    }
    
    /// Get tags for region
    pub fn get_tags(&self, region_id: u64) -> Option<Vec<MemoryTag>> {
        let tags = self.tags.lock();
        tags.get(&region_id).cloned()
    }
    
    /// Get tags by agent
    pub fn get_tags_by_agent(&self, agent_id: u64) -> Vec<(u64, MemoryTag)> {
        let tags = self.tags.lock();
        let mut result = Vec::new();
        for (region_id, region_tags) in tags.iter() {
            for tag in region_tags.iter() {
                if tag.agent_id == agent_id {
                    result.push((*region_id, tag.clone()));
                }
            }
        }
        result
    }
    
    /// Expire leases (called periodically)
    pub fn expire_leases(&self) {
        #[cfg(feature = "alloc")]
        {
            let now = time::now();
            self.check_leases(now);
        }
    }

    /// Get shared page virtual address for agent
    pub fn get_shared_page_addr(&self, page_id: u64, agent_id: u64) -> Option<VirtAddr> {
        let shared_pages = self.shared_pages.lock();
        if let Some(page) = shared_pages.get(&page_id) {
            page.virtual_addrs.get(&agent_id).copied()
        } else {
            None
        }
    }

    /// Increment memory version (for versioning)
    pub fn increment_version(&self, page_id: u64) -> Result<u64, MemoryFabricError> {
        let mut shared_pages = self.shared_pages.lock();
        if let Some(page) = shared_pages.get_mut(&page_id) {
            page.version += 1;
            Ok(page.version)
        } else {
            Err(MemoryFabricError::InvalidRegion)
        }
    }

    /// Get memory version
    pub fn get_version(&self, page_id: u64) -> Option<u64> {
        let shared_pages = self.shared_pages.lock();
        shared_pages.get(&page_id).map(|page| page.version)
    }

    /// Add agent to shared page
    pub fn add_agent_to_shared_page(&self, page_id: u64, agent_id: u64) -> Result<(), MemoryFabricError> {
        use super::virtual_mem;
        
        let mut shared_pages = self.shared_pages.lock();
        if let Some(page) = shared_pages.get_mut(&page_id) {
            // Check if agent already has access
            if page.agents.contains(&agent_id) {
                return Ok(());
            }
            
            let physical_addr = page.physical_addr;
            // Map page into agent's address space
            let virtual_addr = VirtAddr::new(0x30000000 + (page_id as u64 * 4096) + (agent_id * 0x1000000));
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
            
            drop(shared_pages); // Release lock before mapping
            
            if let Err(_) = virtual_mem::map_page(virtual_addr, physical_addr, flags, agent_id, None) {
                return Err(MemoryFabricError::OutOfMemory);
            }
            
            let mut shared_pages = self.shared_pages.lock();
            if let Some(page) = shared_pages.get_mut(&page_id) {
                page.virtual_addrs.insert(agent_id, virtual_addr);
                page.agents.push(agent_id);
                page.ref_count += 1;
            }
            
            Ok(())
        } else {
            Err(MemoryFabricError::InvalidRegion)
        }
    }

    /// Generate region ID
    fn generate_region_id(&self) -> u64 {
        static COUNTER: Mutex<u64> = Mutex::new(1);
        let mut counter = COUNTER.lock();
        let id = *counter;
        *counter = counter.wrapping_add(1);
        id
    }

    /// Generate page ID
    fn generate_page_id(&self) -> u64 {
        static COUNTER: Mutex<u64> = Mutex::new(1);
        let mut counter = COUNTER.lock();
        let id = *counter;
        *counter = counter.wrapping_add(1);
        id
    }

    /// Generate lease ID
    fn generate_lease_id(&self) -> u64 {
        static COUNTER: Mutex<u64> = Mutex::new(1);
        let mut counter = COUNTER.lock();
        let id = *counter;
        *counter = counter.wrapping_add(1);
        id
    }
}

#[derive(Debug)]
pub enum MemoryFabricError {
    InvalidRegion,
    InvalidAgent,
    OutOfMemory,
    PermissionDenied,
}

