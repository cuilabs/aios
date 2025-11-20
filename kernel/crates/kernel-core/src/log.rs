//! Kernel logging system
//! 
//! Provides kernel logging with levels, serial/VGA output, and stack traces

use core::fmt;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

/// Log levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

/// Log entry
struct LogEntry {
    level: LogLevel,
    message: &'static str,
    timestamp: u64,
}

/// Kernel logger
pub struct Logger {
    level: LogLevel,
    entries: Mutex<alloc::vec::Vec<LogEntry>>,
    max_entries: usize,
}

static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// Initialize kernel logger
pub fn init(level: LogLevel, max_entries: usize) {
    let mut logger = LOGGER.lock();
    *logger = Some(Logger {
        level,
        entries: Mutex::new(alloc::vec::Vec::new()),
        max_entries,
    });
}

/// Log a message
pub fn log(level: LogLevel, message: &'static str) {
    let logger = LOGGER.lock();
    if let Some(ref logger) = *logger {
        if level >= logger.level {
            // Write to serial/VGA
            write_output(level, message);
            
            // Store in log buffer
            #[cfg(feature = "alloc")]
            {
                let mut entries = logger.entries.lock();
                if entries.len() >= logger.max_entries {
                    entries.remove(0);
                }
                entries.push(LogEntry {
                    level,
                    message,
                    timestamp: get_timestamp(),
                });
            }
        }
    }
}

/// Write output to serial/VGA
fn write_output(level: LogLevel, message: &'static str) {
    // Serial port output (COM1)
    unsafe {
        let port = 0x3F8; // COM1
        #[cfg(feature = "alloc")]
        let formatted = format_message(level, message);
        #[cfg(not(feature = "alloc"))]
        let formatted = message;
        
        for byte in formatted.as_bytes() {
            // Wait for transmit buffer to be empty
            while (x86_64::instructions::port::Port::<u8>::new(port + 5).read() & 0x20) == 0 {}
            x86_64::instructions::port::Port::<u8>::new(port).write(*byte);
        }
    }
}

/// Format log message
#[cfg(feature = "alloc")]
fn format_message(level: LogLevel, message: &'static str) -> alloc::string::String {
    let level_str = match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Fatal => "FATAL",
    };
    
    alloc::format!("[{}] {}\n", level_str, message)
}

/// Get current timestamp (nanoseconds since boot)
fn get_timestamp() -> u64 {
    crate::time::now()
}

/// Log macros
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Trace, alloc::format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Debug, alloc::format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Info, alloc::format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Warn, alloc::format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Error, alloc::format!($($arg)*).as_str());
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Fatal, alloc::format!($($arg)*).as_str());
    };
}

/// Print stack trace on panic
pub fn print_stack_trace() {
    // TODO: Implement stack trace printing
    log_error!("Stack trace not yet implemented");
}

