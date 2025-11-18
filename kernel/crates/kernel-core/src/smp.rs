//! Symmetric Multiprocessing (SMP) Support
//! 
//! Multi-core initialization, per-CPU data structures, and load balancing

use spin::Mutex;
use x86_64::registers::model_specific::Msr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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
    // Read from ACPI or CPUID
    // For now, assume single core (will be detected from ACPI)
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
        unsafe {
            let msr = Msr::new(0x1B); // IA32_APIC_BASE
            let value = msr.read();
            (value >> 24) as u32 & 0xFF
        }
    } else {
        // TODO: Read from ACPI
        cpu_id
    }
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
                
                // TODO: Send IPI to wake up CPU
                cpu.online = true;
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
                
                // TODO: Migrate agents to other CPUs
                cpu.online = false;
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
            
            // TODO: Migrate agents to balance load
        }
    }
}

/// Send Inter-Processor Interrupt (IPI)
pub fn send_ipi(target_cpu: u32, vector: u8) {
    // TODO: Send IPI via Local APIC
    unsafe {
        // Write to Local APIC ICR (Interrupt Command Register)
        let apic_base = 0xFEE00000u64; // Default APIC base
        let icr_low = (apic_base + 0x300) as *mut u32;
        let icr_high = (apic_base + 0x310) as *mut u32;
        
        // Set destination
        *icr_high = (target_cpu as u32) << 24;
        
        // Set vector and delivery mode
        *icr_low = (vector as u32) | (1 << 14); // Fixed delivery
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

