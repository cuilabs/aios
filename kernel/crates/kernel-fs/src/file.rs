//! File operations
//! 
//! File read/write operations

use crate::inode::Inode;
use crate::FsError;

/// File
pub struct File {
    pub inode: Inode,
    pub offset: u64,
}

impl File {
    pub fn new(inode: Inode) -> Self {
        Self {
            inode,
            offset: 0,
        }
    }
    
    /// Read from file
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, FsError> {
        // TODO: Implement actual file reading
        Ok(0)
    }
    
    /// Write to file
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, FsError> {
        // TODO: Implement actual file writing
        Ok(0)
    }
    
    /// Seek to position
    pub fn seek(&mut self, offset: u64) -> Result<u64, FsError> {
        if offset > self.inode.size {
            return Err(FsError::InvalidPath);
        }
        self.offset = offset;
        Ok(offset)
    }
}

