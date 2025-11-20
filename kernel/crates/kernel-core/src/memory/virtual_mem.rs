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
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
#[cfg(feature = "alloc")]
use crate::time;
// Policy checks happen via event bus or at syscall layer, not here

/// COW page reference count
#[cfg(feature = "alloc")]
struct CowPageRef {
    frame: PhysFrame<Size4KiB>,
    ref_count: usize,
}

/// Page table manager
pub struct PageTableManager {
    active_table: Mutex<Option<&'static mut PageTable>>,
    #[cfg(feature = "alloc")]
    agent_tables: Mutex<BTreeMap<u64, *mut PageTable>>, // Agent ID -> Page table
    #[cfg(not(feature = "alloc"))]
    agent_tables: Mutex<()>,
    frame_allocator: Mutex<Option<&'static mut dyn FrameAllocator<Size4KiB>>>,
    #[cfg(feature = "alloc")]
    cow_pages: Mutex<BTreeMap<PhysAddr, CowPageRef>>, // Physical address -> COW ref count
    #[cfg(not(feature = "alloc"))]
    cow_pages: Mutex<()>,
    #[cfg(feature = "alloc")]
    page_fault_stats: Mutex<PageFaultStats>,
    #[cfg(not(feature = "alloc"))]
    page_fault_stats: Mutex<()>,
}

/// Page fault statistics
#[cfg(feature = "alloc")]
struct PageFaultStats {
    total_faults: u64,
    cow_faults: u64,
    protection_violations: u64,
    demand_paging: u64,
    by_agent: BTreeMap<u64, u64>, // Agent ID -> fault count
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
        #[cfg(feature = "alloc")]
        cow_pages: Mutex::new(BTreeMap::new()),
        #[cfg(not(feature = "alloc"))]
        cow_pages: Mutex::new(()),
        #[cfg(feature = "alloc")]
        page_fault_stats: Mutex::new(PageFaultStats {
            total_faults: 0,
            cow_faults: 0,
            protection_violations: 0,
            demand_paging: 0,
            by_agent: BTreeMap::new(),
        }),
        #[cfg(not(feature = "alloc"))]
        page_fault_stats: Mutex::new(()),
    });
}

/// Map virtual address to physical address
pub fn map_page(
    virtual_addr: VirtAddr,
    physical_addr: PhysAddr,
    flags: PageTableFlags,
    agent_id: u64,
    capability_validated: bool, // Whether capability was validated at syscall layer
) -> Result<(), MapError> {
    // Capability validation happens at syscall layer
    if !capability_validated {
        // Publish security violation event
        #[cfg(feature = "alloc")]
        {
            publish_event(KernelEvent {
                event_type: EventType::Security,
                timestamp: time::now(),
                agent_id: Some(agent_id),
                data: alloc::vec![("action".into(), "unauthorized_memory_map".into())],
            });
        }
        return Err(MapError::PermissionDenied);
    }
    
    // Check policy
    #[cfg(feature = "alloc")]
    {
        if !check_policy(agent_id, "vm_map_page", None) {
            return Err(MapError::PermissionDenied);
        }
    }
    
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
                
                // Update observability counters (4KB page size)
                #[cfg(feature = "alloc")]
                {
                    // Publish memory allocation event (observability will subscribe)
                    publish_event(KernelEvent {
                        event_type: EventType::Memory,
                        timestamp: time::now(),
                        agent_id: Some(agent_id),
                        data: alloc::vec![("action".into(), "allocated".into()), ("size".into(), "4096".into())],
                    });
                }
                // Publish virtual memory event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::VirtualMemory,
                        timestamp: time::now(),
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
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
                
                // Publish virtual memory event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::VirtualMemory,
                        timestamp: time::now(),
                        agent_id: None, // Unmap doesn't have agent_id readily available
                        data: alloc::vec::Vec::new(),
                    });
                }
                
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
    // Update statistics
    #[cfg(feature = "alloc")]
    {
        let manager = PAGE_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let mut stats = manager.page_fault_stats.lock();
            stats.total_faults += 1;
            if let Some(agent_id) = agent_id {
                *stats.by_agent.entry(agent_id).or_insert(0) += 1;
            }
        }
    }
    
    // Check if page is valid
    if !error_code.contains(x86_64::structures::idt::PageFaultErrorCode::PROTECTION_VIOLATION) {
        // Page not present - try to load it (demand paging)
        #[cfg(feature = "alloc")]
        {
            let manager = PAGE_MANAGER.lock();
            if let Some(ref manager) = *manager {
                let mut stats = manager.page_fault_stats.lock();
                stats.demand_paging += 1;
            }
        }
        return load_page(address, agent_id);
    }
    
    // Protection violation
    #[cfg(feature = "alloc")]
    {
        let manager = PAGE_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let mut stats = manager.page_fault_stats.lock();
            stats.protection_violations += 1;
        }
    }
    
    if error_code.contains(x86_64::structures::idt::PageFaultErrorCode::USER_MODE) {
        // User mode violation - check agent permissions
        if let Some(agent_id) = agent_id {
            // Publish virtual memory event (page fault)
            #[cfg(feature = "alloc")]
            {
                publish_event(KernelEvent {
                    event_type: EventType::VirtualMemory,
                    timestamp: time::now(),
                    agent_id: Some(agent_id),
                    data: alloc::vec::Vec::new(),
                });
            }
            return handle_agent_protection_violation(agent_id, address, error_code);
        }
    }
    
    // Kernel protection violation - fatal
    // Publish security violation event
    #[cfg(feature = "alloc")]
    {
        publish_event(KernelEvent {
            event_type: EventType::Security,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec::Vec::new(),
        });
    }
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
    
    // Map page (no capability check needed for page fault handling)
    map_page(address, frame.start_address(), flags, agent_id.unwrap_or(0), true) // Capability validated at syscall layer
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
    if !check_agent_permission(agent_id, address) {
        return Err(PageFaultError::ReadViolation);
    }
    
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

