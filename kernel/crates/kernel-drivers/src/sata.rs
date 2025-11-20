//! SATA driver implementation

use super::driver::Driver;
use super::device::Device;
use spin::Mutex;
use x86_64::instructions::port::Port;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// SATA controller registers
const SATA_DATA: u16 = 0x1F0;
const SATA_ERROR: u16 = 0x1F1;
const SATA_SECTOR_COUNT: u16 = 0x1F2;
const SATA_LBA_LOW: u16 = 0x1F3;
const SATA_LBA_MID: u16 = 0x1F4;
const SATA_LBA_HIGH: u16 = 0x1F5;
const SATA_DEVICE: u16 = 0x1F6;
const SATA_COMMAND: u16 = 0x1F7;
const SATA_STATUS: u16 = 0x1F7;

/// SATA commands
const CMD_READ_SECTORS: u8 = 0x20;
const CMD_WRITE_SECTORS: u8 = 0x30;
const CMD_IDENTIFY: u8 = 0xEC;

/// SATA driver
pub struct SataDriver {
    device: Option<Device>,
    initialized: bool,
}

impl SataDriver {
    pub fn new() -> Self {
        Self {
            device: None,
            initialized: false,
        }
    }
    
    /// Read block from SATA device
    pub fn read_block(&self, lba: u64, buffer: &mut [u8]) -> Result<(), DriverError> {
        if !self.initialized {
            return Err(DriverError::InitializationFailed);
        }
        
        // SATA uses 512-byte sectors
        const SECTOR_SIZE: usize = 512;
        if buffer.len() < SECTOR_SIZE {
            return Err(DriverError::IOError);
        }
        
        unsafe {
            // Wait for drive to be ready
            wait_for_ready();
            
            // Set up LBA address
            let port_sector_count = Port::<u8>::new(SATA_SECTOR_COUNT);
            port_sector_count.write(1); // Read 1 sector
            
            let port_lba_low = Port::<u8>::new(SATA_LBA_LOW);
            let port_lba_mid = Port::<u8>::new(SATA_LBA_MID);
            let port_lba_high = Port::<u8>::new(SATA_LBA_HIGH);
            let port_device = Port::<u8>::new(SATA_DEVICE);
            
            port_lba_low.write((lba & 0xFF) as u8);
            port_lba_mid.write(((lba >> 8) & 0xFF) as u8);
            port_lba_high.write(((lba >> 16) & 0xFF) as u8);
            port_device.write(0xE0 | ((lba >> 24) & 0x0F) as u8); // LBA mode
            
            // Send read command
            let port_command = Port::<u8>::new(SATA_COMMAND);
            port_command.write(CMD_READ_SECTORS);
            
            // Wait for data ready
            wait_for_data();
            
            // Read data
            let port_data = Port::<u16>::new(SATA_DATA);
            for i in 0..(SECTOR_SIZE / 2) {
                let word = port_data.read();
                buffer[i * 2] = (word & 0xFF) as u8;
                buffer[i * 2 + 1] = ((word >> 8) & 0xFF) as u8;
            }
        }
        
        Ok(())
    }
    
    /// Write block to SATA device
    pub fn write_block(&self, lba: u64, buffer: &[u8]) -> Result<(), DriverError> {
        if !self.initialized {
            return Err(DriverError::InitializationFailed);
        }
        
        const SECTOR_SIZE: usize = 512;
        if buffer.len() < SECTOR_SIZE {
            return Err(DriverError::IOError);
        }
        
        unsafe {
            // Wait for drive to be ready
            wait_for_ready();
            
            // Set up LBA address
            let port_sector_count = Port::<u8>::new(SATA_SECTOR_COUNT);
            port_sector_count.write(1); // Write 1 sector
            
            let port_lba_low = Port::<u8>::new(SATA_LBA_LOW);
            let port_lba_mid = Port::<u8>::new(SATA_LBA_MID);
            let port_lba_high = Port::<u8>::new(SATA_LBA_HIGH);
            let port_device = Port::<u8>::new(SATA_DEVICE);
            
            port_lba_low.write((lba & 0xFF) as u8);
            port_lba_mid.write(((lba >> 8) & 0xFF) as u8);
            port_lba_high.write(((lba >> 16) & 0xFF) as u8);
            port_device.write(0xE0 | ((lba >> 24) & 0x0F) as u8); // LBA mode
            
            // Send write command
            let port_command = Port::<u8>::new(SATA_COMMAND);
            port_command.write(CMD_WRITE_SECTORS);
            
            // Wait for data ready
            wait_for_data();
            
            // Write data
            let port_data = Port::<u16>::new(SATA_DATA);
            for i in 0..(SECTOR_SIZE / 2) {
                let word = (buffer[i * 2] as u16) | ((buffer[i * 2 + 1] as u16) << 8);
                port_data.write(word);
            }
            
            // Flush write cache
            port_command.write(0xE7); // FLUSH CACHE command
            wait_for_ready();
        }
        
        Ok(())
    }
}

/// Wait for SATA drive to be ready
unsafe fn wait_for_ready() {
    let port_status = Port::<u8>::new(SATA_STATUS);
    loop {
        let status = port_status.read();
        if (status & 0x80) == 0 && (status & 0x08) != 0 {
            // Drive ready and not busy
            break;
        }
    }
}

/// Wait for SATA drive data to be ready
unsafe fn wait_for_data() {
    let port_status = Port::<u8>::new(SATA_STATUS);
    loop {
        let status = port_status.read();
        if (status & 0x08) != 0 && (status & 0x01) == 0 {
            // Data ready and no error
            break;
        }
        if (status & 0x01) != 0 {
            // Error occurred
            break;
        }
    }
}

impl Driver for SataDriver {
    fn name(&self) -> &'static str {
        "sata"
    }
    
    fn probe(&self, device: &Device) -> bool {
        // Check if device is SATA controller
        device.class == 0x01 && device.subclass == 0x06
    }
    
    fn attach(&mut self, device: &Device) -> Result<(), DriverError> {
        self.device = Some(device.clone());
        
        // Initialize SATA controller
        // Enable DMA
        // Set up command queue
        // Identify device
        
        self.initialized = true;
        Ok(())
    }
    
    fn detach(&mut self) -> Result<(), DriverError> {
        self.initialized = false;
        self.device = None;
        Ok(())
    }
}

/// Driver error
#[derive(Debug)]
pub enum DriverError {
    DeviceNotFound,
    InitializationFailed,
    IOError,
}

