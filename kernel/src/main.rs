//! AIOS Kernel Entry Point
//! 
//! This is the main kernel binary that gets loaded by the bootloader

#![no_std]
#![no_main]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(lang_items)]

#[cfg(feature = "alloc")]
extern crate alloc;

use aios_kernel_core::boot;
use aios_kernel_core::boot_info;

/// Kernel entry point
/// Called by bootloader after kernel is loaded
/// 
/// Bootloader passes boot information via register or memory location
/// For Multiboot: EBX contains multiboot info structure address
/// For UEFI: Boot info is passed via memory location
#[no_mangle]
pub extern "C" fn _start(boot_info_ptr: *const aios_kernel_core::boot_info::BootInfo) -> ! {
    // Receive boot information from bootloader
    if !boot_info_ptr.is_null() {
        unsafe {
            let boot_info = &*boot_info_ptr;
            aios_kernel_core::boot_info::set_boot_info(boot_info);
        }
    }
    
    // Initialize kernel
    boot::init();
    
    // Kernel main loop
    loop {
        // Handle interrupts, schedule agents, etc.
        unsafe {
            x86_64::instructions::hlt(); // Halt until interrupt
        }
    }
}

/// Panic handler
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Use kernel console for logging
    loop {}
}

/// Language items
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

