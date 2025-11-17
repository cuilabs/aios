#![no_std]

//! Binary IPC kernel module
//! 
//! Provides deterministic, byte-level inter-agent communication.
//! Semantic interpretation is handled in userland via semantic-ipcd daemon.

extern crate alloc;

pub mod message;
pub mod bus;
pub mod routing;

use message::IPCMessage;
use bus::IPCMessageBus;

/// Binary IPC system
/// 
/// Kernel handles only binary message packets.
/// Semantic interpretation happens in userland.
pub struct BinaryIPC {
    bus: spin::Mutex<IPCMessageBus>,
}

impl BinaryIPC {
    pub fn new() -> Self {
        Self {
            bus: spin::Mutex::new(IPCMessageBus::new()),
        }
    }

    /// Send binary message packet
    /// 
    /// Kernel validates structure and routes by agent ID only.
    /// No semantic interpretation.
    pub fn send(&self, message: IPCMessage) -> Result<(), IPCError> {
        // Validate message structure (deterministic checks only)
        if message.from == 0 || message.to == 0 {
            return Err(IPCError::InvalidMessage);
        }
        
        if message.data.len() > MAX_MESSAGE_SIZE {
            return Err(IPCError::MessageTooLarge);
        }

        let mut bus = self.bus.lock();
        bus.send(message)
    }

    /// Receive binary message packet
    /// 
    /// Returns raw binary data. Semantic interpretation in userland.
    pub fn receive(&self, agent_id: u64) -> Option<IPCMessage> {
        let mut bus = self.bus.lock();
        bus.receive(agent_id)
    }
}

/// Maximum message size (64KB)
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

#[derive(Debug)]
pub enum IPCError {
    InvalidMessage,
    AgentNotFound,
    SecurityViolation,
    MessageTooLarge,
}

