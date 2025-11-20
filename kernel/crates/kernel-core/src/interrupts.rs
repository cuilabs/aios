//! Interrupt handling
//! 
//! Complete interrupt handling with PIC/APIC support, ISRs, and statistics

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
#[cfg(feature = "alloc")]
use crate::time;

/// Interrupt handler function type
pub type InterruptHandler = fn(InterruptStackFrame);

/// Interrupt statistics
#[cfg(feature = "alloc")]
struct InterruptStats {
    count: u64,
    last_time: u64,
    max_latency: u64,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

#[cfg(feature = "alloc")]
static INTERRUPT_HANDLERS: Mutex<BTreeMap<u8, Vec<InterruptHandler>>> = Mutex::new(BTreeMap::new());

#[cfg(feature = "alloc")]
static INTERRUPT_STATS: Mutex<BTreeMap<u8, InterruptStats>> = Mutex::new(BTreeMap::new());

/// Initialize interrupt handling
pub fn init() {
    // Initialize IDT
    unsafe {
        // Set up exception handlers (0-31)
        IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler);
        
        // Set up interrupt handlers (32-255)
        IDT[32].set_handler_fn(timer_interrupt_handler);      // IRQ 0
        IDT[33].set_handler_fn(keyboard_interrupt_handler);   // IRQ 1
        IDT[34].set_handler_fn(cascade_interrupt_handler);     // IRQ 2
        IDT[35].set_handler_fn(serial_port_2_handler);         // IRQ 3
        IDT[36].set_handler_fn(serial_port_1_handler);         // IRQ 4
        IDT[37].set_handler_fn(parallel_port_2_handler);      // IRQ 5
        IDT[38].set_handler_fn(floppy_disk_handler);           // IRQ 6
        IDT[39].set_handler_fn(parallel_port_1_handler);        // IRQ 7
        IDT[40].set_handler_fn(rtc_handler);                   // IRQ 8
        IDT[41].set_handler_fn(acpi_handler);                  // IRQ 9
        IDT[42].set_handler_fn(reserved_handler);               // IRQ 10
        IDT[43].set_handler_fn(network_interrupt_handler);     // IRQ 11
        IDT[44].set_handler_fn(mouse_handler);                  // IRQ 12
        IDT[45].set_handler_fn(fpu_handler);                   // IRQ 13
        IDT[46].set_handler_fn(ata_primary_handler);           // IRQ 14
        IDT[47].set_handler_fn(ata_secondary_handler);         // IRQ 15
        
        // Set up NMI handler (vector 2)
        // NMI cannot be masked and has special handling
        IDT.non_maskable_interrupt.set_handler_fn(nmi_handler);
        
        // Set default handler for unhandled interrupts
        for i in 48..256 {
            IDT[i].set_handler_fn(default_interrupt_handler);
        }
        
        IDT.load();
    }
    
    // Initialize PIC or APIC
    if has_apic() {
        init_apic();
    } else {
        init_pic();
    }
}

/// Check if APIC is available
fn has_apic() -> bool {
    unsafe {
        let result = x86_64::instructions::cpuid::cpuid(1);
        (result.ecx & (1 << 9)) != 0 // APIC bit
    }
}

