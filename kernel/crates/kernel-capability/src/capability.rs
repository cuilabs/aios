//! Capability token management

/// Capability token
#[repr(C)]
pub struct CapabilityToken {
    pub token_id: u64,
    pub agent_id: u64,
    pub capabilities: u64,        // Bitmask
    pub expires_at: u64,          // Unix timestamp
    pub signature: [u8; 64],      // Cryptographic signature
}

bitflags::bitflags! {
    /// Capability flags
    #[repr(transparent)]
    pub struct Capabilities: u64 {
        const SPAWN_AGENT = 1 << 0;
        const KILL_AGENT = 1 << 1;
        const ALLOC_MEMORY = 1 << 2;
        const ACCESS_FS = 1 << 3;
        const ACCESS_NET = 1 << 4;
        const ACCESS_IO = 1 << 5;
        const ACCESS_GPU = 1 << 6;
        const ACCESS_TPU = 1 << 7;
        const ACCESS_NPU = 1 << 8;
        const SUPERVISOR = 1 << 9;
        const ADMIN = 1 << 10;
    }
}

impl CapabilityToken {
    /// Check if token has capability
    pub fn has_capability(&self, capability: Capabilities) -> bool {
        (self.capabilities & capability.bits()) != 0
    }

    /// Check if token is expired
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at < now
    }
}

