//! Directory operations
//! 
//! Directory management and traversal

use crate::inode::{Inode, InodeNumber};
use crate::FsError;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// Directory entry
#[cfg(feature = "alloc")]
pub struct DirectoryEntry {
    pub name: String,
    pub inode: InodeNumber,
}

/// Directory
pub struct Directory {
    pub inode: Inode,
    #[cfg(feature = "alloc")]
    pub entries: alloc::collections::BTreeMap<String, InodeNumber>,
}

impl Directory {
    pub fn new(inode: Inode) -> Self {
        Self {
            inode,
            #[cfg(feature = "alloc")]
            entries: alloc::collections::BTreeMap::new(),
        }
    }
    
    /// Add entry to directory
    #[cfg(feature = "alloc")]
    pub fn add_entry(&mut self, name: String, inode: InodeNumber) -> Result<(), FsError> {
        if self.entries.contains_key(&name) {
            return Err(FsError::InvalidPath);
        }
        self.entries.insert(name, inode);
        Ok(())
    }
    
    /// Remove entry from directory
    #[cfg(feature = "alloc")]
    pub fn remove_entry(&mut self, name: &str) -> Result<(), FsError> {
        self.entries.remove(name).ok_or(FsError::NotFound)?;
        Ok(())
    }
    
    /// Look up entry by name
    #[cfg(feature = "alloc")]
    pub fn lookup(&self, name: &str) -> Option<InodeNumber> {
        self.entries.get(name).copied()
    }
}

