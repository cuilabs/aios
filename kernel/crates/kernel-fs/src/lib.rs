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

use spin::Mutex;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// Filesystem manager
pub struct FilesystemManager {
    #[cfg(feature = "alloc")]
    mounts: Mutex<BTreeMap<String, MountPoint>>,
    #[cfg(not(feature = "alloc"))]
    mounts: Mutex<()>,
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
        #[cfg(not(feature = "alloc"))]
        mounts: Mutex::new(()),
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

