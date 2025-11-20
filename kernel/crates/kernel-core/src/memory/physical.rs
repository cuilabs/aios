//! Physical memory management
//! 
//! Provides frame allocation using a bitmap-based allocator

use x86_64::structures::paging::{PhysFrame, Size4KiB};
use x86_64::PhysAddr;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Frame allocator using bitmap
pub struct BitmapFrameAllocator {
    bitmap: &'static mut [u64], // Each bit represents a frame (1 = free, 0 = used)
    total_frames: usize,
    first_frame: PhysFrame<Size4KiB>,
    next_free: Mutex<usize>,
}

static mut FRAME_ALLOCATOR: Option<BitmapFrameAllocator> = None;

/// Initialize physical memory management
pub fn init() {
    // Detect available memory from bootloader or ACPI
    // Use a fixed memory region
    // Memory map would be read from bootloader/ACPI in full implementation
    
    // Assume 4GB of RAM starting at 0x100000 (1MB, after BIOS)
    let memory_start = 0x100000;
    let memory_size = 4 * 1024 * 1024 * 1024; // 4GB
    let frame_size = Size4KiB::SIZE as usize;
    let total_frames = memory_size / frame_size;
    
    // Calculate bitmap size (one bit per frame)
    let bitmap_size = (total_frames + 63) / 64; // Round up to u64 boundary
    let bitmap_start = memory_start;
    
    // Initialize bitmap (all frames free initially)
    unsafe {
        let bitmap_ptr = bitmap_start as *mut u64;
        let bitmap_slice = core::slice::from_raw_parts_mut(bitmap_ptr, bitmap_size);
        
        // Set all bits to 1 (free)
        for word in bitmap_slice.iter_mut() {
            *word = u64::MAX;
        }
        
        // Mark frames used by kernel (first 1MB)
        let kernel_frames = 0x100000 / frame_size;
        for i in 0..kernel_frames {
            let word_idx = i / 64;
            let bit_idx = i % 64;
            if word_idx < bitmap_size {
                bitmap_slice[word_idx] &= !(1u64 << bit_idx);
            }
        }
        
        let first_frame = PhysFrame::containing_address(PhysAddr::new(memory_start));
        
        FRAME_ALLOCATOR = Some(BitmapFrameAllocator {
            bitmap: bitmap_slice,
            total_frames,
            first_frame,
            next_free: Mutex::new(kernel_frames),
        });
    }
}

/// Allocate a physical frame
pub fn allocate_frame() -> Option<PhysFrame<Size4KiB>> {
    unsafe {
        if let Some(ref allocator) = FRAME_ALLOCATOR {
            let mut next = allocator.next_free.lock();
            let start = *next;
            
            // Search for free frame starting from next_free
            for i in 0..allocator.total_frames {
                let frame_idx = (start + i) % allocator.total_frames;
                let word_idx = frame_idx / 64;
                let bit_idx = frame_idx % 64;
                
                if word_idx < allocator.bitmap.len() {
                    let word = allocator.bitmap[word_idx];
                    if (word & (1u64 << bit_idx)) != 0 {
                        // Frame is free - mark as used
                        allocator.bitmap[word_idx] &= !(1u64 << bit_idx);
                        *next = (frame_idx + 1) % allocator.total_frames;
                        
                        // Calculate physical address
                        let frame_addr = allocator.first_frame.start_address() + 
                            (frame_idx * Size4KiB::SIZE as usize);
                        return Some(PhysFrame::containing_address(frame_addr));
                    }
                }
            }
        }
        
        None
    }
}

/// Deallocate a physical frame
pub fn deallocate_frame(frame: PhysFrame<Size4KiB>) {
    unsafe {
        if let Some(ref allocator) = FRAME_ALLOCATOR {
            let frame_addr = frame.start_address();
            let first_addr = allocator.first_frame.start_address();
            
            if frame_addr < first_addr {
                return; // Invalid frame
            }
            
            let offset = (frame_addr - first_addr).as_usize();
            let frame_idx = offset / Size4KiB::SIZE as usize;
            
            if frame_idx < allocator.total_frames {
                let word_idx = frame_idx / 64;
                let bit_idx = frame_idx % 64;
                
                if word_idx < allocator.bitmap.len() {
                    // Mark frame as free
                    allocator.bitmap[word_idx] |= 1u64 << bit_idx;
                    
                    // Update next_free if this frame is before it
                    let mut next = allocator.next_free.lock();
                    if frame_idx < *next {
                        *next = frame_idx;
                    }
                }
            }
        }
    }
}

/// Get number of free frames
pub fn free_frames() -> usize {
    unsafe {
        if let Some(ref allocator) = FRAME_ALLOCATOR {
            let mut count = 0;
            for word in allocator.bitmap.iter() {
                count += word.count_ones() as usize;
            }
            count
        } else {
            0
        }
    }
}

/// Get total number of frames
pub fn total_frames() -> usize {
    unsafe {
        if let Some(ref allocator) = FRAME_ALLOCATOR {
            allocator.total_frames
        } else {
            0
        }
    }
}