/// Initialize APIC
fn init_apic() {
    // Use HAL APIC module
    aios_kernel_hal::apic::init();
    
    // I/O APIC initialization would require ACPI table parsing
    // Local APIC is used for interrupt handling
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
    
    // Publish timer event
    #[cfg(feature = "alloc")]
    {
        publish_event(KernelEvent {
            event_type: EventType::Performance,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec::Vec::new(),
        });
    }
    
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
    
    // Decode scan code using input manager
    #[cfg(feature = "alloc")]
    {
        use kernel_hal::input;
        if let Some(input_mgr) = input::get() {
            // Queue keyboard event
            // Basic scan code to key code mapping (PS/2 scan code set 1)
            let key_code = match scan_code & 0x7F {
                0x01 => 0x1B, // Escape
                0x02 => 0x31, // 1
                0x03 => 0x32, // 2
                0x04 => 0x33, // 3
                0x05 => 0x34, // 4
                0x06 => 0x35, // 5
                0x07 => 0x36, // 6
                0x08 => 0x37, // 7
                0x09 => 0x38, // 8
                0x0A => 0x39, // 9
                0x0B => 0x30, // 0
                0x0C => 0x2D, // -
                0x0D => 0x3D, // =
                0x0E => 0x08, // Backspace
                0x0F => 0x09, // Tab
                0x10 => 0x51, // Q
                0x11 => 0x57, // W
                0x12 => 0x45, // E
                0x13 => 0x52, // R
                0x14 => 0x54, // T
                0x15 => 0x59, // Y
                0x16 => 0x55, // U
                0x17 => 0x49, // I
                0x18 => 0x4F, // O
                0x19 => 0x50, // P
                0x1A => 0x5B, // [
                0x1B => 0x5D, // ]
                0x1C => 0x0D, // Enter
                0x1E => 0x41, // A
                0x1F => 0x53, // S
                0x20 => 0x44, // D
                0x21 => 0x46, // F
                0x22 => 0x47, // G
                0x23 => 0x48, // H
                0x24 => 0x4A, // J
                0x25 => 0x4B, // K
                0x26 => 0x4C, // L
                0x27 => 0x3B, // ;
                0x28 => 0x27, // '
                0x29 => 0x60, // `
                0x2A => 0x10, // Left Shift
                0x2C => 0x5A, // Z
                0x2D => 0x58, // X
                0x2E => 0x43, // C
                0x2F => 0x56, // V
                0x30 => 0x42, // B
                0x31 => 0x4E, // N
                0x32 => 0x4D, // M
                0x33 => 0x2C, // ,
                0x34 => 0x2E, // .
                0x35 => 0x2F, // /
                0x36 => 0x10, // Right Shift
                0x38 => 0x11, // Left Alt
                0x39 => 0x20, // Space
                0x3A => 0x14, // Caps Lock
                0x3B => 0x70, // F1
                0x3C => 0x71, // F2
                0x3D => 0x72, // F3
                0x3E => 0x73, // F4
                0x3F => 0x74, // F5
                0x40 => 0x75, // F6
                0x41 => 0x76, // F7
                0x42 => 0x77, // F8
                0x43 => 0x78, // F9
                0x44 => 0x79, // F10
                0x45 => 0x90, // Num Lock
                0x46 => 0x91, // Scroll Lock
                0x47 => 0x24, // Home (numeric keypad)
                0x48 => 0x26, // Up Arrow
                0x49 => 0x21, // Page Up
                0x4A => 0x2D, // - (numeric keypad)
                0x4B => 0x25, // Left Arrow
                0x4C => 0x0C, // 5 (numeric keypad)
                0x4D => 0x27, // Right Arrow
                0x4E => 0x2B, // + (numeric keypad)
                0x4F => 0x23, // End
                0x50 => 0x28, // Down Arrow
                0x51 => 0x22, // Page Down
                0x52 => 0x2E, // Insert
                0x53 => 0x2E, // Delete
                _ => scan_code as u32, // Unknown key - use scan code as key code
            };
            let pressed = (scan_code & 0x80) == 0; // Bit 7 = 0 for press, 1 for release
            
            let event = input::InputEvent {
                device_id: 1, // Keyboard device
                event_type: if pressed { input::InputEventType::KeyPress } else { input::InputEventType::KeyRelease },
                data: input::InputEventData::Key {
                    keycode: key_code,
                    scancode: scan_code as u32,
                },
                timestamp: time::now(),
            };
            input_mgr.queue_event(event);
        }
    }
    
    #[cfg(not(feature = "alloc"))]
    {
        // No-op when alloc is disabled
    }
    
    // Call registered handlers
    call_handlers(1, stack_frame);
    
    // Send EOI
    send_eoi(1);
}

/// Cascade interrupt handler (IRQ 2)
extern "x86-interrupt" fn cascade_interrupt_handler(stack_frame: InterruptStackFrame) {
    // PIC cascade - no action needed
    call_handlers(2, stack_frame);
    send_eoi(2);
}

/// Serial port 2 handler (IRQ 3)
extern "x86-interrupt" fn serial_port_2_handler(stack_frame: InterruptStackFrame) {
    call_handlers(3, stack_frame);
    send_eoi(3);
}

/// Serial port 1 handler (IRQ 4)
extern "x86-interrupt" fn serial_port_1_handler(stack_frame: InterruptStackFrame) {
    call_handlers(4, stack_frame);
    send_eoi(4);
}

/// Parallel port 2 handler (IRQ 5)
extern "x86-interrupt" fn parallel_port_2_handler(stack_frame: InterruptStackFrame) {
    call_handlers(5, stack_frame);
    send_eoi(5);
}

/// Floppy disk handler (IRQ 6)
extern "x86-interrupt" fn floppy_disk_handler(stack_frame: InterruptStackFrame) {
    call_handlers(6, stack_frame);
    send_eoi(6);
}

/// Parallel port 1 handler (IRQ 7)
extern "x86-interrupt" fn parallel_port_1_handler(stack_frame: InterruptStackFrame) {
    call_handlers(7, stack_frame);
    send_eoi(7);
}

/// RTC handler (IRQ 8)
extern "x86-interrupt" fn rtc_handler(stack_frame: InterruptStackFrame) {
    // RTC periodic interrupt
    // Update system time from RTC
    use crate::time;
    // RTC interrupt triggers time update (time manager reads RTC on demand)
    // Publish time update event
    #[cfg(feature = "alloc")]
    {
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        publish_event(KernelEvent {
            event_type: EventType::System,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec![("event".into(), "rtc_interrupt".into())],
        });
    }
    
    call_handlers(8, stack_frame);
    send_eoi(8);
}

/// ACPI handler (IRQ 9)
extern "x86-interrupt" fn acpi_handler(stack_frame: InterruptStackFrame) {
    // ACPI interrupt
    call_handlers(9, stack_frame);
    send_eoi(9);
}

