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
    // Use HAL timer module
    aios_kernel_hal::timer::has_hpet()
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
            // Read TSC frequency from HAL
            aios_kernel_hal::timer::get_tsc_frequency().unwrap_or(2_400_000_000)
        }
        TimeSource::Hpet => {
            // Read HPET frequency from HAL
            aios_kernel_hal::timer::get_hpet_frequency().unwrap_or(10_000_000)
        }
        TimeSource::Pit => {
            // PIT frequency
            1_193_182 // ~1.19 MHz
        }
    }
}

/// Get wall clock time (nanoseconds since epoch)
fn get_wall_clock_time() -> u64 {
    // Read from RTC (Real-Time Clock)
    // RTC is accessed via I/O ports 0x70 (address) and 0x71 (data)
    unsafe {
        let mut time: u64 = 0;
        
        // Read RTC registers
        // Note: RTC returns BCD format, needs conversion
        let addr_port = x86_64::instructions::port::Port::<u8>::new(0x70);
        let data_port = x86_64::instructions::port::Port::<u8>::new(0x71);
        
        // Read seconds
        addr_port.write(0x00);
        let seconds = bcd_to_binary(data_port.read());
        
        // Read minutes
        addr_port.write(0x02);
        let minutes = bcd_to_binary(data_port.read());
        
        // Read hours
        addr_port.write(0x04);
        let hours = bcd_to_binary(data_port.read());
        
        // Read day
        addr_port.write(0x07);
        let day = bcd_to_binary(data_port.read());
        
        // Read month
        addr_port.write(0x08);
        let month = bcd_to_binary(data_port.read());
        
        // Read year
        addr_port.write(0x09);
        let year = bcd_to_binary(data_port.read()) + 2000; // RTC year is BCD, add 2000 for full year
        
        // Convert to AIOS timestamp (nanoseconds since epoch)
        // Handles UTC timezone (RTC is typically in local time, but we assume UTC for system time)
        time = calculate_aios_timestamp(year, month, day, hours, minutes, seconds);
        
        // Convert to nanoseconds
        time * 1_000_000_000
    }
}

/// Convert BCD to binary
fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Calculate AIOS timestamp (nanoseconds since epoch)
/// Implements proper date/time calculation with leap year handling
fn calculate_aios_timestamp(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> u64 {
    let days_since_epoch = days_since_1970(year, month, day);
    let seconds = (days_since_epoch * 86400) as u64 + 
                  (hour as u64 * 3600) + 
                  (minute as u64 * 60) + 
                  (second as u64);
    seconds
}

/// Calculate days since 1970-01-01 with proper leap year handling
fn days_since_1970(year: u16, month: u8, day: u8) -> u64 {
    // Calculate days with proper leap year handling
    let mut days = 0u64;
    
    // Days from 1970 to year
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    
    // Days from January 1 to month
    let month_days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let month_offset = if month > 0 && month <= 12 {
        let offset = month_days[(month - 1) as usize];
        // Add leap day if month is after February and year is leap year
        if month > 2 && is_leap_year(year) {
            offset + 1
        } else {
            offset
        }
    } else {
        0
    };
    
    days + month_offset as u64 + (day - 1) as u64
}

/// Check if year is a leap year
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
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
fn read_time_source(source: &TimeSource, _frequency: u64) -> u64 {
    // Use HAL timer module for high-precision timing
    aios_kernel_hal::timer::now_nanos()
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
        // Spin wait (proper sleep would use scheduler)
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

/// Sleep for specified milliseconds
pub fn sleep_ms(millis: u64) {
    sleep_ns(millis * 1_000_000);
}

