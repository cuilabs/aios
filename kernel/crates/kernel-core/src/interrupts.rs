//! Interrupt handling
//! 
//! Handles hardware interrupts, PIC/APIC setup, and interrupt routing

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Interrupt handler callback
pub type InterruptHandler = fn(InterruptStackFrame) -> bool; // Returns true if handled

/// Interrupt statistics
struct InterruptStats {
    count: u64,
    last_time: u64,
    max_latency: u64,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

#[cfg(feature = "alloc")]
static INTERRUPT_HANDLERS: Mutex<BTreeMap<u8, Vec<InterruptHandler>>> = Mutex::new(BTreeMap::new());
#[cfg(not(feature = "alloc"))]
static INTERRUPT_HANDLERS: Mutex<()> = Mutex::new(());

#[cfg(feature = "alloc")]
static INTERRUPT_STATS: Mutex<BTreeMap<u8, InterruptStats>> = Mutex::new(BTreeMap::new());
#[cfg(not(feature = "alloc"))]
static INTERRUPT_STATS: Mutex<()> = Mutex::new(());

/// Initialize interrupt descriptor table
pub fn init() {
    unsafe {
        // Set up exception handlers (0-31)
        setup_exception_handlers();
        
        // Set up hardware interrupt handlers (32-255)
        setup_hardware_interrupts();
        
        // Initialize PIC or APIC
        init_interrupt_controller();
        
        IDT.load();
    }
}

/// Set up exception handlers (0-31)
unsafe fn setup_exception_handlers() {
    // Exceptions are handled in exceptions.rs
    // This is just for hardware interrupts
}

/// Set up hardware interrupt handlers (32-255)
unsafe fn setup_hardware_interrupts() {
    // Timer interrupt (IRQ 0 -> INT 32)
    IDT[32].set_handler_fn(timer_interrupt_handler);
    
    // Keyboard interrupt (IRQ 1 -> INT 33)
    IDT[33].set_handler_fn(keyboard_interrupt_handler);
    
    // Network interrupt (IRQ 11 -> INT 43)
    IDT[43].set_handler_fn(network_interrupt_handler);
    
    // Default handler for unhandled interrupts
    for i in 32..256 {
        if IDT[i].get_handler().is_none() {
            IDT[i].set_handler_fn(default_interrupt_handler);
        }
    }
}

/// Initialize interrupt controller (PIC or APIC)
fn init_interrupt_controller() {
    // Try APIC first (modern systems)
    if has_apic() {
        init_apic();
    } else {
        // Fall back to PIC (legacy)
        init_pic();
    }
}

/// Check if APIC is available
fn has_apic() -> bool {
    // Check CPUID for APIC support
    unsafe {
        let result = x86_64::instructions::cpuid::cpuid(1);
        (result.ecx & (1 << 9)) != 0 // APIC bit
    }
}

/// Initialize APIC
fn init_apic() {
    // TODO: Initialize Local APIC and I/O APIC
    // This requires reading ACPI tables
}

/// Initialize PIC (Programmable Interrupt Controller)
fn init_pic() {
    unsafe {
        // Remap PIC interrupts to 32-47
        let a1: u8 = 0x11; // ICW1: Initialize
        let a2: u8 = 0x20; // ICW2: Master PIC vector offset
        let a3: u8 = 0x04; // ICW3: Master PIC has slave at IRQ2
        let a4: u8 = 0x01; // ICW4: 8086 mode
        
        // Initialize master PIC
        x86_64::instructions::port::Port::<u8>::new(0x20).write(a1);
        x86_64::instructions::port::Port::<u8>::new(0x21).write(a2);
        x86_64::instructions::port::Port::<u8>::new(0x21).write(a3);
        x86_64::instructions::port::Port::<u8>::new(0x21).write(a4);
        
        // Initialize slave PIC
        let a2: u8 = 0x28; // ICW2: Slave PIC vector offset
        x86_64::instructions::port::Port::<u8>::new(0xA0).write(a1);
        x86_64::instructions::port::Port::<u8>::new(0xA1).write(a2);
        x86_64::instructions::port::Port::<u8>::new(0xA1).write(a3);
        x86_64::instructions::port::Port::<u8>::new(0xA1).write(a4);
        
        // Mask all interrupts initially
        x86_64::instructions::port::Port::<u8>::new(0x21).write(0xFF);
        x86_64::instructions::port::Port::<u8>::new(0xA1).write(0xFF);
    }
}

/// Register interrupt handler
#[cfg(feature = "alloc")]
pub fn register_handler(irq: u8, handler: InterruptHandler) {
    let mut handlers = INTERRUPT_HANDLERS.lock();
    handlers.entry(irq).or_insert_with(Vec::new).push(handler);
}

#[cfg(not(feature = "alloc"))]
pub fn register_handler(_irq: u8, _handler: InterruptHandler) {
    // No-op without alloc
}

/// Timer interrupt handler (IRQ 0)
extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Update time
    crate::time::process_timers();
    
