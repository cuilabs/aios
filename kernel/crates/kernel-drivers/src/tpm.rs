//! TPM (Trusted Platform Module) Driver
//! 
//! Provides TPM 2.0 interface for attestation, key management, and secure operations

use spin::Mutex;
use x86_64::PhysAddr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// TPM interface type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TpmInterface {
    CRB,  // Command Response Buffer (TPM 2.0)
    FIFO, // FIFO interface (TPM 1.2/2.0)
    MMIO, // Memory-mapped I/O
    None,
}

/// TPM device
pub struct TpmDevice {
    interface: TpmInterface,
    base_address: PhysAddr,
    locality: u8,
    initialized: bool,
}

/// TPM command header (TPM 2.0)
#[repr(C, packed)]
struct TpmCommandHeader {
    tag: u16,      // TPM_ST_NO_SESSIONS or TPM_ST_SESSIONS
    size: u32,     // Command size
    code: u32,     // Command code
}

/// TPM response header (TPM 2.0)
#[repr(C, packed)]
struct TpmResponseHeader {
    tag: u16,
    size: u32,
    response_code: u32,
}

/// TPM command codes (TPM 2.0)
#[repr(u32)]
enum TpmCommandCode {
    Startup = 0x00000144,
    GetCapability = 0x0000017A,
    ReadPCR = 0x0000017E,
    Quote = 0x0000016E,
    VerifySignature = 0x00000177,
    CreatePrimary = 0x00000131,
    Sign = 0x0000015D,
}

/// TPM response codes
#[repr(u32)]
pub enum TpmResponseCode {
    Success = 0x00000000,
    BadTag = 0x0000001E,
    BadSize = 0x0000001D,
    BadParameter = 0x00000003,
    NotAvailable = 0x00000097,
}

static TPM_DEVICE: Mutex<Option<TpmDevice>> = Mutex::new(None);

/// Initialize TPM driver
pub fn init() {
    // Detect TPM via ACPI TPM2 table
    use aios_kernel_hal::acpi;
    
    if acpi::is_available() {
        if let Some(tpm2_table_ptr) = acpi::get_table(b"TPM2") {
            unsafe {
                // Parse TPM2 ACPI table
                // TPM2 table format: [Signature(4)][Length(4)][Revision(1)][Checksum(1)][OEMID(6)][OEMTableID(8)][OEMRevision(4)][CreatorID(4)][CreatorRevision(4)][ControlAddress(8)][StartMethod(4)]
                // Read table length from header
                let length_ptr = tpm2_table_ptr.add(4) as *const u32;
                let table_length = *length_ptr;
                
                if table_length >= 52 {
                    let table = core::slice::from_raw_parts(tpm2_table_ptr, table_length as usize);
                    
                    let control_address = u64::from_le_bytes([
                        table[36], table[37], table[38], table[39],
                        table[40], table[41], table[42], table[43],
                    ]);
                    let start_method = u32::from_le_bytes([
                        table[44], table[45], table[46], table[47],
                    ]);
                    
                    // Determine interface type based on start method
                    let interface = match start_method {
                        6 => TpmInterface::CRB,  // Command Response Buffer
                        7 => TpmInterface::FIFO, // FIFO
                        _ => TpmInterface::MMIO,
                    };
                    
                    let mut device = TPM_DEVICE.lock();
                    *device = Some(TpmDevice {
                        interface,
                        base_address: PhysAddr::new(control_address),
                        locality: 0,
                        initialized: false,
                    });
                    
                    // Initialize TPM
                    if let Some(ref mut tpm) = *device {
                        if initialize_tpm(tpm).is_ok() {
                            tpm.initialized = true;
                        }
                    }
                }
            }
        }
    }
}

/// Initialize TPM device
fn initialize_tpm(tpm: &mut TpmDevice) -> Result<(), TpmError> {
    // Send TPM2_Startup command
    let startup_cmd = build_startup_command();
    let response = send_command(tpm, &startup_cmd)?;
    
    // Verify response
    if parse_response_code(&response) != TpmResponseCode::Success as u32 {
        return Err(TpmError::InitializationFailed);
    }
    
    Ok(())
}

/// Build TPM2_Startup command
fn build_startup_command() -> Vec<u8> {
    let mut cmd = Vec::with_capacity(16);
    
    // Header: TPM_ST_NO_SESSIONS, size, command code
    cmd.extend_from_slice(&(0x8001u16).to_le_bytes()); // TPM_ST_NO_SESSIONS
    cmd.extend_from_slice(&(12u32).to_le_bytes());     // Size
    cmd.extend_from_slice(&(TpmCommandCode::Startup as u32).to_le_bytes()); // TPM2_Startup
    
    // Startup type: TPM_SU_CLEAR (0x0000)
    cmd.extend_from_slice(&(0x0000u16).to_le_bytes());
    
    cmd
}

