//! Kernel boot and initialization

use crate::sync::println;
use crate::memory;
use crate::error_recovery;
use crate::exceptions;
use crate::interrupts;
use crate::time;
use crate::smp;

/// Initialize the kernel
pub fn init() {
    println!("AIOS Kernel v0.1.0 - AI-Native Operating System");
    println!("Initializing kernel...");

    // Initialize error recovery and resilience first
    error_recovery::init();
    println!("  ✓ Error recovery system initialized");
    
    // Initialize capability system (needed by other subsystems)
    #[cfg(feature = "alloc")]
    {
        aios_kernel_capability::init();
        println!("  ✓ Capability system initialized");
    }

    // Initialize time management
    time::init();
    println!("  ✓ Time management initialized");

    // Initialize exception handlers
    exceptions::init();
    println!("  ✓ Exception handlers initialized");

    // Initialize interrupt handling
    interrupts::init();
    println!("  ✓ Interrupt handling initialized");

    // Initialize SMP (multi-core support)
    smp::init();
    println!("  ✓ SMP support initialized");

    // Initialize memory management (must be after time for timestamps)
    memory::init();
    println!("  ✓ Memory management initialized");
    
    // Initialize AI-powered features
    #[cfg(feature = "alloc")]
    {
        use crate::ai_healing;
        ai_healing::init();
        ai_healing::init_maintenance();
        println!("  ✓ AI self-healing initialized");
        
        // Note: AI scheduler is initialized when first agent is added
        // AI memory analyzer is initialized in memory::init()
    }

    println!("Kernel initialized successfully");
    println!("AIOS is ready for agent-first computing");
}

