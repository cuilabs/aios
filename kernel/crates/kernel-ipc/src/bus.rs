//! Binary IPC message bus
//! 
//! Routes binary messages between agents by agent ID.
//! No semantic interpretation - pure routing.

use super::message::IPCMessage;
use super::IPCError;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Binary IPC message bus
/// 
/// Routes messages by agent ID only.
/// Deterministic, byte-level routing.
pub struct IPCMessageBus {
    /// Message queues per agent
    queues: Mutex<BTreeMap<u64, Vec<IPCMessage>>>,
    /// Message ID counter
    next_id: Mutex<u64>,
}

impl IPCMessageBus {
    pub fn new() -> Self {
        Self {
            queues: Mutex::new(BTreeMap::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Send binary message
    /// 
    /// Routes by agent ID. No semantic interpretation.
    pub fn send(&mut self, mut message: IPCMessage) -> Result<(), IPCError> {
        // Generate message ID
        let mut next_id = self.next_id.lock();
        message.id = *next_id;
        *next_id = next_id.wrapping_add(1);
        
        // Set timestamp (simplified - would use kernel time)
        message.timestamp = 0; // TODO: Get from kernel time

        // Route to destination agent
        let mut queues = self.queues.lock();
        let queue = queues.entry(message.to).or_insert_with(Vec::new);
        queue.push(message);
        
        Ok(())
    }

    /// Receive binary message for agent
    /// 
    /// Returns raw binary data. No semantic interpretation.
    pub fn receive(&mut self, agent_id: u64) -> Option<IPCMessage> {
        let mut queues = self.queues.lock();
        let queue = queues.get_mut(&agent_id)?;
        queue.pop()
    }

    /// Get queue length for agent
    pub fn queue_length(&self, agent_id: u64) -> usize {
        let queues = self.queues.lock();
        queues.get(&agent_id).map(|q| q.len()).unwrap_or(0)
    }
}
