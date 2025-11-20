//! Synchronization primitives

use core::fmt;

/// Print to kernel console
/// 
/// Implements kernel console output using serial port (COM1)
pub fn println(args: fmt::Arguments) {
    // Kernel console output implementation
    // Uses serial port (COM1) for kernel logging
    // Serial port I/O at 0x3F8 (COM1)
    unsafe {
        use x86_64::instructions::port::Port;
        let mut serial_port = Port::<u8>::new(0x3F8 + 5); // Line Status Register
        
        // Wait for transmit buffer empty
        let mut timeout = 10000;
        while (serial_port.read() & 0x20) == 0 && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
        
        if timeout > 0 {
            // Format and write character by character
            let mut data_port = Port::<u8>::new(0x3F8);
            #[cfg(feature = "alloc")]
            {
                let formatted_str = alloc::format!("{}", args);
                for byte in formatted_str.as_bytes() {
                    // Wait for transmit ready
                    let mut ready_timeout = 10000;
                    while (serial_port.read() & 0x20) == 0 && ready_timeout > 0 {
                        core::hint::spin_loop();
                        ready_timeout -= 1;
                    }
                    if ready_timeout > 0 {
                        data_port.write(*byte);
                    }
                }
            }
            #[cfg(not(feature = "alloc"))]
            {
                // Without alloc, serial output is suppressed (formatting requires alloc)
            }
        }
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::sync::println(format_args!($($arg)*));
    };
}
