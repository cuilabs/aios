//! APIC (Advanced Programmable Interrupt Controller)
//! 
//! Handles interrupt routing and CPU communication

use x86_64::registers::model_specific::Msr;
use spin::Mutex;

/// Local APIC base address (default)
const APIC_BASE: u64 = 0xFEE00000;

/// APIC registers
const APIC_ID: u64 = 0x20;
const APIC_VERSION: u64 = 0x30;
const APIC_TPR: u64 = 0x80; // Task Priority Register
const APIC_EOI: u64 = 0xB0; // End of Interrupt
const APIC_SVR: u64 = 0xF0; // Spurious Interrupt Vector
const APIC_ICR_LOW: u64 = 0x300; // Interrupt Command Register (low)
const APIC_ICR_HIGH: u64 = 0x310; // Interrupt Command Register (high)
const APIC_LVT_TIMER: u64 = 0x320; // Local Vector Table - Timer
const APIC_LVT_LINT0: u64 = 0x350; // Local Vector Table - LINT0
const APIC_LVT_LINT1: u64 = 0x360; // Local Vector Table - LINT1

/// APIC state
struct ApicState {
    base: u64,
    enabled: bool,
}

static APIC_STATE: Mutex<Option<ApicState>> = Mutex::new(None);

/// Initialize APIC
pub fn init() {
    // Check if APIC is available
    if !has_apic() {
        return;
    }
    
    // Read APIC base from MSR
    let apic_base = read_apic_base();
    
    // Enable Local APIC
    enable_local_apic(apic_base);
    
    // Configure spurious interrupt vector
    configure_spurious_vector(apic_base);
    
    // Set task priority to 0 (accept all interrupts)
    set_task_priority(apic_base, 0);
    
    let mut state = APIC_STATE.lock();
    *state = Some(ApicState {
        base: apic_base,
        enabled: true,
    });
}

/// Check if APIC is available
fn has_apic() -> bool {
    unsafe {
        let result = x86_64::instructions::cpuid::cpuid(1);
        (result.ecx & (1 << 9)) != 0 // APIC bit
    }
}

/// Read APIC base from MSR
fn read_apic_base() -> u64 {
    unsafe {
        let msr = Msr::new(0x1B); // IA32_APIC_BASE
        let value = msr.read();
        (value & 0xFFFFF000) | APIC_BASE
    }
}

/// Enable Local APIC
fn enable_local_apic(base: u64) {
    unsafe {
        let msr = Msr::new(0x1B); // IA32_APIC_BASE
        let value = msr.read();
        msr.write(value | (1 << 11)); // Set APIC enable bit
    }
}

/// Configure spurious interrupt vector
fn configure_spurious_vector(base: u64) {
    unsafe {
        let svr = (base + APIC_SVR) as *mut u32;
        // Set spurious vector to 0xFF and enable APIC
        *svr = 0x1FF; // Vector 0xFF, APIC enabled
    }
}

/// Set task priority
fn set_task_priority(base: u64, priority: u8) {
    unsafe {
        let tpr = (base + APIC_TPR) as *mut u32;
        *tpr = priority as u32;
    }
}

/// Send End of Interrupt (EOI)
pub fn send_eoi() {
    let state = APIC_STATE.lock();
    if let Some(ref state) = *state {
        if state.enabled {
            unsafe {
                let eoi = (state.base + APIC_EOI) as *mut u32;
                *eoi = 0; // Write 0 to EOI register
            }
        }
    }
}

/// Send IPI (Inter-Processor Interrupt)
pub fn send_ipi(target_cpu: u8, vector: u8) {
    let state = APIC_STATE.lock();
    if let Some(ref state) = *state {
        if state.enabled {
            unsafe {
                // Set destination in ICR high
                let icr_high = (state.base + APIC_ICR_HIGH) as *mut u32;
                *icr_high = (target_cpu as u32) << 24;
                
                // Set vector and delivery mode in ICR low
                let icr_low = (state.base + APIC_ICR_LOW) as *mut u32;
                *icr_low = (vector as u32) | (1 << 14); // Fixed delivery mode
            }
        }
    }
}

/// Get current CPU ID (APIC ID)
pub fn current_cpu_id() -> u8 {
    let state = APIC_STATE.lock();
    if let Some(ref state) = *state {
        if state.enabled {
            unsafe {
                let apic_id = (state.base + APIC_ID) as *const u32;
                ((*apic_id >> 24) & 0xFF) as u8
            }
        } else {
            0
        }
    } else {
        0
    }
}

/// Configure Local APIC timer
pub fn configure_timer(vector: u8, initial_count: u32) {
    let state = APIC_STATE.lock();
    if let Some(ref state) = *state {
        if state.enabled {
            unsafe {
                // Configure LVT timer
                let lvt_timer = (state.base + APIC_LVT_TIMER) as *mut u32;
                // Set vector and one-shot mode
                *lvt_timer = (vector as u32) | (0 << 17); // One-shot mode
                
                // Set initial count
                let initial_count_reg = (state.base + 0x380) as *mut u32;
                *initial_count_reg = initial_count;
            }
        }
    }
}

/// Get APIC version
pub fn get_version() -> u8 {
    let state = APIC_STATE.lock();
    if let Some(ref state) = *state {
        if state.enabled {
            unsafe {
                let version = (state.base + APIC_VERSION) as *const u32;
                (*version & 0xFF) as u8
            }
        } else {
            0
        }
    } else {
        0
    }
}
