//! Secure Enclave Driver (Unified Interface)
//! 
//! Provides unified interface for SGX, SEV, and other TEE (Trusted Execution Environment) technologies

use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Enclave type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnclaveType {
    Sgx,    // Intel SGX
    Sev,    // AMD SEV
    SevSnp, // AMD SEV-SNP
    Tdx,    // Intel TDX
    None,
}

/// Enclave attestation
pub struct EnclaveAttestation {
    pub enclave_type: EnclaveType,
    pub enclave_id: u64,
    pub measurement: Vec<u8>,
    pub certificate: Vec<u8>,
    pub signature: Vec<u8>,
    pub report: Vec<u8>,
}

/// Unified enclave driver
pub struct EnclaveDriver {
    enclave_type: EnclaveType,
    initialized: bool,
}

static ENCLAVE_DRIVER: Mutex<Option<EnclaveDriver>> = Mutex::new(None);

/// Initialize enclave driver
pub fn init() {
    // Detect available enclave technology
    let enclave_type = detect_enclave_type();
    
    if enclave_type != EnclaveType::None {
        let mut driver = ENCLAVE_DRIVER.lock();
        *driver = Some(EnclaveDriver {
            enclave_type,
            initialized: true,
        });
    }
}

/// Detect available enclave technology
fn detect_enclave_type() -> EnclaveType {
    // Check for Intel SGX
    #[cfg(feature = "sgx")]
    {
        use crate::sgx;
        if sgx::is_available() {
            return EnclaveType::Sgx;
        }
    }
    
    // Check for AMD SEV
    unsafe {
        let cpuid_result = x86_64::instructions::cpuid::CpuId::new();
        if let Some(extended_feature_info) = cpuid_result.get_extended_feature_info() {
            // Check for SEV support (CPUID.8000001FH:EAX[bit 1] = SEV)
            // Check for SEV-SNP support (CPUID.8000001FH:EAX[bit 4] = SEV-SNP)
            // SEV driver implementation is required for SEV/SEV-SNP support
            // Return None until SEV driver is implemented
        }
    }
    
    // Check for Intel TDX
    // TDX detection via CPUID and ACPI tables
    // TDX driver implementation is required for TDX support
    // Return None until TDX driver is implemented
    
    EnclaveType::None
}

/// Create enclave
pub fn create_enclave(base_address: u64, size: u64, attributes: u64) -> Result<u64, EnclaveError> {
    let driver = ENCLAVE_DRIVER.lock();
    if let Some(ref driver) = *driver {
        match driver.enclave_type {
            EnclaveType::Sgx => {
                use crate::sgx;
                sgx::create_enclave(base_address, size, attributes)
                    .map_err(|_| EnclaveError::CreationFailed)
            }
            EnclaveType::Sev | EnclaveType::SevSnp => {
                // SEV enclave creation
                // SEV driver is required for SEV/SEV-SNP enclave creation
                Err(EnclaveError::NotImplemented)
            }
            EnclaveType::Tdx => {
                // TDX enclave creation
                // TDX driver is required for TDX enclave creation
                Err(EnclaveError::NotImplemented)
            }
            EnclaveType::None => Err(EnclaveError::NotAvailable),
        }
    } else {
        Err(EnclaveError::NotAvailable)
    }
}

/// Initialize enclave
pub fn initialize_enclave(enclave_id: u64, launch_token: Option<&[u8]>) -> Result<(), EnclaveError> {
    let driver = ENCLAVE_DRIVER.lock();
    if let Some(ref driver) = *driver {
        match driver.enclave_type {
            EnclaveType::Sgx => {
                use crate::sgx;
                sgx::initialize_enclave(enclave_id, launch_token)
                    .map_err(|_| EnclaveError::InitializationFailed)
            }
            EnclaveType::Sev | EnclaveType::SevSnp => {
                // SEV enclave initialization
                Err(EnclaveError::NotImplemented)
            }
            EnclaveType::Tdx => {
                // TDX enclave initialization
                Err(EnclaveError::NotImplemented)
            }
            EnclaveType::None => Err(EnclaveError::NotAvailable),
        }
    } else {
        Err(EnclaveError::NotAvailable)
    }
}

/// Generate enclave attestation
pub fn generate_attestation(enclave_id: u64, report_data: &[u8]) -> Result<EnclaveAttestation, EnclaveError> {
    let driver = ENCLAVE_DRIVER.lock();
    if let Some(ref driver) = *driver {
        match driver.enclave_type {
            EnclaveType::Sgx => {
                use crate::sgx;
                let report = sgx::generate_attestation_report(enclave_id, report_data)
                    .map_err(|_| EnclaveError::AttestationFailed)?;
                
                Ok(EnclaveAttestation {
                    enclave_type: EnclaveType::Sgx,
                    enclave_id,
                    measurement: report[0..32].to_vec(), // MRENCLAVE
                    certificate: Vec::new(), // Would include certificate chain
                    signature: Vec::new(),   // Would include signature
                    report,
                })
            }
            EnclaveType::Sev | EnclaveType::SevSnp => {
                // SEV attestation
                Err(EnclaveError::NotImplemented)
            }
            EnclaveType::Tdx => {
                // TDX attestation
                Err(EnclaveError::NotImplemented)
            }
            EnclaveType::None => Err(EnclaveError::NotAvailable),
        }
    } else {
        Err(EnclaveError::NotAvailable)
    }
}

/// Verify enclave attestation
pub fn verify_attestation(attestation: &EnclaveAttestation, expected_measurement: &[u8]) -> Result<bool, EnclaveError> {
    match attestation.enclave_type {
        EnclaveType::Sgx => {
            use crate::sgx;
            sgx::verify_attestation_report(&attestation.report, expected_measurement)
                .map_err(|_| EnclaveError::VerificationFailed)
        }
        EnclaveType::Sev | EnclaveType::SevSnp => {
            // SEV attestation verification
            Ok(false) // Not implemented
        }
        EnclaveType::Tdx => {
            // TDX attestation verification
            Ok(false) // Not implemented
        }
        EnclaveType::None => Ok(false),
    }
}

/// Check if enclave support is available
pub fn is_available() -> bool {
    let driver = ENCLAVE_DRIVER.lock();
    driver.as_ref().map(|d| d.initialized).unwrap_or(false)
}

/// Get enclave type
pub fn get_enclave_type() -> EnclaveType {
    let driver = ENCLAVE_DRIVER.lock();
    driver.as_ref().map(|d| d.enclave_type).unwrap_or(EnclaveType::None)
}

#[derive(Debug, Clone, Copy)]
pub enum EnclaveError {
    NotAvailable,
    NotSupported,
    NotImplemented,
    CreationFailed,
    InitializationFailed,
    AttestationFailed,
    VerificationFailed,
}

