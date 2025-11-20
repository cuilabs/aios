//! ACPI (Advanced Configuration and Power Interface)
//! 
//! Provides system configuration and power management

use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

/// ACPI table header
#[repr(C, packed)]
pub struct ACPITableHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: [u8; 4],
    pub creator_revision: u32,
}

/// ACPI table entry
#[cfg(feature = "alloc")]
struct ACPITableEntry {
    signature: [u8; 4],
    address: *const u8,
    length: u32,
}

/// ACPI manager
#[cfg(feature = "alloc")]
struct ACIManager {
    tables: BTreeMap<[u8; 4], ACPITableEntry>,
    rsdp_address: Option<*const u8>,
}

#[cfg(feature = "alloc")]
static ACPI_MANAGER: Mutex<Option<ACIManager>> = Mutex::new(None);

/// RSDP (Root System Description Pointer) signature
const RSDP_SIGNATURE: &[u8] = b"RSD PTR ";

/// Initialize ACPI
pub fn init() {
    #[cfg(feature = "alloc")]
    {
        let mut manager = ACPI_MANAGER.lock();
        *manager = Some(ACIManager {
            tables: BTreeMap::new(),
            rsdp_address: None,
        });
        
        // Find RSDP
        if let Some(rsdp) = find_rsdp() {
            let mut manager = ACPI_MANAGER.lock();
            if let Some(ref mut manager) = *manager {
                manager.rsdp_address = Some(rsdp);
                
                // Parse RSDT/XSDT
                parse_rsdt(rsdp);
            }
        }
    }
}

/// Find RSDP (Root System Description Pointer)
#[cfg(feature = "alloc")]
fn find_rsdp() -> Option<*const u8> {
    // Search in BIOS memory area (0xE0000-0xFFFFF) for RSDP signature "RSD PTR "
    // RSDP signature is 8 bytes: "RSD PTR " (with space)
    const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
    
    unsafe {
        // Search in BIOS memory area
        for addr in (0xE0000..0xFFFFF).step_by(16) {
            let ptr = addr as *const u8;
            let signature = core::slice::from_raw_parts(ptr, 8);
            
            if signature == RSDP_SIGNATURE {
                // Verify checksum (first 20 bytes for ACPI 1.0, 36 bytes for ACPI 2.0+)
                let version = *ptr.add(15); // ACPI version at offset 15
                let checksum_len = if version >= 2 { 36 } else { 20 };
                
                let mut sum: u8 = 0;
                for i in 0..checksum_len {
                    sum = sum.wrapping_add(*ptr.add(i));
                }
                
                if sum == 0 {
                    return Some(ptr);
                }
            }
        }
    }
    
    None
}

/// Parse RSDT/XSDT (Root System Description Table)
#[cfg(feature = "alloc")]
fn parse_rsdt(rsdp: *const u8) {
    unsafe {
        let mut manager = ACPI_MANAGER.lock();
        if let Some(ref mut manager) = *manager {
            // Read RSDT/XSDT address from RSDP
            let version = *rsdp.add(15); // ACPI version at offset 15
            
            if version >= 2 {
                // ACPI 2.0+: Use XSDT (64-bit addresses)
                let xsdt_addr_ptr = rsdp.add(24) as *const u64;
                let xsdt_addr = *xsdt_addr_ptr;
                
                if xsdt_addr != 0 {
                    let xsdt = xsdt_addr as *const ACPITableHeader;
                    let header = &*xsdt;
                    let entry_count = ((header.length - 36) / 8) as usize; // 36 = header size, 8 = 64-bit entry
                    
                    // Parse XSDT entries
                    for i in 0..entry_count {
                        let entry_ptr = xsdt.add(36 + (i * 8)) as *const u64;
                        let table_addr = *entry_ptr;
                        
                        if table_addr != 0 {
                            let table = table_addr as *const ACPITableHeader;
                            let table_header = &*table;
                            
                            // Store table by signature
                            let signature: [u8; 4] = [
                                table_header.signature[0],
                                table_header.signature[1],
                                table_header.signature[2],
                                table_header.signature[3],
                            ];
                            
                            manager.tables.insert(signature, ACPITableEntry {
                                signature,
                                address: table_addr as *const u8,
                                length: table_header.length,
                            });
                        }
                    }
                }
            } else {
                // ACPI 1.0: Use RSDT (32-bit addresses)
                let rsdt_addr_ptr = rsdp.add(16) as *const u32;
                let rsdt_addr = *rsdt_addr_ptr as u64;
                
                if rsdt_addr != 0 {
                    let rsdt = rsdt_addr as *const ACPITableHeader;
                    let header = &*rsdt;
                    let entry_count = ((header.length - 36) / 4) as usize; // 36 = header size, 4 = 32-bit entry
                    
                    // Parse RSDT entries
                    for i in 0..entry_count {
                        let entry_ptr = rsdt.add(36 + (i * 4)) as *const u32;
                        let table_addr = *entry_ptr as u64;
                        
                        if table_addr != 0 {
                            let table = table_addr as *const ACPITableHeader;
                            let table_header = &*table;
                            
                            // Store table by signature
                            let signature: [u8; 4] = [
                                table_header.signature[0],
                                table_header.signature[1],
                                table_header.signature[2],
                                table_header.signature[3],
                            ];
                            
                            manager.tables.insert(signature, ACPITableEntry {
                                signature,
                                address: table_addr as *const u8,
                                length: table_header.length,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Get ACPI table
pub fn get_table(signature: &[u8; 4]) -> Option<*const u8> {
    #[cfg(feature = "alloc")]
    {
        let manager = ACPI_MANAGER.lock();
        if let Some(ref manager) = *manager {
            manager.tables.get(signature).map(|entry| entry.address)
        } else {
            None
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        None
    }
}

/// Get FADT (Fixed ACPI Description Table)
pub fn get_fadt() -> Option<*const u8> {
    get_table(b"FACP")
}

/// Get MADT (Multiple APIC Description Table)
pub fn get_madt() -> Option<*const u8> {
    get_table(b"APIC")
}

/// Get DSDT (Differentiated System Description Table)
pub fn get_dsdt() -> Option<*const u8> {
    // DSDT is referenced from FADT
    // Read DSDT address from FADT
    if let Some(fadt_ptr) = get_fadt() {
        unsafe {
            let fadt = fadt_ptr as *const u8;
            let revision = *fadt.add(1); // FADT revision
            
            if revision >= 2 {
                // ACPI 2.0+: 64-bit DSDT address at offset 0x88
                let dsdt_addr_ptr = fadt.add(0x88) as *const u64;
                let dsdt_addr = *dsdt_addr_ptr;
                if dsdt_addr != 0 {
                    return Some(dsdt_addr as *const u8);
                }
            } else {
                // ACPI 1.0: 32-bit DSDT address at offset 0x28
                let dsdt_addr_ptr = fadt.add(0x28) as *const u32;
                let dsdt_addr = *dsdt_addr_ptr as u64;
                if dsdt_addr != 0 {
                    return Some(dsdt_addr as *const u8);
                }
            }
        }
    }
    
    None
}

/// Check if ACPI is available
pub fn is_available() -> bool {
    #[cfg(feature = "alloc")]
    {
        let manager = ACPI_MANAGER.lock();
        if let Some(ref manager) = *manager {
            manager.rsdp_address.is_some()
        } else {
            false
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        false
    }
}