/// Send command to TPM
fn send_command(tpm: &TpmDevice, command: &[u8]) -> Result<Vec<u8>, TpmError> {
    match tpm.interface {
        TpmInterface::CRB => send_command_crb(tpm, command),
        TpmInterface::FIFO => send_command_fifo(tpm, command),
        TpmInterface::MMIO => send_command_mmio(tpm, command),
        TpmInterface::None => Err(TpmError::NotAvailable),
    }
}

/// Send command via CRB interface
fn send_command_crb(tpm: &TpmDevice, command: &[u8]) -> Result<Vec<u8>, TpmError> {
    // CRB interface: write command to CRB command buffer, set go bit, wait for completion
    // CRB registers at base_address:
    // - CRB_CTRL_REQ (offset 0x00): Request register
    // - CRB_CTRL_STS (offset 0x04): Status register
    // - CRB_CTRL_DATA_BUFFER (offset 0x80): Command/response buffer
    
    unsafe {
        // Map CRB registers
        let ctrl_req = (tpm.base_address.as_u64() + 0x00) as *mut u32;
        let ctrl_sts = (tpm.base_address.as_u64() + 0x04) as *mut u32;
        let data_buffer = (tpm.base_address.as_u64() + 0x80) as *mut u8;
        
        // Write command to data buffer
        for (i, &byte) in command.iter().enumerate() {
            *data_buffer.add(i) = byte;
        }
        
        // Set request bit (bit 0 = go)
        *ctrl_req = 1;
        
        // Wait for completion (status bit 0 = idle)
        let mut timeout = 1000000;
        while (*ctrl_sts & 1) == 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(TpmError::Timeout);
        }
        
        // Read response size from header
        let resp_size = u32::from_le_bytes([
            *data_buffer.add(2),
            *data_buffer.add(3),
            *data_buffer.add(4),
            *data_buffer.add(5),
        ]);
        
        // Read response
        let mut response = Vec::with_capacity(resp_size as usize);
        for i in 0..resp_size as usize {
            response.push(*data_buffer.add(i));
        }
        
        Ok(response)
    }
}

/// Send command via FIFO interface
fn send_command_fifo(tpm: &TpmDevice, command: &[u8]) -> Result<Vec<u8>, TpmError> {
    // FIFO interface: write to FIFO register, wait for completion
    // FIFO registers at base_address:
    // - TPM_ACCESS (offset 0x00): Access register
    // - TPM_STS (offset 0x01): Status register
    // - TPM_DATA_FIFO (offset 0x05): Data FIFO
    
    unsafe {
        let access = (tpm.base_address.as_u64() + 0x00) as *mut u8;
        let sts = (tpm.base_address.as_u64() + 0x01) as *mut u8;
        let fifo = (tpm.base_address.as_u64() + 0x05) as *mut u8;
        
        // Request locality
        *access = 0x02; // Request use
        
        // Wait for granted
        let mut timeout = 1000000;
        while (*access & 0x01) == 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(TpmError::Timeout);
        }
        
        // Write command to FIFO
        for &byte in command {
            // Wait for FIFO ready
            while (*sts & 0x20) == 0 {
                core::hint::spin_loop();
            }
            *fifo = byte;
        }
        
        // Set command ready
        *sts = 0x40;
        
        // Wait for completion
        timeout = 1000000;
        while (*sts & 0x80) == 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(TpmError::Timeout);
        }
        
        // Read response
        let mut response = Vec::new();
        let resp_size = u32::from_le_bytes([
            *fifo, *fifo, *fifo, *fifo,
        ]);
        
        for _ in 0..resp_size {
            while (*sts & 0x10) == 0 {
                core::hint::spin_loop();
            }
            response.push(*fifo);
        }
        
        Ok(response)
    }
}

/// Send command via MMIO interface
fn send_command_mmio(tpm: &TpmDevice, command: &[u8]) -> Result<Vec<u8>, TpmError> {
    // MMIO interface: similar to CRB but uses different register layout
    // MMIO interface uses similar register layout to CRB
    send_command_crb(tpm, command)
}

/// Parse response code from TPM response
fn parse_response_code(response: &[u8]) -> u32 {
    if response.len() >= 10 {
        u32::from_le_bytes([
            response[6], response[7], response[8], response[9],
        ])
    } else {
        0
    }
}

