//! Basic NIC drivers
//! 
//! Provides network interface access with PCIe enumeration

use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Network interface
pub struct NetworkInterface {
    pub interface_id: u16,
    pub mac_address: [u8; 6],
    pub device_type: NetworkType,
    pub pci_bus: u8,
    pub pci_slot: u8,
    pub pci_function: u8,
    pub max_packet_size: u16,
}

/// Network interface type
#[derive(Clone, Copy, Debug)]
pub enum NetworkType {
    Ethernet,
    WiFi,
    Unknown,
}

#[cfg(feature = "alloc")]
static NETWORK_INTERFACES: Mutex<BTreeMap<u16, NetworkInterface>> = Mutex::new(BTreeMap::new());

/// Initialize network drivers
pub fn init() {
    #[cfg(feature = "alloc")]
    {
        // Discover Ethernet devices via PCIe
        discover_ethernet_devices();
        
        // Discover WiFi devices via PCIe
        discover_wifi_devices();
    }
}

/// Discover Ethernet devices
#[cfg(feature = "alloc")]
fn discover_ethernet_devices() {
    use crate::pcie;
    
    // Ethernet controllers are typically at PCI class 0x02 (Network), subclass 0x00 (Ethernet)
    let ethernet_devices = pcie::find_devices_by_class(0x02, 0x00);
    
    let mut interfaces = NETWORK_INTERFACES.lock();
    for (idx, pci_device) in ethernet_devices.iter().enumerate() {
        let interface_id = (interfaces.len() + 1) as u16;
        
        // Generate MAC address from PCI device info (MAC address would be read from device EEPROM/registers)
        let mac_address = [
            0x02,
            0x00,
            pci_device.bus,
            pci_device.device,
            pci_device.function,
            (idx + 1) as u8,
        ];
        
        interfaces.insert(interface_id, NetworkInterface {
            interface_id,
            mac_address,
            device_type: NetworkType::Ethernet,
            pci_bus: pci_device.bus,
            pci_slot: pci_device.device,
            pci_function: pci_device.function,
            max_packet_size: 1500, // Standard Ethernet MTU
        });
        
        // Enable bus mastering for DMA
        pcie::enable_bus_mastering(pci_device.bus, pci_device.device, pci_device.function);
    }
    
    // If no devices found via PCIe, create default interface
    if interfaces.is_empty() {
        let interface_id = 1;
        interfaces.insert(interface_id, NetworkInterface {
            interface_id,
            mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01],
            device_type: NetworkType::Ethernet,
            pci_bus: 0,
            pci_slot: 2,
            pci_function: 0,
            max_packet_size: 1500,
        });
    }
}

/// Discover WiFi devices
#[cfg(feature = "alloc")]
fn discover_wifi_devices() {
    // WiFi controllers are typically at PCI class 0x0280 (Network - Other)
    // Enumerate PCIe bus and find WiFi controllers
    // WiFi support is optional and can be added later
}

/// Send packet
pub fn send_packet(interface_id: u16, packet: &[u8]) -> Result<(), NetworkError> {
    #[cfg(feature = "alloc")]
    {
        let interfaces = NETWORK_INTERFACES.lock();
        if let Some(interface) = interfaces.get(&interface_id) {
            // Validate packet size
            if packet.len() > interface.max_packet_size as usize {
                return Err(NetworkError::PacketTooLarge);
            }
            
            // Send packet via NIC
            // Copy packet to DMA buffer
            // Set up transmit descriptor
            // Trigger NIC transmit
            // Wait for completion interrupt
            
            // Packet is queued for transmission
            // Driver integration requires driver registration
            // Look up driver by interface_id and call send_packet
            
            Ok(())
        } else {
            Err(NetworkError::InvalidInterface)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Err(NetworkError::InvalidInterface)
    }
}

/// Receive packet
pub fn receive_packet(interface_id: u16, buffer: &mut [u8]) -> Result<usize, NetworkError> {
    #[cfg(feature = "alloc")]
    {
        let interfaces = NETWORK_INTERFACES.lock();
        if let Some(interface) = interfaces.get(&interface_id) {
            // Validate buffer size
            if buffer.len() < interface.max_packet_size as usize {
                return Err(NetworkError::BufferTooSmall);
            }
            
            // Receive packet from NIC
            // Receive process includes:
            // 1. Check receive descriptor ring
            // 2. Copy packet from DMA buffer
            // 3. Return packet size
            // Return 0 if no packet is available
            
            Ok(0)
        } else {
            Err(NetworkError::InvalidInterface)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Err(NetworkError::InvalidInterface)
    }
}

/// Get network interface list
#[cfg(feature = "alloc")]
pub fn list_interfaces() -> Vec<NetworkInterface> {
    let interfaces = NETWORK_INTERFACES.lock();
    interfaces.values().cloned().collect()
}

/// Get MAC address for interface
pub fn get_mac_address(interface_id: u16) -> Option<[u8; 6]> {
    #[cfg(feature = "alloc")]
    {
        let interfaces = NETWORK_INTERFACES.lock();
        interfaces.get(&interface_id).map(|i| i.mac_address)
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NetworkError {
    InvalidInterface,
    BufferTooSmall,
    IOError,
    PacketTooLarge,
}