/// Reserved handler (IRQ 10)
extern "x86-interrupt" fn reserved_handler(stack_frame: InterruptStackFrame) {
    call_handlers(10, stack_frame);
    send_eoi(10);
}

/// Network interrupt handler (IRQ 11)
extern "x86-interrupt" fn network_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Network device interrupt
    // Process network packets from NIC receive ring
    use kernel_hal::network;
    use kernel_net::ip;
    
    // Check for received packets on all network interfaces
    let interfaces = network::list_interfaces();
    for interface in interfaces {
        let mut buffer = [0u8; 1518]; // Max Ethernet frame size
        if let Ok(packet_len) = network::receive_packet(interface.interface_id, &mut buffer) {
            if packet_len > 0 {
                // Process IP packet
                let _ = ip::process_ip_packet(&buffer[..packet_len]);
            }
        }
    }
    
    // Publish network event
    #[cfg(feature = "alloc")]
    {
        publish_event(KernelEvent {
            event_type: EventType::Network,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec::Vec::new(),
        });
    }
    
    call_handlers(11, stack_frame);
    send_eoi(11);
}

/// Mouse handler (IRQ 12)
extern "x86-interrupt" fn mouse_handler(stack_frame: InterruptStackFrame) {
    // PS/2 mouse interrupt
    let data = unsafe {
        x86_64::instructions::port::Port::<u8>::new(0x60).read()
    };
    // Process mouse data
    call_handlers(12, stack_frame);
    send_eoi(12);
}

/// FPU handler (IRQ 13)
extern "x86-interrupt" fn fpu_handler(stack_frame: InterruptStackFrame) {
    // FPU coprocessor error
    call_handlers(13, stack_frame);
    send_eoi(13);
}

/// ATA primary handler (IRQ 14)
extern "x86-interrupt" fn ata_primary_handler(stack_frame: InterruptStackFrame) {
    // ATA primary channel interrupt
    
    // Publish I/O event
    #[cfg(feature = "alloc")]
    {
        publish_event(KernelEvent {
            event_type: EventType::IO,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec::Vec::new(),
        });
    }
    
    call_handlers(14, stack_frame);
    send_eoi(14);
}

/// ATA secondary handler (IRQ 15)
extern "x86-interrupt" fn ata_secondary_handler(stack_frame: InterruptStackFrame) {
    // ATA secondary channel interrupt
    
    // Publish I/O event
    #[cfg(feature = "alloc")]
    {
        publish_event(KernelEvent {
            event_type: EventType::IO,
            timestamp: time::now(),
            agent_id: None,
            data: alloc::vec::Vec::new(),
        });
    }
    
    call_handlers(15, stack_frame);
    send_eoi(15);
}

/// Breakpoint handler
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::log::log_warn!("Breakpoint hit at {:?}", stack_frame.instruction_pointer);
}

/// Double fault handler
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    crate::log::log_fatal!("Double fault at {:?}", stack_frame.instruction_pointer);
    panic!("Double fault");
}

/// Default interrupt handler
extern "x86-interrupt" fn default_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Spurious interrupt or unhandled
    crate::log::log_warn!("Unhandled interrupt at {:?}", stack_frame.instruction_pointer);
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
    // Try APIC first (if available)
    if has_apic() {
        // Use HAL APIC EOI
        aios_kernel_hal::apic::send_eoi();
        return;
    }
    
    // Fall back to PIC
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

/// Get all interrupt statistics
#[cfg(feature = "alloc")]
pub fn get_all_stats() -> BTreeMap<u8, (u64, u64)> {
    let stats = INTERRUPT_STATS.lock();
    stats.iter()
        .map(|(irq, s)| (*irq, (s.count, s.max_latency)))
        .collect()
}

#[cfg(not(feature = "alloc"))]
pub fn get_all_stats() -> BTreeMap<u8, (u64, u64)> {
    BTreeMap::new()
}

/// Mask interrupt (disable)
pub fn mask_interrupt(irq: u8) {
    if has_apic() {
        // APIC interrupt masking would be done via I/O APIC
        // Local APIC doesn't support per-IRQ masking (use LVT masking instead)
        // I/O APIC masking requires ACPI table parsing
    } else {
        // PIC interrupt masking
        unsafe {
            let port = if irq >= 8 { 0xA1 } else { 0x21 };
            let mask = x86_64::instructions::port::Port::<u8>::new(port).read();
            x86_64::instructions::port::Port::<u8>::new(port).write(mask | (1 << (irq % 8)));
        }
    }
}

/// Unmask interrupt (enable)
pub fn unmask_interrupt(irq: u8) {
    if has_apic() {
        // APIC interrupt unmasking would be done via I/O APIC
        // Local APIC doesn't support per-IRQ masking (use LVT masking instead)
        // I/O APIC unmasking requires ACPI table parsing
    } else {
        // PIC interrupt unmasking
        unsafe {
            let port = if irq >= 8 { 0xA1 } else { 0x21 };
            let mask = x86_64::instructions::port::Port::<u8>::new(port).read();
            x86_64::instructions::port::Port::<u8>::new(port).write(mask & !(1 << (irq % 8)));
        }
    }
}