/// Read PCR value
pub fn read_pcr(pcr_index: u8) -> Result<Vec<u8>, TpmError> {
    let device = TPM_DEVICE.lock();
    if let Some(ref tpm) = *device {
        if !tpm.initialized {
            return Err(TpmError::NotInitialized);
        }
        
        // Build TPM2_ReadPCR command
        let mut cmd = Vec::with_capacity(20);
        cmd.extend_from_slice(&(0x8001u16).to_le_bytes()); // TPM_ST_NO_SESSIONS
        cmd.extend_from_slice(&(16u32).to_le_bytes());      // Size
        cmd.extend_from_slice(&(TpmCommandCode::ReadPCR as u32).to_le_bytes()); // TPM2_ReadPCR
        cmd.push(pcr_index); // PCR index
        cmd.extend_from_slice(&(0u16).to_le_bytes()); // Reserved
        
        let response = send_command(tpm, &cmd)?;
        
        if parse_response_code(&response) != TpmResponseCode::Success as u32 {
            return Err(TpmError::CommandFailed);
        }
        
        // Extract PCR value (32 bytes for SHA-256)
        if response.len() >= 42 {
            Ok(response[10..42].to_vec())
        } else {
            Err(TpmError::InvalidResponse)
        }
    } else {
        Err(TpmError::NotAvailable)
    }
}

/// Generate TPM quote
pub fn generate_quote(pcr_selection: &[u8], nonce: &[u8]) -> Result<Vec<u8>, TpmError> {
    let device = TPM_DEVICE.lock();
    if let Some(ref tpm) = *device {
        if !tpm.initialized {
            return Err(TpmError::NotInitialized);
        }
        
        // Build TPM2_Quote command
        // TPM2_Quote command structure:
        // Header: tag(2), size(4), command_code(4)
        // Handle: quote_handle(4)
        // Auth: auth_session(4), auth_size(4), auth_data(variable)
        // PCR selection: pcr_select_count(4), pcr_select(variable)
        // Nonce: nonce_size(2), nonce(variable)
        
        let mut cmd = Vec::with_capacity(128);
        cmd.extend_from_slice(&(0x8002u16).to_le_bytes()); // TPM_ST_SESSIONS
        cmd.extend_from_slice(&(0u32).to_le_bytes());       // Size (will update later)
        cmd.extend_from_slice(&(TpmCommandCode::Quote as u32).to_le_bytes()); // TPM2_Quote
        
        // Quote handle (TPM_RH_PLATFORM = 0x40000007)
        cmd.extend_from_slice(&(0x40000007u32).to_le_bytes());
        
        // Auth session (TPM_RS_PW = 0x40000009 for password session)
        cmd.extend_from_slice(&(0x40000009u32).to_le_bytes());
        cmd.extend_from_slice(&(0u32).to_le_bytes()); // auth_size = 0 (no auth)
        
        // PCR selection (select all 24 PCRs)
        cmd.extend_from_slice(&(1u32).to_le_bytes()); // pcr_select_count = 1
        cmd.extend_from_slice(&(0u8).to_le_bytes());  // hash algorithm (TPM_ALG_SHA256 = 0x000B)
        cmd.extend_from_slice(&(0x0Bu16).to_le_bytes());
        cmd.extend_from_slice(&(3u8).to_le_bytes());  // size of select (3 bytes for 24 PCRs)
        cmd.extend_from_slice(&[0xFFu8, 0xFFu8, 0xFFu8]); // select all 24 PCRs
        
        // Nonce
        let nonce_size = nonce.len().min(32) as u16;
        cmd.extend_from_slice(&nonce_size.to_le_bytes());
        cmd.extend_from_slice(&nonce[0..nonce_size as usize]);
        
        // Update command size
        let cmd_size = cmd.len() as u32;
        cmd[2..6].copy_from_slice(&cmd_size.to_le_bytes());
        
        let response = send_command(tpm, &cmd)?;
        
        if parse_response_code(&response) != TpmResponseCode::Success as u32 {
            return Err(TpmError::CommandFailed);
        }
        
        Ok(response)
    } else {
        Err(TpmError::NotAvailable)
    }
}

/// Check if TPM is available
pub fn is_available() -> bool {
    let device = TPM_DEVICE.lock();
    device.as_ref().map(|tpm| tpm.initialized).unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub enum TpmError {
    NotAvailable,
    NotInitialized,
    Timeout,
    CommandFailed,
    InvalidResponse,
    InitializationFailed,
    NotImplemented,
}

