//! PCIe enumeration and device discovery
//! 
//! Discovers and initializes PCIe devices

#[cfg(feature = "alloc")]
extern crate alloc;

use spin::Mutex;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// PCIe device
#[derive(Clone, Debug)]
pub struct PCIeDevice {
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
}

/// PCIe configuration space base address
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// PCIe device manager
#[cfg(feature = "alloc")]
struct PCIeManager {
    devices: BTreeMap<u32, PCIeDevice>, // BDF (Bus:Device:Function) -> Device
}

#[cfg(feature = "alloc")]
static PCIE_MANAGER: Mutex<Option<PCIeManager>> = Mutex::new(None);

/// Initialize PCIe enumeration
pub fn init() {
    #[cfg(feature = "alloc")]
    {
        let mut manager = PCIE_MANAGER.lock();
        *manager = Some(PCIeManager {
            devices: BTreeMap::new(),
        });
        
        // Enumerate all PCIe devices
        enumerate_all_devices();
    }
}

/// Enumerate all PCIe devices
#[cfg(feature = "alloc")]
fn enumerate_all_devices() {
    let mut devices_list = BTreeMap::new();
    
    // Scan all buses (0-255), devices (0-31), functions (0-7)
    for bus in 0..=255 {
        for device in 0..=31 {
            for function in 0..=7 {
                let vendor_id = read_config(bus, device, function, 0) as u16;
                
                // Check if device exists (vendor ID 0xFFFF means no device)
                if vendor_id == 0xFFFF {
                    continue;
                }
                
                let device_id = (read_config(bus, device, function, 0) >> 16) as u16;
                let class_subclass = read_config(bus, device, function, 0x08);
                let class = ((class_subclass >> 24) & 0xFF) as u8;
                let subclass = ((class_subclass >> 16) & 0xFF) as u8;
                let header_type = ((read_config(bus, device, function, 0x0C) >> 16) & 0xFF) as u8;
                let interrupt_line = (read_config(bus, device, function, 0x3C) & 0xFF) as u8;
                let interrupt_pin = ((read_config(bus, device, function, 0x3C) >> 8) & 0xFF) as u8;
                
                let bdf = ((bus as u32) << 16) | ((device as u32) << 11) | ((function as u32) << 8);
                
                devices_list.insert(bdf, PCIeDevice {
                    vendor_id,
                    device_id,
                    class,
                    subclass,
                    bus,
                    device,
                    function,
                    header_type,
                    interrupt_line,
                    interrupt_pin,
                });
                
                // If this is not a multi-function device, skip remaining functions
                if function == 0 && (header_type & 0x80) == 0 {
                    break;
                }
            }
        }
    }
    
    // Store devices in manager
    let manager = PCIE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut devices = manager.devices.lock();
        *devices = devices_list;
    }
}

/// Enumerate all PCIe devices
#[cfg(feature = "alloc")]
pub fn enumerate_devices() -> Vec<PCIeDevice> {
    let manager = PCIE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let devices = manager.devices.lock();
        devices.values().cloned().collect()
    } else {
        Vec::new()
    }
}

#[cfg(not(feature = "alloc"))]
pub fn enumerate_devices() -> &'static [PCIeDevice] {
    &[]
}

/// Find devices by class
#[cfg(feature = "alloc")]
pub fn find_devices_by_class(class: u8, subclass: u8) -> Vec<PCIeDevice> {
    let manager = PCIE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let devices = manager.devices.lock();
        devices.values()
            .filter(|d| d.class == class && d.subclass == subclass)
            .cloned()
            .collect()
    } else {
        Vec::new()
    }
}

/// Read PCIe configuration register
pub fn read_config(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    use x86_64::instructions::port::Port;
    
    // Build configuration address
    let address = 0x80000000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    
    unsafe {
        let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        
        addr_port.write(address);
        data_port.read()
    }
}

/// Write PCIe configuration register
pub fn write_config(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    use x86_64::instructions::port::Port;
    
    // Build configuration address
    let address = 0x80000000u32
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    
    unsafe {
        let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        
        addr_port.write(address);
        data_port.write(value);
    }
}

/// Enable bus mastering for device
pub fn enable_bus_mastering(bus: u8, device: u8, function: u8) {
    let command = read_config(bus, device, function, 0x04);
    let new_command = command | 0x0004; // Set bit 2 (Bus Master Enable)
    write_config(bus, device, function, 0x04, new_command);
}

/// Get device BAR (Base Address Register)
pub fn get_bar(bus: u8, device: u8, function: u8, bar_index: u8) -> Option<u64> {
    if bar_index >= 6 {
        return None;
    }
    
    let offset = 0x10 + (bar_index as u8 * 4);
    let bar = read_config(bus, device, function, offset);
    
    // Check if 64-bit BAR
    if (bar & 0x07) == 0x04 {
        // 64-bit BAR - read next register
        let bar_high = read_config(bus, device, function, offset + 4);
        Some(((bar_high as u64) << 32) | ((bar & 0xFFFFFFF0) as u64))
    } else {
        // 32-bit BAR
        Some((bar & 0xFFFFFFF0) as u64)
    }
}

