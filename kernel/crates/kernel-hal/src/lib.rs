#![no_std]

//! Hardware Abstraction Layer (HAL)
//! 
//! Provides portable hardware interfaces for:
//! - APIC (Advanced Programmable Interrupt Controller)
//! - Timer (HPET/TSC)
//! - PCIe enumeration
//! - IOMMU setup
//! - ACPI
//! - SATA/NVMe drivers
//! - Basic NIC drivers

pub mod apic;
pub mod timer;
pub mod pcie;
pub mod iommu;
pub mod acpi;
pub mod storage;
pub mod network;
#[cfg(feature = "alloc")]
pub mod gpu;
#[cfg(feature = "alloc")]
pub mod graphics;
#[cfg(feature = "alloc")]
pub mod input;
#[cfg(feature = "alloc")]
pub mod edid;

/// Initialize hardware abstraction layer
pub fn init() {
    acpi::init();
    apic::init();
    timer::init();
    pcie::init();
    iommu::init();
    storage::init();
    network::init();
    #[cfg(feature = "alloc")]
    {
        graphics::init();
        input::init();
    }
    #[cfg(feature = "alloc")]
    gpu::init();
}

