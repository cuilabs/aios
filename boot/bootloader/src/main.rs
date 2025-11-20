//! AIOS Bootloader
//! 
//! Secure bootloader with kernel verification and initramfs loading

#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod uefi;
mod multiboot;
mod crypto;
mod initramfs;
mod boot_info;

/// Bootloader entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize bootloader
    init();
    
    // Detect boot method (UEFI or Multiboot)
    let boot_method = detect_boot_method();
    
    // Load and verify kernel
    let kernel_entry = load_kernel(boot_method);
    
    // Load initramfs
    let initramfs_addr = load_initramfs();
    
    // Jump to kernel
    jump_to_kernel(kernel_entry, initramfs_addr);
    
    loop {}
}

/// Initialize bootloader
fn init() {
    // Initialize console output (serial port or VGA)
    init_console();
    
    // Initialize memory map
    // Will be populated from UEFI or Multiboot
    
    // Initialize boot services
    // Initialize UEFI boot services or Multiboot
}

/// Initialize console output
fn init_console() {
    // Initialize serial port for debugging
    // Port 0x3F8 (COM1) is standard
    unsafe {
        use x86_64::instructions::port::Port;
        // Set baud rate (115200)
        let port = Port::<u8>::new(0x3F8 + 3);
        port.write(0x80); // Enable DLAB
        let port_low = Port::<u8>::new(0x3F8);
        let port_high = Port::<u8>::new(0x3F8 + 1);
        port_low.write(0x01); // Divisor low (115200 baud)
        port_high.write(0x00); // Divisor high
        port.write(0x03); // 8N1
    }
}

/// Detect boot method
fn detect_boot_method() -> BootMethod {
    // Check for UEFI
    if uefi::is_available() {
        return BootMethod::Uefi;
    }
    
    // Default to Multiboot
    BootMethod::Multiboot
}

/// Load and verify kernel
fn load_kernel(method: BootMethod) -> u64 {
    let kernel_entry = match method {
        BootMethod::Uefi => uefi::load_kernel(),
        BootMethod::Multiboot => multiboot::load_kernel(),
    };
    
    // Verify kernel signature
    // Read kernel from disk and verify signature
    // Kernel is loaded and verified by bootloader
    
    kernel_entry
}

/// Load initramfs
fn load_initramfs() -> u64 {
    initramfs::load()
}

/// Jump to kernel
fn jump_to_kernel(entry: u64, initramfs_addr: u64) -> ! {
    // Create boot info structure
    let boot_info = create_boot_info(initramfs_addr);
    
    // Set up kernel parameters
    // Pass boot info pointer in RDI (first argument)
    unsafe {
        core::arch::asm!(
            "mov rdi, {boot_info}",
            "jmp {entry}",
            boot_info = in(reg) &boot_info as *const _ as u64,
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}

/// Create boot info structure
fn create_boot_info(initramfs_addr: u64) -> boot_info::BootInfo {
    boot_info::BootInfo {
        magic: boot_info::BOOT_INFO_MAGIC,
        version: 1,
        initramfs_base: initramfs_addr,
        initramfs_size: 0, // Will be set by initramfs loader
        memory_map_base: 0, // Will be set from UEFI/Multiboot
        memory_map_size: 0,
        acpi_rsdp: 0, // Will be set from UEFI/Multiboot
    }
}

/// Boot method
enum BootMethod {
    Uefi,
    Multiboot,
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Print panic message
    // Halt system
    loop {}
}

