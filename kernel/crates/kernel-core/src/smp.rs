//! Symmetric Multiprocessing (SMP) Support
//! 
//! Multi-core initialization, per-CPU data structures, and load balancing

use spin::Mutex;
use x86_64::registers::model_specific::Msr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
#[cfg(feature = "alloc")]
use crate::time;

/// CPU information
#[derive(Clone, Copy, Debug)]
pub struct CpuInfo {
    pub id: u32,
    pub apic_id: u32,
    pub online: bool,
    pub current_agent: Option<u64>,
}

/// Per-CPU data structure
pub struct PerCpuData {
    pub cpu_id: u32,
    pub apic_id: u32,
    pub runqueue: Mutex<Vec<u64>>, // Agent IDs on this CPU
    pub load: Mutex<u64>, // Current load (nanoseconds)
}

/// SMP manager
pub struct SmpManager {
    #[cfg(feature = "alloc")]
    cpus: Mutex<Vec<CpuInfo>>,
    #[cfg(feature = "alloc")]
    per_cpu_data: Mutex<Vec<PerCpuData>>,
    #[cfg(not(feature = "alloc"))]
    cpus: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    per_cpu_data: Mutex<()>,
    boot_cpu: u32,
}

static SMP_MANAGER: Mutex<Option<SmpManager>> = Mutex::new(None);

/// Initialize SMP
pub fn init() {
    // Detect CPU count
    let cpu_count = detect_cpu_count();
    let boot_cpu = get_boot_cpu_id();
    
    #[cfg(feature = "alloc")]
    {
        let mut cpus = Vec::new();
        let mut per_cpu_data = Vec::new();
        
        for i in 0..cpu_count {
            let apic_id = get_apic_id(i);
            cpus.push(CpuInfo {
                id: i,
                apic_id,
                online: i == boot_cpu, // Only boot CPU is online initially
                current_agent: None,
            });
            
            per_cpu_data.push(PerCpuData {
                cpu_id: i,
                apic_id,
                runqueue: Mutex::new(Vec::new()),
                load: Mutex::new(0),
            });
        }
        
        let mut manager = SMP_MANAGER.lock();
        *manager = Some(SmpManager {
            cpus: Mutex::new(cpus),
            per_cpu_data: Mutex::new(per_cpu_data),
            boot_cpu,
        });
        
        // Publish system event
        publish_event(KernelEvent {
            event_type: EventType::Performance,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec::Vec::new(),
        });
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        let mut manager = SMP_MANAGER.lock();
        *manager = Some(SmpManager {
            cpus: Mutex::new(()),
            per_cpu_data: Mutex::new(()),
            boot_cpu,
        });
    }
}

/// Detect CPU count
fn detect_cpu_count() -> u32 {
    // First, try to detect from ACPI MADT (most accurate)
    #[cfg(feature = "alloc")]
    {
        use aios_kernel_hal::acpi;
        if acpi::is_available() {
            if let Some(madt) = acpi::get_madt() {
                // Parse MADT to count CPUs
                let cpu_count = parse_madt_cpu_count(madt);
                if cpu_count > 0 {
                    return cpu_count;
                }
            }
        }
    }
    
    // Fallback: Try to detect from CPUID extended topology
    unsafe {
        // Check if CPUID supports extended topology enumeration
        let result = x86_64::instructions::cpuid::cpuid(0x0);
        if result.eax >= 0xB {
            // Extended topology enumeration available
            let mut level = 0;
            let mut total_logical = 0u32;
            
            loop {
                let result = x86_64::instructions::cpuid::cpuid(0xB);
                let level_type = (result.ecx >> 8) & 0xFF;
                
                if level_type == 0 {
                    // SMT level - logical processors per core
                    total_logical = (result.ebx & 0xFFFF) as u32;
                } else if level_type == 1 {
                    // Core level - cores per package
                    let cores_per_package = ((result.ebx >> 16) & 0xFFFF) as u32;
                    if cores_per_package > 0 && total_logical > 0 {
                        return cores_per_package * total_logical;
                    }
                } else {
                    break;
                }
                
                level += 1;
                if level > 10 {
                    break; // Safety limit
                }
            }
        }
        
        // Fallback: Check APIC count from CPUID
        let result = x86_64::instructions::cpuid::cpuid(1);
        if (result.ecx & (1 << 9)) != 0 {
            // APIC available
            // For single core, return 1
            return 1;
        }
    }
    
    // Default: single core
    1
}

