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
    /// For large messages, uses memory fabric shared pages.
    pub fn send(&self, message: IPCMessage) -> Result<(), IPCError> {
        // Validate message structure (deterministic checks only)
        if message.from == 0 || message.to == 0 {
            return Err(IPCError::InvalidMessage);
        }
        
        // For large messages (>32KB), use memory fabric shared pages
        if message.data.len() > 32 * 1024 {
            return self.send_via_memory_fabric(message);
        }
        
        if message.data.len() > MAX_MESSAGE_SIZE {
            return Err(IPCError::MessageTooLarge);
        }

        let mut bus = self.bus.lock();
        bus.send(message)
    }
    
    /// Send large message via memory fabric shared pages
    /// 
    /// Note: Memory fabric access is handled by kernel-core to avoid circular dependency.
    /// This is a stub that returns an error - kernel-core should handle large messages directly.
    #[cfg(feature = "alloc")]
    fn send_via_memory_fabric(&self, _message: IPCMessage) -> Result<(), IPCError> {
        // Memory fabric access removed to break circular dependency
        // Kernel-core should handle large messages via memory fabric before calling IPC
        Err(IPCError::MessageTooLarge)
            let agents = alloc::vec![message.from, message.to];
            if let Ok(page_id) = fabric.create_shared_page(agents) {
                // Write message data to shared page
                if let Some(virt_addr) = fabric.get_shared_page_addr(page_id, message.from) {
                    // Write message data to shared page
                    // Copy message data to virtual address
                    unsafe {
                        let dst = virt_addr.as_mut_ptr();
                        let src = message.data.as_ptr();
                        core::ptr::copy_nonoverlapping(src, dst, message.data.len().min(4096));
                    }
                    
                    // Send message with shared page reference
                    let mut bus = self.bus.lock();
                    // Create message with page_id in metadata instead of data
                    let mut metadata = alloc::vec![];
                    // Encode page_id in metadata (8-byte little-endian)
                    metadata.extend_from_slice(&page_id.to_le_bytes());
                    let shared_message = IPCMessage {
                        id: message.id,
                        from: message.from,
                        to: message.to,
                        data: alloc::vec![], // Empty data, page_id in metadata
                        metadata,
                        timestamp: message.timestamp,
                        flags: message.flags,
                    };
                    bus.send(shared_message)
                } else {
                    Err(IPCError::SecurityViolation)
                }
            } else {
                Err(IPCError::SecurityViolation)
            }
        } else {
            Err(IPCError::SecurityViolation)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    fn send_via_memory_fabric(&self, _message: IPCMessage) -> Result<(), IPCError> {
        Err(IPCError::MessageTooLarge)
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

