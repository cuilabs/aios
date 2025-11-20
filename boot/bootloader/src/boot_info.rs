//! Boot information structure for passing to kernel
//! 
//! Must match kernel/crates/kernel-core/src/boot_info.rs structure

/// Boot information magic number
pub const BOOT_INFO_MAGIC: u64 = 0x41494F53; // "AIOS" in ASCII

/// Boot information passed from bootloader to kernel
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

