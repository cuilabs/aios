#![no_std]

//! Network stack
//! 
//! TCP/IP implementation with agent isolation

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod ip;
pub mod tcp;
pub mod udp;
pub mod socket;

use spin::Mutex;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Network manager
pub struct NetworkManager {
    #[cfg(feature = "alloc")]
    sockets: Mutex<BTreeMap<u64, SocketHandle>>, // Socket ID -> Handle
    #[cfg(not(feature = "alloc"))]
    sockets: Mutex<()>,
}

static NET_MANAGER: Mutex<Option<NetworkManager>> = Mutex::new(None);

/// Socket handle
#[cfg(feature = "alloc")]
struct SocketHandle {
    socket_id: u64,
    agent_id: u64,
    socket_type: SocketType,
}

/// Socket type
#[derive(Clone, Copy, Debug)]
pub enum SocketType {
    Tcp,
    Udp,
    Raw,
}

/// Initialize network manager
pub fn init() {
    let mut manager = NET_MANAGER.lock();
    *manager = Some(NetworkManager {
        #[cfg(feature = "alloc")]
        sockets: Mutex::new(BTreeMap::new()),
        #[cfg(not(feature = "alloc"))]
        sockets: Mutex::new(()),
    });
}

/// Create socket
#[cfg(feature = "alloc")]
pub fn create_socket(agent_id: u64, socket_type: SocketType) -> Result<u64, NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        let socket_id = sockets.len() as u64 + 1;
        sockets.insert(socket_id, SocketHandle {
            socket_id,
            agent_id,
            socket_type,
        });
        Ok(socket_id)
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn create_socket(_agent_id: u64, _socket_type: SocketType) -> Result<u64, NetError> {
    Err(NetError::NotInitialized)
}

/// Close socket
#[cfg(feature = "alloc")]
pub fn close_socket(socket_id: u64) -> Result<(), NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        sockets.remove(&socket_id).ok_or(NetError::NotFound)?;
        Ok(())
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn close_socket(_socket_id: u64) -> Result<(), NetError> {
    Err(NetError::NotInitialized)
}

/// Network error
#[derive(Debug, Clone, Copy)]
pub enum NetError {
    NotInitialized,
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    Timeout,
    NetworkUnreachable,
    IoError,
}