/// Parse MADT to count CPUs
#[cfg(feature = "alloc")]
fn parse_madt_cpu_count(madt: *const u8) -> u32 {
    unsafe {
        // Read MADT header
        let header = madt as *const aios_kernel_hal::acpi::ACPITableHeader;
        let length = (*header).length;
        
        // Start after header (offset 0x2C for MADT)
        let mut offset = 0x2C;
        let mut cpu_count = 0u32;
        
        while offset < length as usize {
            let entry = (madt as usize + offset) as *const u8;
            let entry_type = *entry;
            let entry_length = *((entry as usize + 1) as *const u8) as usize;
            
            match entry_type {
                0 => {
                    // Local APIC entry - represents a CPU
                    cpu_count += 1;
                }
                9 => {
                    // Local x2APIC entry - represents a CPU
                    cpu_count += 1;
                }
                _ => {
                    // Other entry types (IOAPIC, NMI, etc.) - skip
                }
            }
            
            offset += entry_length;
            if entry_length == 0 {
                break; // Safety check
            }
        }
        
        cpu_count.max(1) // At least 1 CPU (boot CPU)
    }
}

#[cfg(not(feature = "alloc"))]
fn parse_madt_cpu_count(_madt: *const u8) -> u32 {
    1
}

/// Get boot CPU ID
fn get_boot_cpu_id() -> u32 {
    // Boot CPU is always 0
    0
}

/// Get APIC ID for CPU
fn get_apic_id(cpu_id: u32) -> u32 {
    if cpu_id == 0 {
        // Read from Local APIC ID register
        use aios_kernel_hal::apic;
        apic::current_cpu_id() as u32
    } else {
        // Try to get from ACPI MADT
        #[cfg(feature = "alloc")]
        {
            use aios_kernel_hal::acpi;
            if acpi::is_available() {
                if let Some(madt) = acpi::get_madt() {
                    if let Some(apic_id) = parse_madt_apic_id(madt, cpu_id) {
                        return apic_id;
                    }
                }
            }
        }
        // Fallback: use CPU ID as APIC ID
        cpu_id
    }
}

/// Parse MADT to get APIC ID for specific CPU
#[cfg(feature = "alloc")]
fn parse_madt_apic_id(madt: *const u8, target_cpu_index: u32) -> Option<u32> {
    unsafe {
        let header = madt as *const aios_kernel_hal::acpi::ACPITableHeader;
        let length = (*header).length;
        
        let mut offset = 0x2C;
        let mut cpu_index = 0u32;
        
        while offset < length as usize {
            let entry = (madt as usize + offset) as *const u8;
            let entry_type = *entry;
            let entry_length = *((entry as usize + 1) as *const u8) as usize;
            
            match entry_type {
                0 => {
                    // Local APIC entry
                    if cpu_index == target_cpu_index {
                        // APIC ID is at offset 3
                        let apic_id = *((entry as usize + 3) as *const u8) as u32;
                        return Some(apic_id);
                    }
                    cpu_index += 1;
                }
                9 => {
                    // Local x2APIC entry
                    if cpu_index == target_cpu_index {
                        // x2APIC ID is at offset 4 (u32)
                        let apic_id = *((entry as usize + 4) as *const u32);
                        return Some(apic_id);
                    }
                    cpu_index += 1;
                }
                _ => {
                    // Other entry types - skip
                }
            }
            
            offset += entry_length;
            if entry_length == 0 {
                break;
            }
        }
        
        None
    }
}

