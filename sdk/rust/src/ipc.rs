//! IPC API

use crate::kernel::KernelClient;

/// IPC client
pub struct IPCClient {
    kernel: KernelClient,
}

impl IPCClient {
    /// Send message
    pub fn send(&self, to: u64, data: &[u8]) -> Result<(), IPCError> {
        // TODO: Send IPC message
        Ok(())
    }
    
    /// Receive message
    pub fn receive(&self) -> Result<IPCMessage, IPCError> {
        // TODO: Receive IPC message
        Err(IPCError::NoMessage)
    }
}

/// IPC message
pub struct IPCMessage {
    pub from: u64,
    pub data: Vec<u8>,
}

/// IPC error
#[derive(Debug)]
pub enum IPCError {
    NoMessage,
    InvalidMessage,
    PermissionDenied,
}

