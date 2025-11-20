//! Character device interface
//! 
//! Provides character-level I/O operations

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Character device
pub struct CharDevice {
    device_id: u64,
    buffer: Vec<u8>,
}

impl CharDevice {
    pub fn new(device_id: u64) -> Self {
        Self {
            device_id,
            #[cfg(feature = "alloc")]
            buffer: Vec::new(),
        }
    }
    
    /// Read from device
    pub fn read(&self, len: usize) -> Result<Vec<u8>, crate::IOError> {
        #[cfg(feature = "alloc")]
        {
            let read_len = core::cmp::min(len, self.buffer.len());
            Ok(self.buffer[..read_len].to_vec())
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(crate::IOError::NotInitialized)
        }
    }
    
    /// Write to device
    pub fn write(&mut self, data: &[u8]) -> Result<usize, crate::IOError> {
        #[cfg(feature = "alloc")]
        {
            self.buffer.extend_from_slice(data);
            Ok(data.len())
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(crate::IOError::NotInitialized)
        }
    }
}