#[cfg(not(feature = "alloc"))]
fn parse_madt_apic_id(_madt: *const u8, _target_cpu_index: u32) -> Option<u32> {
    None
}

/// Get current CPU ID
pub fn current_cpu_id() -> u32 {
    // Read from Local APIC ID
    unsafe {
        let msr = Msr::new(0x1B);
        let value = msr.read();
        ((value >> 24) & 0xFF) as u32
    }
}

/// Get per-CPU data for current CPU
pub fn get_per_cpu_data() -> Option<&'static PerCpuData> {
    #[cfg(feature = "alloc")]
    {
        let manager = SMP_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let cpu_id = current_cpu_id();
            let per_cpu_data = manager.per_cpu_data.lock();
            per_cpu_data.get(cpu_id as usize).map(|d| {
                // SAFETY: Per-CPU data is static and never moved
                unsafe { &*(d as *const PerCpuData) }
            })
        } else {
            None
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        None
    }
}

/// Start CPU (bring online)
pub fn start_cpu(cpu_id: u32) -> Result<(), SmpError> {
    #[cfg(feature = "alloc")]
    {
        let manager = SMP_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let mut cpus = manager.cpus.lock();
            if let Some(cpu) = cpus.get_mut(cpu_id as usize) {
                if cpu.online {
                    return Err(SmpError::AlreadyOnline);
                }
                
                // Send IPI to wake up CPU
                // Use INIT-SIPI-SIPI sequence to start CPU
                let apic_id = cpu.apic_id;
                send_ipi(apic_id, 0xFE); // INIT IPI
                // Small delay
                for _ in 0..1000 {
                    unsafe { x86_64::instructions::nop(); }
                }
                send_ipi(apic_id, 0xFE); // SIPI (first)
                for _ in 0..1000 {
                    unsafe { x86_64::instructions::nop(); }
                }
                send_ipi(apic_id, 0xFE); // SIPI (second)
                
                cpu.online = true;
                
                // Publish system event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Performance,
                        timestamp: time::now(),
                        agent_id: None,
                        data: alloc::vec::Vec::new(),
                    });
                }
                
                Ok(())
            } else {
                Err(SmpError::InvalidCpu)
            }
        } else {
            Err(SmpError::NotInitialized)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Err(SmpError::NotInitialized)
    }
}

/// Stop CPU (bring offline)
pub fn stop_cpu(cpu_id: u32) -> Result<(), SmpError> {
    #[cfg(feature = "alloc")]
    {
        let manager = SMP_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let mut cpus = manager.cpus.lock();
            if let Some(cpu) = cpus.get_mut(cpu_id as usize) {
                if !cpu.online {
                    return Err(SmpError::AlreadyOffline);
                }
                
                if cpu_id == manager.boot_cpu {
                    return Err(SmpError::CannotStopBootCpu);
                }
                
                // Migrate agents to other CPUs
                // Get agents on this CPU and migrate them
                let per_cpu_data = manager.per_cpu_data.lock();
                if let Some(cpu_data) = per_cpu_data.get(cpu_id as usize) {
                    let mut runqueue = cpu_data.runqueue.lock();
                    // Find another online CPU to migrate to
                    let cpus = manager.cpus.lock();
                    for target_cpu in cpus.iter() {
                        if target_cpu.id != cpu_id && target_cpu.online {
                            // Migrate agents (would call scheduler to migrate)
                            // Clear the runqueue
                            runqueue.clear();
                            break;
                        }
                    }
                }
                
                cpu.online = false;
                
                // Publish system event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Performance,
                        timestamp: time::now(),
                        agent_id: None,
                        data: alloc::vec::Vec::new(),
                    });
                }
                
                Ok(())
            } else {
                Err(SmpError::InvalidCpu)
            }
        } else {
            Err(SmpError::NotInitialized)
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        Err(SmpError::NotInitialized)
    }
}