/// Check if agent has permission to access address
fn check_agent_permission(agent_id: u64, address: VirtAddr) -> bool {
    // Check if address is in agent's memory region
    use crate::memory::agent_mapping;
    if let Some(mapped_agent_id) = agent_mapping::get_agent_for_address(address) {
        mapped_agent_id == agent_id
    } else {
        // Address not in any agent region - deny access
        false
    }
}

/// Handle copy-on-write fault
fn handle_cow_fault(agent_id: u64, address: VirtAddr) -> Result<(), PageFaultError> {
    // Update COW statistics
    #[cfg(feature = "alloc")]
    {
        let manager = PAGE_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let mut stats = manager.page_fault_stats.lock();
            stats.cow_faults += 1;
        }
    }
    
    // Get current page mapping
    let _page = Page::<Size4KiB>::containing_address(address);
    
    // Unmap current page to get physical frame
    let old_frame = match unmap_page(address) {
        Ok(frame) => frame,
        Err(_) => {
            // Page not mapped - just allocate new frame
            let frame = allocate_frame().ok_or(PageFaultError::OutOfMemory)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
            map_page(address, frame.start_address(), flags, agent_id, true) // Capability validated
                .map_err(|_| PageFaultError::MappingFailed)?;
            return Ok(());
        }
    };
    
    let old_phys_addr = old_frame.start_address();
    
    // Check if this is a COW page with reference counting
    #[cfg(feature = "alloc")]
    {
        let manager = PAGE_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let mut cow_pages = manager.cow_pages.lock();
            
            if let Some(cow_ref) = cow_pages.get_mut(&old_phys_addr) {
                // Decrement reference count
                cow_ref.ref_count -= 1;
                
                if cow_ref.ref_count == 0 {
                    // Last reference - can deallocate
                    cow_pages.remove(&old_phys_addr);
                    crate::memory::physical::deallocate_frame(old_frame);
                } else {
                    // Still referenced - don't deallocate
                    // Just allocate new frame and copy
                }
            }
        }
    }
    
    // Allocate new frame for COW
    let new_frame = allocate_frame().ok_or(PageFaultError::OutOfMemory)?;
    let new_phys_addr = new_frame.start_address();
    
    // Copy page contents
    unsafe {
        let old_ptr = old_phys_addr.as_u64() as *const u8;
        let new_ptr = new_phys_addr.as_u64() as *mut u8;
        core::ptr::copy_nonoverlapping(old_ptr, new_ptr, Size4KiB::SIZE as usize);
    }
    
    // Map new frame with write permission
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    map_page(address, new_phys_addr, flags, agent_id, true) // Capability validated
        .map_err(|_| PageFaultError::MappingFailed)?;
    
    // If old frame wasn't COW tracked, deallocate it
    #[cfg(feature = "alloc")]
    {
        let manager = PAGE_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let cow_pages = manager.cow_pages.lock();
            if !cow_pages.contains_key(&old_phys_addr) {
                crate::memory::physical::deallocate_frame(old_frame);
            }
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        crate::memory::physical::deallocate_frame(old_frame);
    }
    
    Ok(())
}

