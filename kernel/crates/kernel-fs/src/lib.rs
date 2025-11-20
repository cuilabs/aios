#![no_std]

//! Agent-native filesystem
//! 
//! Provides filesystem operations with capability-based access control

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod inode;
pub mod superblock;
pub mod directory;
pub mod file;
pub mod journal;
pub mod snapshot;

use spin::Mutex;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use alloc::string::String;

use inode::{Inode, InodeNumber, InodeType};
use directory::Directory;
use file::File;
use journal::JournalManager;
use snapshot::SnapshotManager;

#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
#[cfg(feature = "alloc")]
use aios_kernel_core::time;
#[cfg(feature = "alloc")]
use aios_kernel_policy::check_policy;

/// Filesystem manager
pub struct FilesystemManager {
    #[cfg(feature = "alloc")]
    mounts: Mutex<BTreeMap<String, MountPoint>>,
    #[cfg(feature = "alloc")]
    inodes: Mutex<BTreeMap<InodeNumber, Inode>>,
    #[cfg(feature = "alloc")]
    directories: Mutex<BTreeMap<InodeNumber, Directory>>,
    #[cfg(feature = "alloc")]
    files: Mutex<BTreeMap<InodeNumber, File>>,
    #[cfg(feature = "alloc")]
    next_inode: Mutex<InodeNumber>,
    #[cfg(feature = "alloc")]
    journal: JournalManager,
    #[cfg(feature = "alloc")]
    snapshots: SnapshotManager,
    #[cfg(not(feature = "alloc"))]
    mounts: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    inodes: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    directories: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    files: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_inode: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    journal: JournalManager,
    #[cfg(not(feature = "alloc"))]
    snapshots: SnapshotManager,
}

static FS_MANAGER: Mutex<Option<FilesystemManager>> = Mutex::new(None);

/// Mount point
#[cfg(feature = "alloc")]
struct MountPoint {
    path: String,
    fs_type: String,
    device: String,
    flags: MountFlags,
}

/// Mount flags
bitflags::bitflags! {
    pub struct MountFlags: u32 {
        const READ_ONLY = 1 << 0;
        const NO_EXEC = 1 << 1;
        const NO_SUID = 1 << 2;
        const NO_DEV = 1 << 3;
    }
}

/// Initialize filesystem manager
pub fn init() {
    let mut manager = FS_MANAGER.lock();
    *manager = Some(FilesystemManager {
        #[cfg(feature = "alloc")]
        mounts: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        inodes: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        directories: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        files: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        next_inode: Mutex::new(1), // Start from 1 (0 is reserved)
        #[cfg(feature = "alloc")]
        journal: JournalManager::new(),
        #[cfg(feature = "alloc")]
        snapshots: SnapshotManager::new(),
        #[cfg(not(feature = "alloc"))]
        mounts: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        inodes: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        directories: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        files: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        next_inode: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        journal: JournalManager::new(),
        #[cfg(not(feature = "alloc"))]
        snapshots: SnapshotManager::new(),
    });
}

/// Mount filesystem
#[cfg(feature = "alloc")]
pub fn mount(path: String, fs_type: String, device: String, flags: MountFlags) -> Result<(), FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut mounts = manager.mounts.lock();
        mounts.insert(path.clone(), MountPoint {
            path,
            fs_type,
            device,
            flags,
        });
        Ok(())
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn mount(_path: &str, _fs_type: &str, _device: &str, _flags: MountFlags) -> Result<(), FsError> {
    Err(FsError::NotInitialized)
}

/// Unmount filesystem
#[cfg(feature = "alloc")]
pub fn unmount(path: &str) -> Result<(), FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut mounts = manager.mounts.lock();
        mounts.remove(path);
        Ok(())
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn unmount(_path: &str) -> Result<(), FsError> {
    Err(FsError::NotInitialized)
}

