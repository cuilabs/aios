//! Kernel memory allocator
//! 
//! Provides dynamic memory allocation for the kernel

use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

/// Simple bump allocator for early kernel use
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: Mutex<usize>,
}

impl BumpAllocator {
    pub const fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            next: Mutex::new(heap_start),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut next = self.next.lock();
        let align = layout.align();
        let size = layout.size();
        
        let addr = (*next + align - 1) & !(align - 1);
        
        if addr + size > self.heap_end {
            return core::ptr::null_mut();
        }
        
        *next = addr + size;
        addr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't support deallocation
    }
}

/// Initialize the kernel allocator
pub fn init() {
    // Allocator initialization
}

