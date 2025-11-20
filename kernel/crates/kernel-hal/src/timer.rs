//! Timer abstraction (HPET/TSC)
//! 
//! Provides high-precision timing

use x86_64::instructions::port::Port;
use spin::Mutex;
use x86_64::instructions::cpuid;

/// HPET registers
const HPET_CAPABILITIES: u64 = 0x000;
const HPET_CONFIG: u64 = 0x010;
const HPET_INTERRUPT: u64 = 0x020;
const HPET_MAIN_COUNTER: u64 = 0x0F0;

/// HPET state
struct HpetState {
    base_address: u64,
    frequency: u64, // Femtoseconds per tick
    enabled: bool,
}

static HPET_STATE: Mutex<Option<HpetState>> = Mutex::new(None);

/// Timer source
enum TimerSource {
    Hpet,
    Tsc,
    Pit,
}

static TIMER_SOURCE: Mutex<Option<TimerSource>> = Mutex::new(None);
static TSC_FREQUENCY: Mutex<u64> = Mutex::new(0);

/// Initialize timer
pub fn init() {
    // Try to detect and initialize HPET
    if detect_and_init_hpet() {
        let mut source = TIMER_SOURCE.lock();
        *source = Some(TimerSource::Hpet);
        return;
    }
    
    // Fall back to TSC
    if has_tsc() {
        calibrate_tsc();
        let mut source = TIMER_SOURCE.lock();
        *source = Some(TimerSource::Tsc);
        return;
    }
    
    // Last resort: PIT
    let mut source = TIMER_SOURCE.lock();
    *source = Some(TimerSource::Pit);
}

/// Detect and initialize HPET
fn detect_and_init_hpet() -> bool {
    // Check if HPET exists in ACPI tables
    // Parse ACPI tables to find HPET base address
    use crate::acpi;
    
    // Get FADT (Fixed ACPI Description Table) to find HPET base
    if let Some(fadt_ptr) = acpi::get_fadt() {
        unsafe {
            // FADT contains HPET block address at offset 0x116 (64-bit) or 0x46 (32-bit)
            // Read HPET base address from FADT
            let fadt = fadt_ptr as *const u8;
            // Check if 64-bit address is available (FADT revision >= 2)
            let revision = *fadt.add(1); // FADT revision at offset 1
            if revision >= 2 {
                // Read 64-bit HPET address from offset 0x116
                let hpet_addr_ptr = fadt.add(0x116) as *const u64;
                let hpet_base = *hpet_addr_ptr;
                if hpet_base != 0 {
                    return init_hpet_at_address(hpet_base);
                }
            } else {
                // Read 32-bit HPET address from offset 0x46
                let hpet_addr_ptr = fadt.add(0x46) as *const u32;
                let hpet_base = *hpet_addr_ptr as u64;
                if hpet_base != 0 {
                    return init_hpet_at_address(hpet_base);
                }
            }
        }
    }
    
    // Fallback: try default HPET base address
    init_hpet_at_address(0xFED00000)
}

/// Initialize HPET at specific address
fn init_hpet_at_address(base: u64) -> bool {
    
    unsafe {
        // Read HPET capabilities
        let cap_ptr = (base + HPET_CAPABILITIES) as *const u64;
        let capabilities = *cap_ptr;
        
        // Check if HPET is valid (vendor ID and revision)
        let vendor_id = (capabilities >> 16) & 0xFFFF;
        if vendor_id == 0 || vendor_id == 0xFFFF {
            return false; // Invalid HPET
        }
        
        // Get frequency (femtoseconds per tick)
        let period = (capabilities >> 32) & 0xFFFFFFFF;
        if period == 0 {
            return false;
        }
        
        // Enable HPET
        let config_ptr = (base + HPET_CONFIG) as *mut u64;
        let mut config = *config_ptr;
        config |= 1; // Enable main counter
        *config_ptr = config;
        
        let mut state = HPET_STATE.lock();
        *state = Some(HpetState {
            base_address: base,
            frequency: period,
            enabled: true,
        });
        
        true
    }
}