/// Open agent storage (agent-scoped, not traditional file open)
#[cfg(feature = "alloc")]
pub fn open_agent_storage(path: &str, agent_id: u64, capability: Option<&aios_kernel_capability::capability::CapabilityToken>, create: bool) -> Result<InodeNumber, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check capability if provided
        if let Some(cap) = capability {
            use aios_kernel_capability::{capability::Capabilities, has_capability};
            if !has_capability(Some(cap), Capabilities::ACCESS_FS) {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: time::now(),
                        agent_id: Some(agent_id),
                        data: alloc::vec![("action".into(), "fs_access_denied".into()), ("path".into(), path.into())],
                    });
                }
                return Err(FsError::PermissionDenied);
            }
        } else {
            // Capability required for filesystem access
            #[cfg(feature = "alloc")]
            {
                publish_event(KernelEvent {
                    event_type: EventType::Security,
                    timestamp: time::now(),
                    agent_id: Some(agent_id),
                    data: alloc::vec![("action".into(), "fs_no_capability".into()), ("path".into(), path.into())],
                });
            }
            return Err(FsError::PermissionDenied);
        }
        
        // Check policy
        #[cfg(feature = "alloc")]
        {
            if !check_policy(agent_id, "fs_open", Some(path)) {
                return Err(FsError::PermissionDenied);
            }
        }
        
        // Parse path and find inode
        let inodes = manager.inodes.lock();
        let directories = manager.directories.lock();
        
        // Parse path (split by '/', handle absolute/relative paths)
        // Normalize path: remove leading/trailing slashes, collapse multiple slashes
        let normalized_path = path.trim_matches('/');
        let parts: Vec<&str> = normalized_path.split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Err(FsError::InvalidPath);
        }
        
        // Start from root directory (inode 1)
        let mut current_inode = 1;
        
        // Traverse path
        for (i, part) in parts.iter().enumerate() {
            if let Some(dir) = directories.get(&current_inode) {
                if let Some(next_inode) = dir.lookup(part) {
                    current_inode = next_inode;
                } else {
                    // Not found
                    if create && i == parts.len() - 1 {
                        // Create new file
                        return create_file(current_inode, part, agent_id);
                    } else {
                        return Err(FsError::NotFound);
                    }
                }
            } else {
                return Err(FsError::NotFound);
            }
        }
        
        // Check permissions
        if let Some(inode) = inodes.get(&current_inode) {
            if !inode.check_permission(agent_id, true, false, false) {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: time::now(),
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(FsError::PermissionDenied);
            }
            
            // Publish filesystem event
            #[cfg(feature = "alloc")]
            {
                publish_event(KernelEvent {
                    event_type: EventType::FileSystem,
                    timestamp: time::now(),
                    agent_id: Some(agent_id),
                    data: alloc::vec::Vec::new(),
                });
            }
            
            Ok(current_inode)
        } else {
            Err(FsError::NotFound)
        }
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn open_agent_storage(_path: &str, _agent_id: u64, _capability: Option<&aios_kernel_capability::capability::CapabilityToken>, _create: bool) -> Result<InodeNumber, FsError> {
    Err(FsError::NotInitialized)
}

/// Create new file
#[cfg(feature = "alloc")]
fn create_file(parent_inode: InodeNumber, name: &str, agent_id: u64) -> Result<InodeNumber, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut next_inode = manager.next_inode.lock();
        let inode_num = *next_inode;
        *next_inode += 1;
        
        // Create inode (agent-scoped storage)
        let inode = Inode::new(inode_num, InodeType::File, agent_id); // agent_id is creator_agent_id
        let mut inodes = manager.inodes.lock();
        inodes.insert(inode_num, inode);
        
        // Create file
        let file = File::new(inode_num);
        let mut files = manager.files.lock();
        files.insert(inode_num, file);
        
        // Add to parent directory
        let mut directories = manager.directories.lock();
        if let Some(parent_dir) = directories.get_mut(&parent_inode) {
            parent_dir.add_entry(String::from(name), inode_num)?;
        } else {
            return Err(FsError::NotFound);
        }
        
        Ok(inode_num)
    } else {
        Err(FsError::NotInitialized)
    }
}

