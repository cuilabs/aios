//! Memory management for AIOS kernel
//! 
//! Provides physical and virtual memory management,
//! with special support for agent memory pools.

pub mod physical;
pub mod virtual_mem;
pub mod allocator;
pub mod agent_pool;
pub mod fabric;
pub mod agent_mapping;
#[cfg(feature = "alloc")]
pub mod ai_adaptive;

use core::alloc::Layout;

/// Initialize memory management
pub fn init() {
    physical::init();
    virtual_mem::init();
    allocator::init();
    agent_pool::init();
    fabric::MemoryFabricManager::init();
    #[cfg(feature = "alloc")]
    ai_adaptive::init();
}

/// Allocate memory for agent use
pub fn allocate_agent_memory(size: usize) -> Option<*mut u8> {
    agent_pool::allocate(size)
}

/// Deallocate agent memory
pub unsafe fn deallocate_agent_memory(ptr: *mut u8, size: usize) {
    agent_pool::deallocate(ptr, size);
}

