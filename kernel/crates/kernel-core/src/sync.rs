//! Synchronization primitives

use core::fmt;

/// Print to kernel console
/// 
/// Placeholder - will be implemented with VGA text mode or serial port
pub fn println(_args: fmt::Arguments) {
    // Simple print implementation
    // In production, use proper VGA text mode or serial port
    // For now, this is a placeholder that does nothing
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::sync::println(format_args!($($arg)*));
    };
}
