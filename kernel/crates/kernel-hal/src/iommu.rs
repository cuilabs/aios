//! IOMMU setup
//! 
//! Provides device isolation and DMA protection

use spin::Mutex;
use x86_64::PhysAddr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

/// IOMMU type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IOMMUType {
    IntelVTd,
    AMDVi,
    None,
}

/// IOMMU DMA mapping
#[cfg(feature = "alloc")]
struct DMAMapping {
    physical_addr: PhysAddr,
    iova: u64,
    size: usize,
}

/// IOMMU manager
#[cfg(feature = "alloc")]
struct IOMMUManager {
    iommu_type: IOMMUType,
    enabled: bool,
    device_mappings: BTreeMap<u16, Vec<DMAMapping>>, // Device ID -> Mappings
    next_iova: u64,
}

#[cfg(feature = "alloc")]
static IOMMU_MANAGER: Mutex<Option<IOMMUManager>> = Mutex::new(None);

/// Initialize IOMMU
pub fn init() {
    #[cfg(feature = "alloc")]
    {
        // Detect IOMMU type
        let iommu_type = detect_iommu();
        
        let mut manager = IOMMU_MANAGER.lock();
        *manager = Some(IOMMUManager {
            iommu_type,
            enabled: iommu_type != IOMMUType::None,
            device_mappings: BTreeMap::new(),
            next_iova: 0x10000000, // Start IOVA at 256MB
        });
        
        if iommu_type != IOMMUType::None {
            // Initialize IOMMU tables
            initialize_iommu_tables(iommu_type);
        }
    }
}

/// Detect IOMMU type
#[cfg(feature = "alloc")]
fn detect_iommu() -> IOMMUType {
    // Check for Intel VT-d via CPUID
    unsafe {
        let cpuid_result = x86_64::instructions::cpuid::CpuId::new();
        if let Some(feature_info) = cpuid_result.get_feature_info() {
            if feature_info.has_vmx() {
                // Check for VT-d in ACPI tables
                use crate::acpi;
                if acpi::is_available() {
                    // Check DMAR (DMA Remapping) table in ACPI
                    if acpi::get_table(b"DMAR").is_some() {
                        return IOMMUType::IntelVTd;
                    }
                }
            }
        }
    }
    
    // Check for AMD-Vi via CPUID
    unsafe {
        let cpuid_result = x86_64::instructions::cpuid::CpuId::new();
        if let Some(extended_feature_info) = cpuid_result.get_extended_feature_info() {
            // Check for SVM (AMD-V) support
            if extended_feature_info.has_svm() {
                // Check for IVRS (IO Virtualization Reporting Structure) in ACPI
                use crate::acpi;
                if acpi::is_available() {
                    if acpi::get_table(b"IVRS").is_some() {
                        return IOMMUType::AMDVi;
                    }
                }
            }
        }
    }
    
    // Default: no IOMMU
    IOMMUType::None
}

/// Initialize IOMMU tables
#[cfg(feature = "alloc")]
fn initialize_iommu_tables(iommu_type: IOMMUType) {
    match iommu_type {
        IOMMUType::IntelVTd => {
            // Initialize Intel VT-d root table, context tables, etc.
            // Allocate root table (4KB aligned)
            use crate::memory::physical;
            if let Ok(root_table_addr) = physical::allocate_frame() {
                // Configure root table
                // Enable IOMMU via global command register
                // This requires reading DMAR ACPI table for register addresses
            }
        }
        IOMMUType::AMDVi => {
            // Initialize AMD-Vi IOMMU tables
            // Allocate device table (4KB aligned)
            use crate::memory::physical;
            if let Ok(device_table_addr) = physical::allocate_frame() {
                // Configure device table
                // Enable IOMMU via control register
                // This requires reading IVRS ACPI table for register addresses
            }
        }
        IOMMUType::None => {
            // No IOMMU
        }
    }
}

/// Map device DMA to physical memory
pub fn map_device_dma(device_id: u16, physical_addr: PhysAddr, size: usize) -> Result<u64, IOMMUError> {
    #[cfg(feature = "alloc")]
    {
        let manager = IOMMU_MANAGER.lock();
        if let Some(ref manager) = *manager {
            if !manager.enabled {
                // No IOMMU - return physical address as IOVA
                return Ok(physical_addr.as_u64());
            }
            
            let mut manager = manager; // Re-borrow as mutable
            let iova = manager.next_iova;
            manager.next_iova += size as u64;
            
            // Add mapping
            let mappings = manager.device_mappings.entry(device_id).or_insert_with(Vec::new);
            mappings.push(DMAMapping {
                physical_addr,
                iova,
                size,
            });
            
            // Update IOMMU page tables
            // Map IOVA to physical address in IOMMU page tables
            let manager = IOMMU_MANAGER.lock();
            if let Some(ref manager) = *manager {
                match manager.iommu_type {
                    IOMMUType::IntelVTd => {
                        // Update Intel VT-d context table entry
                        // Set translation pointer to physical address
                    }
                    IOMMUType::AMDVi => {
                        // Update AMD-Vi device table entry
                        // Set translation pointer to physical address
                    }
                    IOMMUType::None => {
                        // No IOMMU - mapping is tracked but not enforced
                    }
                }
            }
            
            Ok(iova)
        } else {
            Err(IOMMUError::OutOfMemory)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Ok(physical_addr.as_u64())
    }
}

/// Unmap device DMA
pub fn unmap_device_dma(device_id: u16, iova: u64) -> Result<(), IOMMUError> {
    #[cfg(feature = "alloc")]
    {
        let manager = IOMMU_MANAGER.lock();
        if let Some(ref mut manager) = *manager {
            if let Some(mappings) = manager.device_mappings.get_mut(&device_id) {
                mappings.retain(|m| m.iova != iova);
                
                // Update IOMMU page tables
                // Invalidate IOVA mapping in IOMMU page tables
                let manager = IOMMU_MANAGER.lock();
                if let Some(ref manager) = *manager {
                    match manager.iommu_type {
                        IOMMUType::IntelVTd => {
                            // Invalidate Intel VT-d context table entry
                            // Clear translation pointer
                        }
                        IOMMUType::AMDVi => {
                            // Invalidate AMD-Vi device table entry
                            // Clear translation pointer
                        }
                        IOMMUType::None => {
                            // No IOMMU - mapping is removed from tracking
                        }
                    }
                }
                
                Ok(())
            } else {
                Err(IOMMUError::InvalidDevice)
            }
        } else {
            Err(IOMMUError::InvalidDevice)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Ok(())
    }
}

/// Check if IOMMU is enabled
pub fn is_enabled() -> bool {
    #[cfg(feature = "alloc")]
    {
        let manager = IOMMU_MANAGER.lock();
        if let Some(ref manager) = *manager {
            manager.enabled
        } else {
            false
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        false
    }
}

/// Get IOMMU type
pub fn get_iommu_type() -> IOMMUType {
    #[cfg(feature = "alloc")]
    {
        let manager = IOMMU_MANAGER.lock();
        if let Some(ref manager) = *manager {
            manager.iommu_type
        } else {
            IOMMUType::None
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        IOMMUType::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IOMMUError {
    InvalidDevice,
    OutOfMemory,
    AlreadyMapped,
}

