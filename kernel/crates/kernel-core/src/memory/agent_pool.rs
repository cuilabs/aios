//! Agent memory pool
//! 
//! Specialized memory pools for agent execution contexts
//! and cognitive memory fabric

use core::alloc::Layout;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::PhysAddr;
use x86_64::structures::paging::{Page, Size4KiB, PageTableFlags};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::memory::physical;
use crate::memory::virtual_mem;

/// Agent memory allocation entry
struct AgentMemoryEntry {
    virtual_addr: VirtAddr,
    size: usize,
    pages: Vec<Page<Size4KiB>>,
}

/// Agent memory pool
/// 
/// Provides isolated memory pools for agent execution
pub struct AgentMemoryPool {
    #[cfg(feature = "alloc")]
    agent_allocations: Mutex<BTreeMap<u64, Vec<AgentMemoryEntry>>>,
    #[cfg(not(feature = "alloc"))]
    agent_allocations: Mutex<()>,
    next_virtual_addr: Mutex<VirtAddr>,
}

static AGENT_POOL: Mutex<Option<AgentMemoryPool>> = Mutex::new(None);

/// Agent memory region starts at 0x1000_0000_0000 (1TB virtual address)
const AGENT_MEMORY_START: u64 = 0x1000_0000_0000;
const AGENT_MEMORY_PAGE_SIZE: usize = Size4KiB::SIZE as usize;

impl AgentMemoryPool {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            agent_allocations: Mutex::new(BTreeMap::new()),
            #[cfg(not(feature = "alloc"))]
            agent_allocations: Mutex::new(()),
            next_virtual_addr: Mutex::new(VirtAddr::new(AGENT_MEMORY_START)),
        }
    }

    /// Allocate memory for an agent
    pub fn allocate(&self, agent_id: u64, size: usize) -> Option<*mut u8> {
        #[cfg(feature = "alloc")]
        {
            // Align size to page boundary
            let aligned_size = (size + AGENT_MEMORY_PAGE_SIZE - 1) & !(AGENT_MEMORY_PAGE_SIZE - 1);
            let num_pages = aligned_size / AGENT_MEMORY_PAGE_SIZE;
            
            // Get starting virtual address
            let mut next_addr = self.next_virtual_addr.lock();
            let virtual_addr = *next_addr;
            
            // Allocate pages
            let mut pages = Vec::new();
            let mut current_addr = virtual_addr;
            
            for _ in 0..num_pages {
                // Allocate physical frame
                let frame = physical::allocate_frame()?;
                
                // Map page
                let page = Page::<Size4KiB>::containing_address(current_addr);
                let flags = PageTableFlags::PRESENT | 
                           PageTableFlags::WRITABLE | 
                           PageTableFlags::USER_ACCESSIBLE;
                
                if virtual_mem::map_page(current_addr, frame.start_address(), flags, agent_id, true).is_err() {
                    // Failed to map - deallocate frames
                    for allocated_page in &pages {
                        if let Ok(frame) = virtual_mem::unmap_page(allocated_page.start_address()) {
                            physical::deallocate_frame(frame);
                        }
                    }
                    return None;
                }
                
                pages.push(page);
                current_addr += AGENT_MEMORY_PAGE_SIZE;
            }
            
            // Update next virtual address
            *next_addr = current_addr;
            
            // Record allocation
            let mut allocations = self.agent_allocations.lock();
            let entry = AgentMemoryEntry {
                virtual_addr,
                size: aligned_size,
                pages,
            };
            allocations.entry(agent_id).or_insert_with(Vec::new).push(entry);
            
            Some(virtual_addr.as_mut_ptr())
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            None
        }
    }

    /// Deallocate agent memory
    pub unsafe fn deallocate(&self, ptr: *mut u8, agent_id: u64) {
        #[cfg(feature = "alloc")]
        {
            let virt_addr = VirtAddr::new(ptr as u64);
            let mut allocations = self.agent_allocations.lock();
            
            if let Some(agent_allocs) = allocations.get_mut(&agent_id) {
                // Find and remove allocation
                if let Some(pos) = agent_allocs.iter().position(|e| e.virtual_addr.as_ptr() == ptr) {
                    let entry = agent_allocs.remove(pos);
                    
                    // Unmap and deallocate pages
                    for page in entry.pages {
                        if let Ok(frame) = virtual_mem::unmap_page(page.start_address()) {
                            physical::deallocate_frame(frame);
                        }
                    }
                }
                
                // Remove agent entry if no allocations left
                if agent_allocs.is_empty() {
                    allocations.remove(&agent_id);
                }
            }
        }
    }
}

/// Initialize agent memory pools
pub fn init() {
    let mut pool = AGENT_POOL.lock();
    *pool = Some(AgentMemoryPool::new());
}

/// Allocate memory for agent
pub fn allocate(agent_id: u64, size: usize) -> Option<*mut u8> {
    let pool = AGENT_POOL.lock();
    if let Some(ref pool) = *pool {
        pool.allocate(agent_id, size)
    } else {
        None
    }
}

/// Deallocate agent memory
pub unsafe fn deallocate(ptr: *mut u8, agent_id: u64) {
    let pool = AGENT_POOL.lock();
    if let Some(ref pool) = *pool {
        pool.deallocate(ptr, agent_id);
    }
}