/// Read from agent storage (agent-scoped, not traditional file read)
#[cfg(feature = "alloc")]
pub fn read_agent_storage(inode_num: InodeNumber, agent_id: u64, offset: u64, len: usize) -> Result<Vec<u8>, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check permissions
        let inodes = manager.inodes.lock();
        if let Some(inode) = inodes.get(&inode_num) {
            if !inode.check_permission(agent_id, true, false, false) {
                return Err(FsError::PermissionDenied);
            }
        } else {
            return Err(FsError::NotFound);
        }
        
        // Read from file
        let files = manager.files.lock();
        if let Some(file) = files.get(&inode_num) {
            let result = file.read(offset, len);
            
            // Publish filesystem event on success
            #[cfg(feature = "alloc")]
            if result.is_ok() {
                publish_event(KernelEvent {
                    event_type: EventType::FileSystem,
                    timestamp: time::now(),
                    agent_id: Some(agent_id),
                    data: alloc::vec::Vec::new(),
                });
            }
            
            result
        } else {
            Err(FsError::NotFound)
        }
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn read_agent_storage(_inode_num: InodeNumber, _agent_id: u64, _offset: u64, _len: usize) -> Result<Vec<u8>, FsError> {
    Err(FsError::NotInitialized)
}

/// Write to agent storage (agent-scoped, not traditional file write)
#[cfg(feature = "alloc")]
pub fn write_agent_storage(inode_num: InodeNumber, agent_id: u64, offset: u64, data: &[u8], capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<usize, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check capability if provided
        if let Some(cap) = capability {
            use aios_kernel_capability::{capability::Capabilities, has_capability};
            if !has_capability(Some(cap), Capabilities::ACCESS_FS) {
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: time::now(),
                        agent_id: Some(agent_id),
                        data: alloc::vec![("action".into(), "fs_write_denied".into())],
                    });
                }
                return Err(FsError::PermissionDenied);
            }
        } else {
            #[cfg(feature = "alloc")]
            {
                publish_event(KernelEvent {
                    event_type: EventType::Security,
                    timestamp: time::now(),
                    agent_id: Some(agent_id),
                    data: alloc::vec![("action".into(), "fs_write_no_capability".into())],
                });
            }
            return Err(FsError::PermissionDenied);
        }
        
        // Start transaction for journaling
        let transaction_id = manager.journal.begin_transaction();
        
        // Check permissions
        let mut inodes = manager.inodes.lock();
        if let Some(inode) = inodes.get_mut(&inode_num) {
            if !inode.check_permission(agent_id, false, true, false) {
                manager.journal.rollback_transaction(transaction_id);
                return Err(FsError::PermissionDenied);
            }
            // Update modify time
            inode.mtime = time::now();
        } else {
            manager.journal.rollback_transaction(transaction_id);
            return Err(FsError::NotFound);
        }
        
        // Write to file
        let mut files = manager.files.lock();
        if let Some(file) = files.get_mut(&inode_num) {
            let result = file.write(offset, data);
            
            // Add to journal
            if result.is_ok() {
                let _ = manager.journal.add_entry(
                    transaction_id,
                    journal::JournalEntryType::WriteFile,
                    inode_num,
                    data.to_vec(),
                );
                let _ = manager.journal.commit_transaction(transaction_id);
            } else {
                manager.journal.rollback_transaction(transaction_id);
            }
            
            // Publish filesystem event on success
            #[cfg(feature = "alloc")]
            if result.is_ok() {
                publish_event(KernelEvent {
                    event_type: EventType::FileSystem,
                    timestamp: time::now(),
                    agent_id: Some(agent_id),
                    data: alloc::vec::Vec::new(),
                });
            }
            
            result
        } else {
            manager.journal.rollback_transaction(transaction_id);
            Err(FsError::NotFound)
        }
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn write_agent_storage(_inode_num: InodeNumber, _agent_id: u64, _offset: u64, _data: &[u8]) -> Result<usize, FsError> {
    Err(FsError::NotInitialized)
}

/// Create directory
#[cfg(feature = "alloc")]
pub fn create_dir(path: &str, agent_id: u64) -> Result<InodeNumber, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut next_inode = manager.next_inode.lock();
        let inode_num = *next_inode;
        *next_inode += 1;
        
        // Create inode (agent-scoped storage)
        let inode = Inode::new(inode_num, InodeType::Directory, agent_id); // agent_id is creator_agent_id
        let mut inodes = manager.inodes.lock();
        inodes.insert(inode_num, inode);
        
        // Create directory
        let directory = Directory::new(inode);
        let mut directories = manager.directories.lock();
        directories.insert(inode_num, directory);
        
        // Publish filesystem event
        #[cfg(feature = "alloc")]
        {
            publish_event(KernelEvent {
                event_type: EventType::FileSystem,
                timestamp: time::now(),
                agent_id: Some(agent_id),
                data: alloc::vec::Vec::new(),
            });
        }
        
        Ok(inode_num)
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn create_dir(_path: &str, _agent_id: u64) -> Result<InodeNumber, FsError> {
    Err(FsError::NotInitialized)
}