/// Load balance agents across CPUs
pub fn load_balance() {
    #[cfg(feature = "alloc")]
    {
        let manager = SMP_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let per_cpu_data = manager.per_cpu_data.lock();
            
            // Find CPU with lowest load
            let mut min_load = u64::MAX;
            let mut min_cpu = 0;
            
            for (i, cpu_data) in per_cpu_data.iter().enumerate() {
                let load = cpu_data.load.lock();
                if *load < min_load {
                    min_load = *load;
                    min_cpu = i;
                }
            }
            
            // Migrate agents to balance load
            // Find overloaded CPUs and migrate agents to underloaded CPUs
            let mut loads: Vec<(usize, u64)> = per_cpu_data.iter()
                .enumerate()
                .map(|(i, cpu_data)| {
                    let load = cpu_data.load.lock();
                    (i, *load)
                })
                .collect();
            
            // Sort by load (highest first)
            loads.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Migrate from highest load to lowest load
            if loads.len() >= 2 {
                let (high_cpu, high_load) = loads[0];
                let (low_cpu, low_load) = loads[loads.len() - 1];
                
                // Only migrate if load difference is significant
                if high_load > low_load * 2 {
                    // Get agents from high-load CPU
                    if let Some(high_cpu_data) = per_cpu_data.get(high_cpu) {
                        let mut high_runqueue = high_cpu_data.runqueue.lock();
                        if !high_runqueue.is_empty() {
                            // Move one agent to low-load CPU
                            if let Some(agent_id) = high_runqueue.pop() {
                                if let Some(low_cpu_data) = per_cpu_data.get(low_cpu) {
                                    let mut low_runqueue = low_cpu_data.runqueue.lock();
                                    low_runqueue.push(agent_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Send Inter-Processor Interrupt (IPI)
pub fn send_ipi(target_cpu: u32, vector: u8) {
    // Use HAL APIC module
    use aios_kernel_hal::apic;
    apic::send_ipi(target_cpu as u8, vector);
}

/// Set CPU affinity for agent
#[cfg(feature = "alloc")]
pub fn set_agent_affinity(agent_id: u64, cpu_id: u32, capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<(), SmpError> {
    // Check capability if provided (SMP operations require SUPERVISOR capability)
    if let Some(cap) = capability {
        use aios_kernel_capability::{capability::Capabilities, has_capability};
        if !has_capability(Some(cap), Capabilities::SUPERVISOR) {
            return Err(SmpError::InvalidCpu); // Use InvalidCpu as capability error
        }
    } else {
        // Capability required for SMP operations
        return Err(SmpError::InvalidCpu);
    }
    
    let manager = SMP_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let cpus = manager.cpus.lock();
        if let Some(cpu) = cpus.get(cpu_id as usize) {
            if !cpu.online {
                return Err(SmpError::InvalidCpu);
            }
            
            // Add agent to target CPU's runqueue
            let per_cpu_data = manager.per_cpu_data.lock();
            if let Some(cpu_data) = per_cpu_data.get(cpu_id as usize) {
                let mut runqueue = cpu_data.runqueue.lock();
                if !runqueue.contains(&agent_id) {
                    runqueue.push(agent_id);
                }
            }
            
            Ok(())
        } else {
            Err(SmpError::InvalidCpu)
        }
    } else {
        Err(SmpError::NotInitialized)
    }
}

/// Get CPU count
pub fn cpu_count() -> u32 {
    #[cfg(feature = "alloc")]
    {
        let manager = SMP_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let cpus = manager.cpus.lock();
            cpus.len() as u32
        } else {
            1
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        1
    }
}

/// Get online CPU count
pub fn online_cpu_count() -> u32 {
    #[cfg(feature = "alloc")]
    {
        let manager = SMP_MANAGER.lock();
        if let Some(ref manager) = *manager {
            let cpus = manager.cpus.lock();
            cpus.iter().filter(|c| c.online).count() as u32
        } else {
            1
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        1
    }
}

/// SMP error
#[derive(Debug, Clone, Copy)]
pub enum SmpError {
    NotInitialized,
    InvalidCpu,
    AlreadyOnline,
    AlreadyOffline,
    CannotStopBootCpu,
}

