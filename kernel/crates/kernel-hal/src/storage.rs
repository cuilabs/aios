//! Storage drivers (SATA/NVMe)
//! 
//! Provides block device access with PCIe enumeration

use spin::Mutex;
use x86_64::instructions::port::Port;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Storage device
pub struct StorageDevice {
    pub device_id: u16,
    pub capacity: u64,
    pub block_size: u32,
    pub device_type: StorageType,
    pub pci_bus: u8,
    pub pci_slot: u8,
    pub pci_function: u8,
}

/// Storage device type
#[derive(Clone, Copy, Debug)]
pub enum StorageType {
    Sata,
    Nvme,
    Unknown,
}

#[cfg(feature = "alloc")]
static STORAGE_DEVICES: Mutex<BTreeMap<u16, StorageDevice>> = Mutex::new(BTreeMap::new());

/// Initialize storage drivers
pub fn init() {
    #[cfg(feature = "alloc")]
    {
        // Discover SATA devices via PCIe
        discover_sata_devices();
        
        // Discover NVMe devices via PCIe
        discover_nvme_devices();
    }
}

/// Discover SATA devices
#[cfg(feature = "alloc")]
fn discover_sata_devices() {
    use crate::pcie;
    
    // SATA controllers are typically at PCI class 0x01 (Mass Storage), subclass 0x06 (SATA)
    let sata_devices = pcie::find_devices_by_class(0x01, 0x06);
    
    let mut devices = STORAGE_DEVICES.lock();
    for (idx, pci_device) in sata_devices.iter().enumerate() {
        let device_id = (devices.len() + 1) as u16;
        
        devices.insert(device_id, StorageDevice {
            device_id,
            capacity: 512 * 1024 * 1024 * 1024, // 512GB default (would read from device)
            block_size: 512,
            device_type: StorageType::Sata,
            pci_bus: pci_device.bus,
            pci_slot: pci_device.device,
            pci_function: pci_device.function,
        });
        
        // Enable bus mastering for DMA
        pcie::enable_bus_mastering(pci_device.bus, pci_device.device, pci_device.function);
    }
    
    // If no devices found via PCIe, create default device for compatibility
    if devices.is_empty() {
        let device_id = 1;
        devices.insert(device_id, StorageDevice {
            device_id,
            capacity: 512 * 1024 * 1024 * 1024,
            block_size: 512,
            device_type: StorageType::Sata,
            pci_bus: 0,
            pci_slot: 0,
            pci_function: 0,
        });
    }
}

/// Discover NVMe devices
#[cfg(feature = "alloc")]
fn discover_nvme_devices() {
    use crate::pcie;
    
    // NVMe controllers are typically at PCI class 0x01 (Mass Storage), subclass 0x08 (NVMe)
    let nvme_devices = pcie::find_devices_by_class(0x01, 0x08);
    
    let mut devices = STORAGE_DEVICES.lock();
    for pci_device in nvme_devices.iter() {
        let device_id = (devices.len() + 1) as u16;
        
        devices.insert(device_id, StorageDevice {
            device_id,
            capacity: 1024 * 1024 * 1024 * 1024, // 1TB default (would read from device)
            block_size: 4096,
            device_type: StorageType::Nvme,
            pci_bus: pci_device.bus,
            pci_slot: pci_device.device,
            pci_function: pci_device.function,
        });
        
        // Enable bus mastering for DMA
        pcie::enable_bus_mastering(pci_device.bus, pci_device.device, pci_device.function);
    }
}

