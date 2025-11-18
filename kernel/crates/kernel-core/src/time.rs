//! Time management
//! 
//! Provides system time, monotonic time, timers, and scheduling timeouts

use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Time source (TSC, HPET, or PIT)
enum TimeSource {
    Tsc,
    Hpet,
    Pit,
}

/// Timer callback
pub type TimerCallback = fn(u64) -> bool; // Returns true if timer should repeat

/// Timer entry
struct Timer {
    id: u64,
    expires_at: u64,      // Nanoseconds since boot
    callback: TimerCallback,
    repeat: bool,
    interval: u64,        // Repeat interval in nanoseconds
}

/// Time manager
pub struct TimeManager {
    source: TimeSource,
    boot_time: u64,       // Nanoseconds since epoch at boot
    monotonic_time: u64,  // Nanoseconds since boot
    frequency: u64,        // Ticks per second
    #[cfg(feature = "alloc")]
    timers: Mutex<BTreeMap<u64, Timer>>,
    #[cfg(not(feature = "alloc"))]
    timers: Mutex<()>,
    next_timer_id: Mutex<u64>,
}

static TIME_MANAGER: Mutex<Option<TimeManager>> = Mutex::new(None);

/// Initialize time management
pub fn init() {
    // Detect best time source
    let source = detect_time_source();
    let frequency = get_frequency(&source);
    
    let mut manager = TIME_MANAGER.lock();
    *manager = Some(TimeManager {
        source,
        boot_time: get_wall_clock_time(),
        monotonic_time: 0,
        frequency,
        #[cfg(feature = "alloc")]
        timers: Mutex::new(BTreeMap::new()),
        #[cfg(not(feature = "alloc"))]
        timers: Mutex::new(()),
        next_timer_id: Mutex::new(1),
    });
}

/// Detect best available time source
fn detect_time_source() -> TimeSource {
    // Try HPET first (most accurate)
    if has_hpet() {
        return TimeSource::Hpet;
    }
    
    // Fall back to TSC
    if has_tsc() {
        return TimeSource::Tsc;
    }
    
    // Last resort: PIT
    TimeSource::Pit
}

/// Check if HPET is available
fn has_hpet() -> bool {
    // TODO: Check ACPI for HPET
    false
}

/// Check if TSC is available
fn has_tsc() -> bool {
    // TSC is available on x86_64
    true
}

/// Get frequency of time source
fn get_frequency(source: &TimeSource) -> u64 {
    match source {
        TimeSource::Tsc => {
            // Read TSC frequency from CPUID or calibrate
            2_400_000_000 // 2.4 GHz default
        }
        TimeSource::Hpet => {
            // Read HPET frequency
            10_000_000 // 10 MHz default
        }
        TimeSource::Pit => {
            // PIT frequency
            1_193_182 // ~1.19 MHz
        }
    }
}

/// Get wall clock time (nanoseconds since epoch)
fn get_wall_clock_time() -> u64 {
    // TODO: Read from RTC or NTP
    0
}

/// Get current monotonic time (nanoseconds since boot)
pub fn now() -> u64 {
    let manager = TIME_MANAGER.lock();
    if let Some(ref manager) = *manager {
        read_time_source(&manager.source, manager.frequency)
    } else {
        0
    }
}

/// Read from time source
fn read_time_source(source: &TimeSource, frequency: u64) -> u64 {
    match source {
        TimeSource::Tsc => {
            // Read TSC
            let tsc = unsafe { x86_64::instructions::rdtsc() };
            // Convert to nanoseconds
            (tsc * 1_000_000_000) / frequency
        }
        TimeSource::Hpet => {
            // TODO: Read HPET counter
            0
        }
        TimeSource::Pit => {
            // TODO: Read PIT counter
            0
        }
    }
}

/// Get system time (wall clock)
pub fn system_time() -> u64 {
    let manager = TIME_MANAGER.lock();
    if let Some(ref manager) = *manager {
        manager.boot_time + now()
    } else {
        0
    }
}

/// Create a timer
#[cfg(feature = "alloc")]
pub fn create_timer(callback: TimerCallback, delay_ns: u64, repeat: bool) -> u64 {
    let manager = TIME_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut next_id = manager.next_timer_id.lock();
        let id = *next_id;
        *next_id += 1;
        
        let expires_at = now() + delay_ns;
        let interval = if repeat { delay_ns } else { 0 };
        
        let mut timers = manager.timers.lock();
        timers.insert(id, Timer {
            id,
            expires_at,
            callback,
            repeat,
            interval,
        });
        
        id
    } else {
        0
    }
}

#[cfg(not(feature = "alloc"))]
pub fn create_timer(_callback: TimerCallback, _delay_ns: u64, _repeat: bool) -> u64 {
    0
}

/// Cancel a timer
pub fn cancel_timer(id: u64) {
    let manager = TIME_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut timers = manager.timers.lock();
        timers.remove(&id);
    }
}

/// Process expired timers (called from timer interrupt)
#[cfg(feature = "alloc")]
pub fn process_timers() {
    let manager = TIME_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let current_time = now();
        let mut timers = manager.timers.lock();
        let mut to_remove = Vec::new();
        let mut to_reschedule = Vec::new();
        
        for (id, timer) in timers.iter() {
            if current_time >= timer.expires_at {
                // Timer expired
                let should_repeat = (timer.callback)(*id);
                
                if timer.repeat && should_repeat {
                    // Reschedule
                    to_reschedule.push((*id, timer.interval));
                } else {
                    // Remove
                    to_remove.push(*id);
                }
            }
        }
        
        // Remove expired timers
        for id in to_remove {
            timers.remove(&id);
        }
        
        // Reschedule repeating timers
        for (id, interval) in to_reschedule {
            if let Some(timer) = timers.get_mut(&id) {
                timer.expires_at = current_time + interval;
            }
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub fn process_timers() {
    // No-op without alloc
}

/// Sleep for specified nanoseconds
pub fn sleep_ns(nanos: u64) {
    let start = now();
    while now() - start < nanos {
        // Spin wait (TODO: Use proper sleep with scheduler)
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

/// Sleep for specified milliseconds
pub fn sleep_ms(millis: u64) {
    sleep_ns(millis * 1_000_000);
}

