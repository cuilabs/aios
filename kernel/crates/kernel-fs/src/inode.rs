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

/// Inode permissions
bitflags::bitflags! {
    pub struct InodePermissions: u16 {
        const OWNER_READ = 0o400;
        const OWNER_WRITE = 0o200;
        const OWNER_EXEC = 0o100;
        const GROUP_READ = 0o040;
        const GROUP_WRITE = 0o020;
        const GROUP_EXEC = 0o010;
        const OTHER_READ = 0o004;
        const OTHER_WRITE = 0o002;
        const OTHER_EXEC = 0o001;
    }
}

/// Inode
pub struct Inode {
    pub ino: InodeNumber,
    pub inode_type: InodeType,
    pub permissions: InodePermissions,
    pub owner: u64, // Agent ID
    pub group: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: u64, // Access time
    pub mtime: u64, // Modify time
    pub ctime: u64, // Change time
}

impl Inode {
    pub fn new(ino: InodeNumber, inode_type: InodeType, owner: u64) -> Self {
        Self {
            ino,
            inode_type,
            permissions: InodePermissions::OWNER_READ | InodePermissions::OWNER_WRITE,
            owner,
            group: 0,
            size: 0,
            blocks: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
    
    /// Check if agent has permission to access
    pub fn check_permission(&self, agent_id: u64, read: bool, write: bool, exec: bool) -> bool {
        // TODO: Implement proper permission checking
        // For now, owner has full access
        if self.owner == agent_id {
            return true;
        }
        
        // TODO: Check group and other permissions
        false
    }
}