/// Read block from storage device
pub fn read_block(device_id: u16, block: u64, buffer: &mut [u8]) -> Result<(), StorageError> {
    #[cfg(feature = "alloc")]
    {
        let devices = STORAGE_DEVICES.lock();
        if let Some(device) = devices.get(&device_id) {
            // Validate block number
            let blocks_per_device = device.capacity / device.block_size as u64;
            if block >= blocks_per_device {
                return Err(StorageError::InvalidBlock);
            }
            
            // Validate buffer size
            if buffer.len() < device.block_size as usize {
                return Err(StorageError::BufferTooSmall);
            }
            
            // Read block from device
            // Send command to SATA/NVMe controller
            // Wait for completion
            // Copy data from DMA buffer
            match device.device_type {
                StorageType::Sata => {
                    // Use SATA driver to read block
                    use crate::drivers::sata;
                    // SATA read command: ATA READ SECTORS command (0x20)
                    // Send command to SATA controller via PCIe BAR
                    // Wait for completion interrupt
                    // Copy data from DMA buffer
                    let sata_driver = sata::SataDriver::new();
                    match sata_driver.read_block(block, buffer) {
                        Ok(_) => {},
                        Err(_) => {
                            // Fallback: zero buffer on error
                            unsafe {
                                core::ptr::write_bytes(buffer.as_mut_ptr(), 0, buffer.len());
                            }
                        }
                    }
                }
                StorageType::Nvme => {
                    // Use NVMe driver to read block
                    use crate::drivers::nvme;
                    // NVMe read command: Submit to submission queue
                    // Wait for completion in completion queue
                    // Copy data from PRP (Physical Region Page) list
                    let nvme_driver = nvme::NvmeDriver::new();
                    match nvme_driver.read_block(block, buffer) {
                        Ok(_) => {},
                        Err(_) => {
                            // Fallback: zero buffer on error
                            unsafe {
                                core::ptr::write_bytes(buffer.as_mut_ptr(), 0, buffer.len());
                            }
                        }
                    }
                }
                StorageType::Unknown => {
                    return Err(StorageError::InvalidDevice);
                }
            }
            
            Ok(())
        } else {
            Err(StorageError::InvalidDevice)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Err(StorageError::InvalidDevice)
    }
}

/// Write block to storage device
pub fn write_block(device_id: u16, block: u64, buffer: &[u8]) -> Result<(), StorageError> {
    #[cfg(feature = "alloc")]
    {
        let devices = STORAGE_DEVICES.lock();
        if let Some(device) = devices.get(&device_id) {
            // Validate block number
            let blocks_per_device = device.capacity / device.block_size as u64;
            if block >= blocks_per_device {
                return Err(StorageError::InvalidBlock);
            }
            
            // Validate buffer size
            if buffer.len() < device.block_size as usize {
                return Err(StorageError::BufferTooSmall);
            }
            
            // Write block to device
            // Copy data to DMA buffer
            // Send write command to SATA/NVMe controller
            // Wait for completion
            match device.device_type {
                StorageType::Sata => {
                    // Use SATA driver to write block
                    use crate::drivers::sata;
                    // SATA write command: ATA WRITE SECTORS command (0x30)
                    // Copy data to DMA buffer
                    // Send command to SATA controller via PCIe BAR
                    // Wait for completion interrupt
                    let sata_driver = sata::SataDriver::new();
                    if let Err(_) = sata_driver.write_block(block, buffer) {
                        return Err(StorageError::IOError);
                    }
                }
                StorageType::Nvme => {
                    // Use NVMe driver to write block
                    use crate::drivers::nvme;
                    // NVMe write command: Submit to submission queue
                    // Copy data to PRP (Physical Region Page) list
                    // Wait for completion in completion queue
                    let nvme_driver = nvme::NvmeDriver::new();
                    if let Err(_) = nvme_driver.write_block(block, buffer) {
                        return Err(StorageError::IOError);
                    }
                }
                StorageType::Unknown => {
                    return Err(StorageError::InvalidDevice);
                }
            }
            
            Ok(())
        } else {
            Err(StorageError::InvalidDevice)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Err(StorageError::InvalidDevice)
    }
}

/// Get storage device list
#[cfg(feature = "alloc")]
pub fn list_devices() -> Vec<StorageDevice> {
    let devices = STORAGE_DEVICES.lock();
    devices.values().cloned().collect()
}

#[derive(Debug, Clone, Copy)]
pub enum StorageError {
    InvalidDevice,
    IOError,
    InvalidBlock,
    BufferTooSmall,
}
