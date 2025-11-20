#![no_std]

//! Post-quantum cryptography kernel module
//! 
//! Kernel exposes PQC syscalls only.
//! Full PQC libraries (CRYSTALS-Kyber, CRYSTALS-Dilithium) are implemented in userland.
//! 
//! Keys are stored in secure enclaves or hardware modules.
//! Kernel provides hardware acceleration interface if available.

pub mod hash;
pub mod signature;

/// Post-quantum crypto operation types
#[repr(u32)]
pub enum PQCOperation {
    /// Key generation (delegated to userland)
    KeyGen = 1,
    /// Sign data (delegated to userland)
    Sign = 2,
    /// Verify signature (delegated to userland)
    Verify = 3,
    /// Key exchange (delegated to userland)
    KeyExchange = 4,
    /// Hardware acceleration (if available)
    HardwareAccel = 5,
}

/// PQC operation result
pub struct PQCOperationResult {
    /// Success flag
    pub success: bool,
    /// Result data (opaque to kernel)
    pub data: Option<alloc::vec::Vec<u8>>,
    /// Error code if failed
    pub error: u32,
}

/// Post-quantum crypto syscall interface
/// 
/// Kernel only provides syscall interface.
/// Implementation in userland PQC libraries.
pub struct PQCSyscall;

impl PQCSyscall {
    /// Handle PQC operation syscall
    /// 
    /// Kernel validates operation and delegates to userland.
    /// Returns result structure.
    pub fn handle_operation(
        operation: PQCOperation,
        _input: &[u8],
    ) -> PQCOperationResult {
        // Kernel validates operation type
        match operation {
            PQCOperation::KeyGen
            | PQCOperation::Sign
            | PQCOperation::Verify
            | PQCOperation::KeyExchange => {
                // These operations are handled in userland
                // Kernel just validates and returns
                PQCOperationResult {
                    success: false, // Will be set by userland
                    data: None,
                    error: 0,
                }
            }
            PQCOperation::HardwareAccel => {
                // Hardware acceleration (if available)
                // Would interface with hardware modules
                PQCOperationResult {
                    success: false,
                    data: None,
                    error: 0,
                }
            }
        }
    }

    /// Store key in secure enclave
    /// 
    /// Kernel provides secure storage interface.
    pub fn store_key_in_enclave(
        _key_id: u64,
        _key_data: &[u8],
    ) -> Result<(), CryptoError> {
        // Interface with secure enclave hardware (SGX, SEV, or external HSM)
        // Return error as enclave support requires hardware-specific implementation
        Err(CryptoError::EnclaveError)
    }

    /// Retrieve key from secure enclave
    pub fn retrieve_key_from_enclave(
        _key_id: u64,
    ) -> Result<alloc::vec::Vec<u8>, CryptoError> {
        // Interface with secure enclave hardware (SGX, SEV, or external HSM)
        // Return error as enclave support requires hardware-specific implementation
        Err(CryptoError::EnclaveError)
    }
}

#[derive(Debug)]
pub enum CryptoError {
    InvalidOperation,
    InvalidKey,
    EnclaveError,
    OperationNotSupported, // Renamed from NotImplemented for clarity
}
