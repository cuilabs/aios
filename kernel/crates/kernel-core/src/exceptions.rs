//! CPU Exceptions and Fault Domains
//! 
//! Handles CPU exceptions, fault domain isolation, and kill-on-violation policies

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Initialize exception handlers
pub fn init() {
    unsafe {
        // Set up exception handlers
        IDT.double_fault.set_handler_fn(double_fault_handler);
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        IDT.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        IDT.segment_not_present.set_handler_fn(segment_not_present_handler);
        IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        IDT.alignment_check.set_handler_fn(alignment_check_handler);
        
        IDT.load();
    }
}

/// Double fault handler
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    // Double fault is unrecoverable - kernel panic
    panic!("Double fault: {:?}", stack_frame);
}

/// Page fault handler
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address = Cr2::read();
    
    // Check if fault is in agent sandbox
    if let Some(agent_id) = get_agent_for_address(address) {
        // Agent fault - check kill-on-violation policy
        if should_kill_on_violation(agent_id, &error_code) {
            kill_agent(agent_id);
            return;
        }
        
        // Handle page fault for agent
        handle_agent_page_fault(agent_id, address, error_code);
    } else {
        // Kernel fault - panic
        panic!("Page fault in kernel at {:?}: {:?}", address, error_code);
    }
}

/// General protection fault handler
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Check if fault is in agent sandbox
    if let Some(agent_id) = get_agent_for_address(stack_frame.instruction_pointer) {
        // Agent fault - check kill-on-violation policy
        if should_kill_on_violation(agent_id, &error_code) {
            kill_agent(agent_id);
            return;
        }
        
        // Handle GPF for agent
        handle_agent_gpf(agent_id, error_code);
    } else {
        // Kernel fault - panic
        panic!("General protection fault in kernel: {:?}", error_code);
    }
}

/// Stack segment fault handler
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    if let Some(agent_id) = get_agent_for_address(stack_frame.instruction_pointer) {
        if should_kill_on_violation(agent_id, &error_code) {
            kill_agent(agent_id);
            return;
        }
        handle_agent_stack_fault(agent_id, error_code);
    } else {
        panic!("Stack segment fault in kernel: {:?}", error_code);
    }
}

/// Segment not present handler
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    if let Some(agent_id) = get_agent_for_address(stack_frame.instruction_pointer) {
        if should_kill_on_violation(agent_id, &error_code) {
            kill_agent(agent_id);
            return;
        }
        handle_agent_segment_fault(agent_id, error_code);
    } else {
        panic!("Segment not present in kernel: {:?}", error_code);
    }
}

/// Invalid opcode handler
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    if let Some(agent_id) = get_agent_for_address(stack_frame.instruction_pointer) {
        if should_kill_on_violation(agent_id, &()) {
            kill_agent(agent_id);
            return;
        }
        handle_agent_invalid_opcode(agent_id);
    } else {
        panic!("Invalid opcode in kernel");
    }
}

/// Alignment check handler
extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    if let Some(agent_id) = get_agent_for_address(stack_frame.instruction_pointer) {
        if should_kill_on_violation(agent_id, &error_code) {
            kill_agent(agent_id);
            return;
        }
        handle_agent_alignment_fault(agent_id, error_code);
    } else {
        panic!("Alignment check failed in kernel: {:?}", error_code);
    }
}

/// Get agent ID for address (if in agent sandbox)
fn get_agent_for_address(_address: x86_64::VirtAddr) -> Option<u64> {
    // TODO: Check if address is in agent sandbox
    None
}

/// Check if agent should be killed on violation
fn should_kill_on_violation(_agent_id: u64, _error: &dyn core::fmt::Debug) -> bool {
    // TODO: Check kill-on-violation policy for agent
    false
}

/// Kill agent
fn kill_agent(_agent_id: u64) {
    // TODO: Kill agent via agent manager
}

/// Handle page fault for agent
fn handle_agent_page_fault(agent_id: u64, address: x86_64::VirtAddr, error_code: PageFaultErrorCode) {
    // Use virtual memory page fault handler
    match crate::memory::virtual_mem::handle_page_fault(address, error_code, Some(agent_id)) {
        Ok(()) => {
            // Page fault handled successfully
        }
        Err(e) => {
            // Page fault handling failed - kill agent or log error
            crate::log::log_error!("Page fault handling failed for agent {}: {:?}", agent_id, e);
        }
    }
}

/// Handle GPF for agent
fn handle_agent_gpf(_agent_id: u64, _error_code: u64) {
    // TODO: Handle general protection fault
}

/// Handle stack fault for agent
fn handle_agent_stack_fault(_agent_id: u64, _error_code: u64) {
    // TODO: Handle stack segment fault
}

/// Handle segment fault for agent
fn handle_agent_segment_fault(_agent_id: u64, _error_code: u64) {
    // TODO: Handle segment not present
}

/// Handle invalid opcode for agent
fn handle_agent_invalid_opcode(_agent_id: u64) {
    // TODO: Handle invalid opcode
}

/// Handle alignment fault for agent
fn handle_agent_alignment_fault(_agent_id: u64, _error_code: u64) {
    // TODO: Handle alignment check
}