/// Mark page as COW (for shared memory)
#[cfg(feature = "alloc")]
pub fn mark_cow_page(phys_addr: PhysAddr, frame: PhysFrame<Size4KiB>, initial_refs: usize) {
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut cow_pages = manager.cow_pages.lock();
        cow_pages.insert(phys_addr, CowPageRef {
            frame,
            ref_count: initial_refs,
        });
    }
}

#[cfg(not(feature = "alloc"))]
pub fn mark_cow_page(_phys_addr: PhysAddr, _frame: PhysFrame<Size4KiB>, _initial_refs: usize) {
    // No-op
}

/// Get page fault statistics
#[cfg(feature = "alloc")]
pub fn get_page_fault_stats() -> (u64, u64, u64, u64, BTreeMap<u64, u64>) {
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let stats = manager.page_fault_stats.lock();
        (
            stats.total_faults,
            stats.cow_faults,
            stats.protection_violations,
            stats.demand_paging,
            stats.by_agent.clone(),
        )
    } else {
        (0, 0, 0, 0, BTreeMap::new())
    }
}

#[cfg(not(feature = "alloc"))]
pub fn get_page_fault_stats() -> (u64, u64, u64, u64, BTreeMap<u64, u64>) {
    (0, 0, 0, 0, BTreeMap::new())
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
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        #[cfg(feature = "alloc")]
        {
            let mut allocator = manager.frame_allocator.lock();
            if let Some(ref mut allocator) = *allocator {
                // Allocate frame for level 4 page table
                let frame = allocator.allocate_frame().ok_or(PageTableError::OutOfMemory)?;
                let table_ptr = frame.start_address().as_u64() as *mut PageTable;
                
                unsafe {
                    // Initialize page table (zero it)
                    let table = &mut *table_ptr;
                    core::ptr::write_bytes(table, 0, 1);
                    
                    // Store in agent_tables
                    let mut agent_tables = manager.agent_tables.lock();
                    agent_tables.insert(agent_id, table_ptr);
                }
                
                Ok(())
            } else {
                Err(PageTableError::OutOfMemory)
            }
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(PageTableError::OutOfMemory)
        }
    } else {
        Err(PageTableError::NotInitialized)
    }
}

/// Switch to agent page table
pub fn switch_to_agent_page_table(agent_id: u64) -> Result<(), PageTableError> {
    let manager = PAGE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        #[cfg(feature = "alloc")]
        {
            let agent_tables = manager.agent_tables.lock();
            if let Some(table_ptr) = agent_tables.get(&agent_id) {
                unsafe {
                    // Load page table into CR3
                    let phys_addr = (*table_ptr as *const PageTable as u64) as u64;
                    x86_64::registers::control::Cr3::write(
                        PhysFrame::containing_address(PhysAddr::new(phys_addr)),
                        x86_64::registers::control::Cr3Flags::empty(),
                    );
                }
                Ok(())
            } else {
                Err(PageTableError::InvalidAgent)
            }
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(PageTableError::InvalidAgent)
        }
    } else {
        Err(PageTableError::NotInitialized)
    }
}

/// Map error
#[derive(Debug, Clone, Copy)]
pub enum MapError {
    NotInitialized,
    NoPageTable,
    NoFrameAllocator,
    MappingFailed,
    PermissionDenied,
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
