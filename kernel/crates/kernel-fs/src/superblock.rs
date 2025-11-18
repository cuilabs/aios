//! Superblock management
//! 
//! Superblocks contain filesystem metadata

use crate::inode::InodeNumber;

/// Superblock
pub struct Superblock {
    pub magic: u32,
    pub block_size: u32,
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub root_inode: InodeNumber,
}

impl Superblock {
    pub fn new(block_size: u32, total_blocks: u64) -> Self {
        Self {
            magic: 0x41494F53, // "AIOS"
            block_size,
            total_blocks,
            free_blocks: total_blocks,
            total_inodes: 0,
            free_inodes: 0,
            root_inode: 1,
        }
    }
}

