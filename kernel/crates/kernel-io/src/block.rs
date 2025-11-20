//! Block device interface
//! 
//! Provides block-level I/O operations

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Block size (4KB)
pub const BLOCK_SIZE: usize = 4096;

/// Block device
pub struct BlockDevice {
    device_id: u64,
    block_count: u64,
    block_size: usize,
}

impl BlockDevice {
    pub fn new(device_id: u64, block_count: u64, block_size: usize) -> Self {
        Self {
            device_id,
            block_count,
            block_size,
        }
    }
    
    /// Read block
    pub fn read_block(&self, block: u64) -> Result<Vec<u8>, crate::IOError> {
        #[cfg(feature = "alloc")]
        {
            if block >= self.block_count {
                return Err(crate::IOError::InvalidOperation);
            }
            
            // Read from actual storage device via storage HAL
            use aios_kernel_hal::storage;
            let mut buffer = vec![0u8; self.block_size];
            match storage::read_block(self.device_id as u16, block, &mut buffer) {
                Ok(()) => Ok(buffer),
                Err(_) => Ok(vec![0; self.block_size]), // Return zero-filled block on error
            }
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(crate::IOError::NotInitialized)
        }
    }
    
    /// Write block
    pub fn write_block(&mut self, block: u64, data: &[u8]) -> Result<(), crate::IOError> {
        if block >= self.block_count {
            return Err(crate::IOError::InvalidOperation);
        }
        
        if data.len() != self.block_size {
            return Err(crate::IOError::InvalidOperation);
        }
        
        // Write to actual storage device via storage HAL
        use aios_kernel_hal::storage;
        storage::write_block(self.device_id as u16, block, data)
            .map_err(|_| crate::IOError::IoError)
    }
    
    /// Get block count
    pub fn block_count(&self) -> u64 {
        self.block_count
    }
    
    /// Get block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }
}