    // Call registered handlers
    call_handlers(0, stack_frame);
    
    // Send EOI (End of Interrupt)
    send_eoi(0);
}

/// Keyboard interrupt handler (IRQ 1)
extern "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Read keyboard scan code
    let scan_code = unsafe {
        x86_64::instructions::port::Port::<u8>::new(0x60).read()
    };
    
    // Call registered handlers
    call_handlers(1, stack_frame);
    
    // Send EOI
    send_eoi(1);
}

/// Network interrupt handler (IRQ 11)
extern "x86-interrupt" fn network_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Call registered handlers
    call_handlers(11, stack_frame);
    
    // Send EOI
    send_eoi(11);
}

/// Default interrupt handler
extern "x86-interrupt" fn default_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Spurious interrupt or unhandled
    crate::log::log_warn!("Unhandled interrupt");
}

/// Call registered handlers for IRQ
#[cfg(feature = "alloc")]
fn call_handlers(irq: u8, stack_frame: InterruptStackFrame) {
    let handlers = INTERRUPT_HANDLERS.lock();
    if let Some(handlers) = handlers.get(&irq) {
        for handler in handlers.iter() {
            handler(stack_frame);
        }
    }
    
    // Update statistics
    update_stats(irq);
}

#[cfg(not(feature = "alloc"))]
fn call_handlers(_irq: u8, _stack_frame: InterruptStackFrame) {
    // No-op without alloc
}

/// Update interrupt statistics
#[cfg(feature = "alloc")]
fn update_stats(irq: u8) {
    let mut stats = INTERRUPT_STATS.lock();
    let entry = stats.entry(irq).or_insert_with(|| InterruptStats {
        count: 0,
        last_time: 0,
        max_latency: 0,
    });
    
    let now = crate::time::now();
    entry.count += 1;
    
    if entry.last_time > 0 {
        let latency = now - entry.last_time;
        if latency > entry.max_latency {
            entry.max_latency = latency;
        }
    }
    
    entry.last_time = now;
}

#[cfg(not(feature = "alloc"))]
fn update_stats(_irq: u8) {
    // No-op without alloc
}

/// Send End of Interrupt (EOI)
fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            // Slave PIC
            x86_64::instructions::port::Port::<u8>::new(0xA0).write(0x20);
        }
        // Master PIC
        x86_64::instructions::port::Port::<u8>::new(0x20).write(0x20);
    }
}

/// Enable interrupts
pub fn enable() {
    unsafe {
        x86_64::instructions::interrupts::enable();
    }
}

/// Disable interrupts
pub fn disable() {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
}

/// Get interrupt statistics
#[cfg(feature = "alloc")]
pub fn get_stats(irq: u8) -> Option<(u64, u64)> {
    let stats = INTERRUPT_STATS.lock();
    stats.get(&irq).map(|s| (s.count, s.max_latency))
}

#[cfg(not(feature = "alloc"))]
pub fn get_stats(_irq: u8) -> Option<(u64, u64)> {
    None
}
