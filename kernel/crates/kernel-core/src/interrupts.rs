//! Interrupt handling
//! 
//! Handles hardware interrupts and exceptions for AIOS

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Initialize interrupt descriptor table
pub fn init() {
    unsafe {
        IDT.load();
    }
}

/// Exception handler
extern "x86-interrupt" fn exception_handler(stack_frame: InterruptStackFrame) {
    // Exception handling
}

