//! Inode management
//! 
//! Inodes represent files and directories in the filesystem

use spin::Mutex;
use crate::FsError;

/// Inode number
pub type InodeNumber = u64;

/// Inode type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InodeType {
    File,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,
}

/// Storage access permissions (capability-based)
bitflags::bitflags! {
    pub struct StoragePermissions: u16 {
        /// Agent can read from this storage
        const READ = 1 << 0;
        /// Agent can write to this storage
        const WRITE = 1 << 1;
        /// Agent can execute from this storage
        const EXECUTE = 1 << 2;
        /// Agent can share this storage with other agents
        const SHARE = 1 << 3;
        /// Agent can delete this storage
        const DELETE = 1 << 4;
    }
}

/// Storage node (inode) - represents agent-scoped storage
#[derive(Clone)]
pub struct Inode {
    pub ino: InodeNumber,
    pub inode_type: InodeType,
    pub permissions: StoragePermissions,
    pub creator_agent_id: u64, // Agent that created this storage
    pub shared_with: Vec<u64>, // List of agent IDs that have access (capability-based)
    pub size: u64,
    pub blocks: u64,
    pub atime: u64, // Access time
    pub mtime: u64, // Modify time
    pub ctime: u64, // Change time
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

impl Inode {
    pub fn new(ino: InodeNumber, inode_type: InodeType, creator_agent_id: u64) -> Self {
        Self {
            ino,
            inode_type,
            permissions: StoragePermissions::READ | StoragePermissions::WRITE,
            creator_agent_id,
            #[cfg(feature = "alloc")]
            shared_with: Vec::new(),
            #[cfg(not(feature = "alloc"))]
            shared_with: (),
            size: 0,
            blocks: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
    
    /// Check if agent has permission to access (capability-based)
    /// 
    /// Permissions are checked via capability system, not owner/group model.
    /// Creator agent has full access. Other agents need explicit capability grants.
    pub fn check_permission(&self, agent_id: u64, read: bool, write: bool, exec: bool) -> bool {
        // Creator agent has full access
        if self.creator_agent_id == agent_id {
            if read && !self.permissions.contains(StoragePermissions::READ) {
                return false;
            }
            if write && !self.permissions.contains(StoragePermissions::WRITE) {
                return false;
            }
            if exec && !self.permissions.contains(StoragePermissions::EXECUTE) {
                return false;
            }
            return true;
        }
        
        // Check if agent is in shared_with list (has been granted access)
        #[cfg(feature = "alloc")]
        {
            if self.shared_with.contains(&agent_id) {
                // Agent has been granted access - check specific permissions
                if read && !self.permissions.contains(StoragePermissions::READ) {
                    return false;
                }
                if write && !self.permissions.contains(StoragePermissions::WRITE) {
                    return false;
                }
                if exec && !self.permissions.contains(StoragePermissions::EXECUTE) {
                    return false;
                }
                return true;
            }
        }
        
        // Agent not in shared list - no access
        // Check capability system for dynamic grants
        // Capability-based access is checked via capability token validation
        // If agent has ACCESS_STORAGE capability, grant access
        // This is handled by the caller providing a valid capability token
        false
    }
    
    /// Grant access to another agent (capability-based sharing)
    #[cfg(feature = "alloc")]
    pub fn grant_access(&mut self, agent_id: u64) {
        if !self.shared_with.contains(&agent_id) {
            self.shared_with.push(agent_id);
        }
    }
    
    /// Revoke access from an agent
    #[cfg(feature = "alloc")]
    pub fn revoke_access(&mut self, agent_id: u64) {
        self.shared_with.retain(|&id| id != agent_id);
    }
}

