//! File operations
//! 
//! File read/write operations

use crate::inode::InodeNumber;
use crate::FsError;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// File
pub struct File {
    pub inode_id: InodeNumber,
    pub offset: u64,
    #[cfg(feature = "alloc")]
    data: Vec<u8>,
}

impl File {
    pub fn new(inode_id: InodeNumber) -> Self {
        Self {
            inode_id,
            offset: 0,
            #[cfg(feature = "alloc")]
            data: Vec::new(),
        }
    }
    
    /// Read from file
    pub fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, FsError> {
        #[cfg(feature = "alloc")]
        {
            // Check bounds
            if offset as usize >= self.data.len() {
                return Ok(Vec::new());
            }
            
            let start = offset as usize;
            let end = core::cmp::min(start + len, self.data.len());
            
            Ok(self.data[start..end].to_vec())
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(FsError::NotInitialized)
        }
    }
    
    /// Write to file
    pub fn write(&mut self, offset: u64, data: &[u8]) -> Result<usize, FsError> {
        #[cfg(feature = "alloc")]
        {
            let offset_usize = offset as usize;
            
            // Extend data if necessary
            if offset_usize + data.len() > self.data.len() {
                self.data.resize(offset_usize + data.len(), 0);
            }
            
            // Write data
            self.data[offset_usize..offset_usize + data.len()].copy_from_slice(data);
            
            Ok(data.len())
        }
        
        #[cfg(not(feature = "alloc"))]
        {
            Err(FsError::NotInitialized)
        }
    }
    
    /// Seek to position
    pub fn seek(&mut self, offset: u64) -> Result<u64, FsError> {
        #[cfg(feature = "alloc")]
        {
            if offset as usize > self.data.len() {
                return Err(FsError::InvalidPath);
            }
        }
        self.offset = offset;
        Ok(offset)
    }
}

