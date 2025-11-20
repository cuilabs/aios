//! Boot information structure
//! 
//! Minimal boot information passed from bootloader to kernel
//! No external dependencies - pure kernel code

/// Boot information magic number (must match bootloader)
pub const BOOT_INFO_MAGIC: u64 = 0x41494F53; // "AIOS" in ASCII

/// Boot information passed from bootloader
#[repr(C)]
pub struct BootInfo {
    /// Magic number (must be BOOT_INFO_MAGIC)
    pub magic: u64,
    /// Version (1)
    pub version: u32,
    /// Initramfs base address
    pub initramfs_base: u64,
    /// Initramfs size
    pub initramfs_size: u64,
    /// Memory map base address
    pub memory_map_base: u64,
    /// Memory map size
    pub memory_map_size: u64,
    /// ACPI RSDP address
    pub acpi_rsdp: u64,
}

/// Get boot information (provided by bootloader)
/// This is set by the bootloader before jumping to kernel
static mut BOOT_INFO: Option<&'static BootInfo> = None;

/// Set boot information (called by bootloader or kernel entry)
pub unsafe fn set_boot_info(info: &'static BootInfo) {
    // Verify magic number
    if info.magic != BOOT_INFO_MAGIC {
        // Invalid boot info - panic
        loop {}
    }
    BOOT_INFO = Some(info);
}

/// Get boot information
pub fn get_boot_info() -> Option<&'static BootInfo> {
    unsafe { BOOT_INFO }
}

