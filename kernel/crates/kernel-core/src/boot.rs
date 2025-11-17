//! Kernel boot and initialization

use crate::sync::println;
use crate::memory;

/// Initialize the kernel
pub fn init() {
    println!("AIOS Kernel v0.1.0 - AI-Native Operating System");
    println!("Initializing kernel...");

    // Initialize memory management
    memory::init();

    println!("Kernel initialized successfully");
    println!("AIOS is ready for agent-first computing");
}

