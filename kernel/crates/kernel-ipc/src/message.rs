//! Binary IPC message structure
//! 
//! Kernel handles only binary message packets.
//! All semantic metadata is opaque to the kernel.

use alloc::vec::Vec;

/// Binary IPC message
/// 
/// Kernel sees only:
/// - Source agent ID
/// - Destination agent ID
/// - Binary payload
/// - Optional metadata (opaque to kernel)
/// 
/// Semantic interpretation happens in userland.
#[repr(C)]
pub struct IPCMessage {
    /// Message ID (kernel-generated)
    pub id: u64,
    /// Source agent ID
    pub from: u64,
    /// Destination agent ID
    pub to: u64,
    /// Binary payload (opaque to kernel)
    pub data: Vec<u8>,
    /// Optional metadata (opaque to kernel, for userland use)
    pub metadata: Vec<u8>,
    /// Timestamp (kernel-set)
    pub timestamp: u64,
    /// Flags (kernel-controlled)
    pub flags: MessageFlags,
}

bitflags::bitflags! {
    /// Message flags (kernel-controlled)
    #[repr(transparent)]
    pub struct MessageFlags: u32 {
        /// Message requires delivery confirmation
        const REQUIRES_ACK = 1 << 0;
        /// Message is high priority
        const HIGH_PRIORITY = 1 << 1;
        /// Message is encrypted (kernel doesn't decrypt)
        const ENCRYPTED = 1 << 2;
    }
}

impl IPCMessage {
    /// Create new binary IPC message
    /// 
    /// Kernel validates structure but does not interpret content.
    pub fn new(from: u64, to: u64, data: Vec<u8>, metadata: Vec<u8>) -> Self {
        Self {
            id: 0, // Kernel will generate
            from,
            to,
            data,
            metadata,
            timestamp: 0, // Kernel will set
            flags: MessageFlags::empty(),
        }
    }

    /// Set message flags
    pub fn set_flags(&mut self, flags: MessageFlags) {
        self.flags = flags;
    }
}
