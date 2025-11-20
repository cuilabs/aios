//! Snapshot support
//! 
//! Provides copy-on-write snapshots for filesystem state

use crate::inode::InodeNumber;
use crate::FsError;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use spin::Mutex;

/// Snapshot metadata
#[cfg(feature = "alloc")]
pub struct Snapshot {
    pub id: u64,
    pub name: alloc::string::String,
    pub timestamp: u64,
    pub root_inode: InodeNumber,
    pub inode_map: BTreeMap<InodeNumber, InodeNumber>, // Original -> Snapshot inode
}

/// Snapshot manager
pub struct SnapshotManager {
    #[cfg(feature = "alloc")]
    snapshots: Mutex<BTreeMap<u64, Snapshot>>,
    #[cfg(feature = "alloc")]
    next_snapshot_id: Mutex<u64>,
    #[cfg(not(feature = "alloc"))]
    snapshots: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_snapshot_id: Mutex<()>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            snapshots: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            next_snapshot_id: Mutex::new(1),
            #[cfg(not(feature = "alloc"))]
            snapshots: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            next_snapshot_id: Mutex::new(()),
        }
    }

    /// Create snapshot
    #[cfg(feature = "alloc")]
    pub fn create_snapshot(&self, name: alloc::string::String, root_inode: InodeNumber, timestamp: u64) -> u64 {
        let mut next_id = self.next_snapshot_id.lock();
        let id = *next_id;
        *next_id += 1;
        
        let mut snapshots = self.snapshots.lock();
        snapshots.insert(id, Snapshot {
            id,
            name,
            timestamp,
            root_inode,
            inode_map: BTreeMap::new(),
        });
        
        id
    }

    #[cfg(not(feature = "alloc"))]
    pub fn create_snapshot(&self, _name: &str, _root_inode: InodeNumber, _timestamp: u64) -> u64 {
        0
    }

    /// Get snapshot
    #[cfg(feature = "alloc")]
    pub fn get_snapshot(&self, snapshot_id: u64) -> Option<Snapshot> {
        let snapshots = self.snapshots.lock();
        snapshots.get(&snapshot_id).cloned()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_snapshot(&self, _snapshot_id: u64) -> Option<Snapshot> {
        None
    }

    /// Delete snapshot
    #[cfg(feature = "alloc")]
    pub fn delete_snapshot(&self, snapshot_id: u64) -> Result<(), FsError> {
        let mut snapshots = self.snapshots.lock();
        if snapshots.remove(&snapshot_id).is_some() {
            Ok(())
        } else {
            Err(FsError::NotFound)
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn delete_snapshot(&self, _snapshot_id: u64) -> Result<(), FsError> {
        Err(FsError::NotInitialized)
    }

    /// List snapshots
    #[cfg(feature = "alloc")]
    pub fn list_snapshots(&self) -> Vec<(u64, alloc::string::String, u64)> {
        let snapshots = self.snapshots.lock();
        snapshots.iter()
            .map(|(id, snapshot)| (*id, snapshot.name.clone(), snapshot.timestamp))
            .collect()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn list_snapshots(&self) -> Vec<(u64, alloc::string::String, u64)> {
        Vec::new()
    }

    /// Map inode from snapshot
    #[cfg(feature = "alloc")]
    pub fn map_inode(&self, snapshot_id: u64, original_inode: InodeNumber) -> Option<InodeNumber> {
        let snapshots = self.snapshots.lock();
        snapshots.get(&snapshot_id)
            .and_then(|snapshot| snapshot.inode_map.get(&original_inode).copied())
    }

    #[cfg(not(feature = "alloc"))]
    pub fn map_inode(&self, _snapshot_id: u64, _original_inode: InodeNumber) -> Option<InodeNumber> {
        None
    }

    /// Add inode mapping to snapshot
    #[cfg(feature = "alloc")]
    pub fn add_inode_mapping(&self, snapshot_id: u64, original_inode: InodeNumber, snapshot_inode: InodeNumber) {
        let mut snapshots = self.snapshots.lock();
        if let Some(snapshot) = snapshots.get_mut(&snapshot_id) {
            snapshot.inode_map.insert(original_inode, snapshot_inode);
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn add_inode_mapping(&self, _snapshot_id: u64, _original_inode: InodeNumber, _snapshot_inode: InodeNumber) {
        // No-op
    }
}

#[cfg(feature = "alloc")]
impl Clone for Snapshot {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            timestamp: self.timestamp,
            root_inode: self.root_inode,
            inode_map: self.inode_map.clone(),
        }
    }
}

