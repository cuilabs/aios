//! Virtual memory management
//! 
//! Provides page tables, page fault handling, memory protection, and COW

use x86_64::structures::paging::{
    PageTable, PageTableFlags, PhysFrame, Size4KiB, Mapper, MapperAllSizes,
    FrameAllocator, Page, PageTableIndex,
};
use x86_64::VirtAddr;
use x86_64::PhysAddr;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

/// Page table manager
pub struct PageTableManager {
    active_table: Mutex<Option<&'static mut PageTable>>,
    #[cfg(feature = "alloc")]
    agent_tables: Mutex<BTreeMap<u64, *mut PageTable>>, // Agent ID -> Page table
    #[cfg(not(feature = "alloc"))]
    agent_tables: Mutex<()>,
    frame_allocator: Mutex<Option<&'static mut dyn FrameAllocator<Size4KiB>>>,
}

static PAGE_MANAGER: Mutex<Option<PageTableManager>> = Mutex::new(None);

/// Initialize virtual memory management
pub fn init(level_4_table: &'static mut PageTable, frame_allocator: &'static mut dyn FrameAllocator<Size4KiB>) {
    let mut manager = PAGE_MANAGER.lock();
    *manager = Some(PageTableManager {
        active_table: Mutex::new(Some(level_4_table)),
        #[cfg(feature = "alloc")]
        agent_tables: Mutex::new(BTreeMap::new()),
        #[cfg(not(feature = "alloc"))]
        agent_tables: Mutex::new(()),
        frame_allocator: Mutex::new(Some(frame_allocator)),
    });
}

/// Map virtual address to physical address
pub fn map_page(
    virtual_addr: VirtAddr,
    physical_addr: PhysAddr,
    flags: PageTableFlags,
) -> Result<(), MapError> {
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut table = manager.active_table.lock();
        if let Some(ref mut table) = *table {
            let mut frame_allocator = manager.frame_allocator.lock();
            if let Some(ref mut allocator) = *frame_allocator {
                let page = Page::<Size4KiB>::containing_address(virtual_addr);
                let frame = PhysFrame::<Size4KiB>::containing_address(physical_addr);
                
                unsafe {
                    table.map_to(page, frame, flags, allocator)
                        .map_err(|_| MapError::MappingFailed)?
                        .flush();
                }
                
                Ok(())
            } else {
                Err(MapError::NoFrameAllocator)
            }
        } else {
            Err(MapError::NoPageTable)
        }
    } else {
        Err(MapError::NotInitialized)
    }
}

/// Unmap virtual address
pub fn unmap_page(virtual_addr: VirtAddr) -> Result<PhysFrame<Size4KiB>, UnmapError> {
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut table = manager.active_table.lock();
        if let Some(ref mut table) = *table {
            let page = Page::<Size4KiB>::containing_address(virtual_addr);
            
            unsafe {
                let (frame, flush) = table.unmap(page)
                    .map_err(|_| UnmapError::UnmappingFailed)?;
                flush.flush();
                Ok(frame)
            }
        } else {
            Err(UnmapError::NoPageTable)
        }
    } else {
        Err(UnmapError::NotInitialized)
    }
}

/// Handle page fault
pub fn handle_page_fault(
    address: VirtAddr,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
    agent_id: Option<u64>,
) -> Result<(), PageFaultError> {
    // Check if page is valid
    if !error_code.contains(x86_64::structures::idt::PageFaultErrorCode::PROTECTION_VIOLATION) {
        // Page not present - try to load it
        return load_page(address, agent_id);
    }
    
    // Protection violation
    if error_code.contains(x86_64::structures::idt::PageFaultErrorCode::USER_MODE) {
        // User mode violation - check agent permissions
        if let Some(agent_id) = agent_id {
            return handle_agent_protection_violation(agent_id, address, error_code);
        }
    }
    
    // Kernel protection violation - fatal
    Err(PageFaultError::KernelProtectionViolation)
}

/// Load page (demand paging)
fn load_page(address: VirtAddr, agent_id: Option<u64>) -> Result<(), PageFaultError> {
    // Allocate physical frame
    let frame = allocate_frame().ok_or(PageFaultError::OutOfMemory)?;
    
    // Determine flags
    let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    if let Some(_agent_id) = agent_id {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    }
    
    // Map page
    map_page(address, frame.start_address(), flags)
        .map_err(|_| PageFaultError::MappingFailed)?;
    
    // Zero the page
    unsafe {
        let page_ptr: *mut u8 = address.as_mut_ptr();
        core::ptr::write_bytes(page_ptr, 0, 4096);
    }
    
    Ok(())
}

/// Handle agent protection violation
fn handle_agent_protection_violation(
    agent_id: u64,
    address: VirtAddr,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) -> Result<(), PageFaultError> {
    // Check agent capabilities
    // TODO: Check if agent has permission to access this address
    
    // Check if this is a copy-on-write page
    if error_code.contains(x86_64::structures::idt::PageFaultErrorCode::INSTRUCTION_FETCH) {
        // Instruction fetch violation
        return Err(PageFaultError::InstructionFetchViolation);
    }
    
    if error_code.contains(x86_64::structures::idt::PageFaultErrorCode::WRITE_VIOLATION) {
        // Write violation - might be COW
        return handle_cow_fault(agent_id, address);
    }
    
    // Read violation
    Err(PageFaultError::ReadViolation)
}

/// Handle copy-on-write fault
fn handle_cow_fault(agent_id: u64, address: VirtAddr) -> Result<(), PageFaultError> {
    // TODO: Implement copy-on-write
    // 1. Check if page is marked as COW
    // 2. Allocate new frame
    // 3. Copy page contents
    // 4. Update mapping
    
    // For now, just allow write
    let frame = allocate_frame().ok_or(PageFaultError::OutOfMemory)?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    
    map_page(address, frame.start_address(), flags)
        .map_err(|_| PageFaultError::MappingFailed)?;
    
    Ok(())
}

/// Allocate a physical frame
fn allocate_frame() -> Option<PhysFrame<Size4KiB>> {
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut allocator = manager.frame_allocator.lock();
        if let Some(ref mut allocator) = *allocator {
            allocator.allocate_frame()
        } else {
            None
        }
    } else {
        None
    }
}

/// Create page table for agent
pub fn create_agent_page_table(agent_id: u64) -> Result<(), PageTableError> {
    // TODO: Allocate and initialize page table for agent
    // This requires frame allocation and page table setup
    Ok(())
}

/// Switch to agent page table
pub fn switch_to_agent_page_table(agent_id: u64) -> Result<(), PageTableError> {
    // TODO: Load agent's page table into CR3
    Ok(())
}

/// Map error
#[derive(Debug, Clone, Copy)]
pub enum MapError {
    NotInitialized,
    NoPageTable,
    NoFrameAllocator,
    MappingFailed,
}

/// Unmap error
#[derive(Debug, Clone, Copy)]
pub enum UnmapError {
    NotInitialized,
    NoPageTable,
    UnmappingFailed,
}

/// Page fault error
#[derive(Debug, Clone, Copy)]
pub enum PageFaultError {
    OutOfMemory,
    MappingFailed,
    KernelProtectionViolation,
    InstructionFetchViolation,
    ReadViolation,
    CowFailed,
}

/// Page table error
#[derive(Debug, Clone, Copy)]
pub enum PageTableError {
    NotInitialized,
    OutOfMemory,
    InvalidAgent,
}
