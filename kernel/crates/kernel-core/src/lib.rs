#![no_std]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(lang_items)]

//! AIOS Kernel Core
//! 
//! The base kernel for AIOS - an AI-native operating system where
//! agents are first-class citizens, not traditional processes.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod boot;
pub mod memory;
pub mod interrupts;
pub mod exceptions;
pub mod error;
pub mod sync;
pub mod syscall;
pub mod log;
pub mod time;

use core::panic::PanicInfo;

/// Kernel panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use crate::sync::println;
    println!("Kernel panic: {}", info);
    loop {}
}

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // This will be called by the bootloader
    boot::init();
    loop {}
}