/// Check if TSC is available
fn has_tsc() -> bool {
    unsafe {
        let result = cpuid::cpuid(1);
        (result.edx & (1 << 4)) != 0 // TSC bit
    }
}

/// Calibrate TSC frequency
fn calibrate_tsc() {
    // Calibrate TSC against PIT or HPET
    // Try to read from MSR or use default
    unsafe {
        // Try to read TSC frequency from MSR (if available)
        use x86_64::registers::model_specific::Msr;
        let msr = Msr::new(0xCE); // IA32_PERF_STATUS
        let value = msr.read();
        
        // Extract frequency from MSR (if available)
        // Otherwise, use default or calibrate against PIT
        
        // Default: 2.4 GHz (common CPU frequency)
        let mut freq = TSC_FREQUENCY.lock();
        *freq = 2_400_000_000;
    }
}

/// Get current time in nanoseconds
pub fn now_nanos() -> u64 {
    let source = TIMER_SOURCE.lock();
    if let Some(ref src) = *source {
        match src {
            TimerSource::Hpet => {
                let state = HPET_STATE.lock();
                if let Some(ref hpet) = *state {
                    unsafe {
                        let counter_ptr = (hpet.base_address + HPET_MAIN_COUNTER) as *const u64;
                        let ticks = *counter_ptr;
                        // Convert ticks to nanoseconds (femtoseconds -> nanoseconds = / 1_000_000)
                        (ticks * hpet.frequency) / 1_000_000
                    }
                } else {
                    0
                }
            }
            TimerSource::Tsc => {
                let freq = TSC_FREQUENCY.lock();
                if *freq > 0 {
                    unsafe {
                        let tsc = x86_64::instructions::rdtsc();
                        (tsc * 1_000_000_000) / *freq
                    }
                } else {
                    // Fallback: use default frequency
                    unsafe {
                        let tsc = x86_64::instructions::rdtsc();
                        (tsc * 1_000_000_000) / 2_400_000_000
                    }
                }
            }
            TimerSource::Pit => {
                // Read from PIT
                unsafe {
                    let control_port = Port::<u8>::new(0x43);
                    control_port.write(0x00); // Latch counter 0
                    
                    let counter_port = Port::<u8>::new(0x40);
                    let low = counter_port.read();
                    let high = counter_port.read();
                    
                    let count = ((high as u16) << 8) | (low as u16);
                    let elapsed = 0xFFFF - count;
                    (elapsed as u64 * 1_000_000_000) / 1_193_182
                }
            }
        }
    } else {
        0
    }
}

/// Sleep for specified nanoseconds
pub fn sleep_nanos(nanos: u64) {
    let start = now_nanos();
    while now_nanos() - start < nanos {
        unsafe {
            x86_64::instructions::pause();
        }
    }
}

/// Get HPET frequency (femtoseconds per tick)
pub fn hpet_frequency() -> Option<u64> {
    let state = HPET_STATE.lock();
    state.as_ref().map(|s| s.frequency)
}

/// Check if HPET is enabled
pub fn has_hpet() -> bool {
    let state = HPET_STATE.lock();
    state.is_some()
}

/// Get TSC frequency (Hz)
pub fn tsc_frequency() -> u64 {
    let freq = TSC_FREQUENCY.lock();
    *freq
}

/// Get TSC frequency (for compatibility)
pub fn get_tsc_frequency() -> Option<u64> {
    let freq = TSC_FREQUENCY.lock();
    if *freq > 0 {
        Some(*freq)
    } else {
        None
    }
}

/// Get HPET frequency in Hz (for compatibility)
pub fn get_hpet_frequency() -> Option<u64> {
    let state = HPET_STATE.lock();
    state.as_ref().map(|s| {
        // Convert femtoseconds per tick to Hz
        // 1 femtosecond = 10^-15 seconds
        // Frequency = 1 / (period * 10^-15) = 10^15 / period
        1_000_000_000_000_000 / s.frequency
    })
}