/// List directory
#[cfg(feature = "alloc")]
pub fn list_dir(inode_num: InodeNumber, agent_id: u64) -> Result<Vec<String>, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check permissions
        let inodes = manager.inodes.lock();
        if let Some(inode) = inodes.get(&inode_num) {
            if !inode.check_permission(agent_id, true, false, false) {
                return Err(FsError::PermissionDenied);
            }
        } else {
            return Err(FsError::NotFound);
        }
        
        // List directory entries
        let directories = manager.directories.lock();
        if let Some(dir) = directories.get(&inode_num) {
            #[cfg(feature = "alloc")]
            {
                Ok(dir.entries.keys().cloned().collect())
            }
            #[cfg(not(feature = "alloc"))]
            {
                Err(FsError::NotInitialized)
            }
        } else {
            Err(FsError::NotFound)
        }
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn list_dir(_inode_num: InodeNumber, _agent_id: u64) -> Result<Vec<String>, FsError> {
    Err(FsError::NotInitialized)
}

/// Delete directory
#[cfg(feature = "alloc")]
pub fn delete_dir(inode_num: InodeNumber, agent_id: u64) -> Result<(), FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check permissions
        let mut inodes = manager.inodes.lock();
        if let Some(inode) = inodes.get(&inode_num) {
            if !inode.check_permission(agent_id, false, true, false) {
                return Err(FsError::PermissionDenied);
            }
        } else {
            return Err(FsError::NotFound);
        }
        
        // Check if directory is empty
        {
            let directories = manager.directories.lock();
            if let Some(dir) = directories.get(&inode_num) {
                #[cfg(feature = "alloc")]
                {
                    if !dir.entries.is_empty() {
                        return Err(FsError::InvalidPath); // Directory not empty
                    }
                }
            }
        }
        
        // Remove directory
        let mut directories = manager.directories.lock();
        directories.remove(&inode_num);
        inodes.remove(&inode_num);
        
        Ok(())
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn delete_dir(_inode_num: InodeNumber, _agent_id: u64) -> Result<(), FsError> {
    Err(FsError::NotInitialized)
}

/// Create snapshot
#[cfg(feature = "alloc")]
pub fn create_snapshot(name: &str, agent_id: u64) -> Result<u64, FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let root_inode = 1; // Root inode
        let snapshot_id = manager.snapshots.create_snapshot(
            String::from(name),
            root_inode,
            time::now(),
        );
        
        // Publish filesystem event
        publish_event(KernelEvent {
            event_type: EventType::FileSystem,
            timestamp: time::now(),
            agent_id: Some(agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(snapshot_id)
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn create_snapshot(_name: &str, _agent_id: u64) -> Result<u64, FsError> {
    Err(FsError::NotInitialized)
}

/// List snapshots
#[cfg(feature = "alloc")]
pub fn list_snapshots() -> Vec<(u64, String, u64)> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        manager.snapshots.list_snapshots()
    } else {
        Vec::new()
    }
}

#[cfg(not(feature = "alloc"))]
pub fn list_snapshots() -> Vec<(u64, String, u64)> {
    Vec::new()
}

/// Delete snapshot
#[cfg(feature = "alloc")]
pub fn delete_snapshot(snapshot_id: u64, agent_id: u64) -> Result<(), FsError> {
    let manager = FS_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let result = manager.snapshots.delete_snapshot(snapshot_id);
        
        if result.is_ok() {
            // Publish filesystem event
            publish_event(KernelEvent {
                event_type: EventType::FileSystem,
                timestamp: time::now(),
                agent_id: Some(agent_id),
                data: alloc::vec::Vec::new(),
            });
        }
        
        result
    } else {
        Err(FsError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn delete_snapshot(_snapshot_id: u64, _agent_id: u64) -> Result<(), FsError> {
    Err(FsError::NotInitialized)
}

/// Filesystem error
#[derive(Debug, Clone, Copy)]
pub enum FsError {
    NotInitialized,
    NotFound,
    PermissionDenied,
    InvalidPath,
    OutOfSpace,
    IoError,
}
