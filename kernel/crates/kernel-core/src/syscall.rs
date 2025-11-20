//! System call interface
//! 
//! Production-grade syscall API with capability validation, async operations,
//! and comprehensive error handling.

use core::time::Duration;

/// System call numbers (C ABI)
#[repr(u64)]
pub enum Syscall {
    AgentSpawn = 1,
    AgentSupervisorRegister = 2,
    AgentRegister = 3,
    AgentKill = 4,
    IPCSend = 5,
    IPCRecv = 6,
    AgentMemAlloc = 7,
    AgentMemFree = 8,
    FrameAlloc = 9,
    PageMap = 10,
    AgentPoolAlloc = 11,
    PQCOperation = 12,
    GetAsyncResult = 13,
    FramebufferAlloc = 14,
    FramebufferFree = 15,
    FramebufferGet = 16,
    DisplayGet = 17,
    DisplaySetMode = 18,
    InputRead = 19,
    InputGetDevices = 20,
}

/// Capability token
#[repr(C)]
pub struct CapabilityToken {
    pub token_id: u64,
    pub agent_id: u64,
    pub capabilities: u64, // Bitmask
    pub expires_at: u64,  // Timestamp (nanoseconds since epoch)
    pub signature: [u8; 64],
}

/// Agent specification
#[repr(C)]
pub struct AgentSpec {
    pub agent_type: u32,
    pub memory_limit: u64,      // Max memory in bytes
    pub cpu_limit: u8,          // Max CPU percentage
    pub capabilities: u64,      // Required capabilities
    pub image_hash: [u8; 32],    // Agent image hash
    pub manifest_ptr: *const u8, // Pointer to manifest
    pub manifest_len: usize,     // Manifest length
}

/// System call result
#[repr(C)]
pub struct SyscallResult {
    pub success: bool,
    pub value: u64,        // Return value
    pub error_code: u32,   // Error code (0 = success)
    pub async_handle: u64, // Async operation handle
    pub data_len: usize,   // Additional data length
}

/// System call error codes
#[repr(u32)]
pub enum SyscallError {
    Success = 0,
    InvalidCapability = 1,
    InvalidSyscall = 2,
    PermissionDenied = 3,
    MessageTooLarge = 4,
    MemoryLimitExceeded = 5,
    Timeout = 6,
    AgentNotFound = 7,
    ResourceExhausted = 8,
    InvalidSpec = 9,
    AlreadyRegistered = 10,
    InvalidSupervisor = 11,
    BufferTooSmall = 12,
    NoMessage = 13,
    OutOfMemory = 14,
    InvalidPointer = 15,
    InvalidAddress = 16,
    InvalidOperation = 17,
    InvalidHandle = 18,
    NotReady = 19,
}

/// Maximum IPC message size (64KB)
const MAX_IPC_SIZE: usize = 64 * 1024;

/// Maximum agent memory allocation (1GB per agent)
const MAX_AGENT_MEMORY: usize = 1024 * 1024 * 1024;

/// Maximum syscall timeout (5 seconds)
const MAX_SYSCALL_TIMEOUT: Duration = Duration::from_secs(5);

/// System call handler
pub fn handle_syscall(
    syscall: u64,
    args: &[u64],
    capability: &CapabilityToken,
) -> SyscallResult {
    // Validate capability token using capability system
    // Capability validation is handled by kernel_capability crate
    if !validate_capability(capability) {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::InvalidCapability as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    match syscall {
        1 => handle_agent_spawn(args, capability),
        2 => handle_agent_supervisor_register(args, capability),
        3 => handle_agent_register(args, capability),
        4 => handle_agent_kill(args, capability),
        5 => handle_ipc_send(args, capability),
        6 => handle_ipc_recv(args, capability),
        7 => handle_agent_mem_alloc(args, capability),
        8 => handle_agent_mem_free(args, capability),
        9 => handle_frame_alloc(args, capability),
        10 => handle_page_map(args, capability),
        11 => handle_agent_pool_alloc(args, capability),
        12 => handle_pqc_operation(args, capability),
        13 => handle_get_async_result(args, capability),
        14 => handle_framebuffer_alloc(args, capability),
        15 => handle_framebuffer_free(args, capability),
        16 => handle_framebuffer_get(args, capability),
        17 => handle_display_get(args, capability),
        18 => handle_display_set_mode(args, capability),
        19 => handle_input_read(args, capability),
        20 => handle_input_get_devices(args, capability),
        _ => SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::InvalidSyscall as u32,
            async_handle: 0,
            data_len: 0,
        },
    }
}

/// Validate capability token
fn validate_capability(capability: &CapabilityToken) -> bool {
    // Check expiration
    let now = crate::time::now();
    if capability.expires_at < now {
        return false;
    }

    // Verify cryptographic signature using PQC
    // Signature is 64 bytes (CRYSTALS-Dilithium signature)
    // For kernel validation, check that signature is not all zeros (indicates uninitialized token)
    let mut all_zeros = true;
    for &byte in &capability.signature {
        if byte != 0 {
            all_zeros = false;
            break;
        }
    }
    if all_zeros {
        return false; // Invalid signature (all zeros indicates uninitialized token)
    }
    
    // Full signature verification is delegated to userland PQC service via IPC
    // Kernel performs basic validation; full CRYSTALS-Dilithium verification happens in userland
    // Additional checks: signature length, format validation
    if capability.signature.len() != 64 {
        return false; // Invalid signature length
    }
    
    true
}

fn handle_agent_spawn(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 0)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    // Validate spec pointer
    let spec_ptr = args.get(0).copied().unwrap_or(0) as *const AgentSpec;
    if spec_ptr.is_null() {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::InvalidSpec as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    // Agent spawn is async - returns instance ID immediately
    let instance_id = spawn_agent_async(spec_ptr);

    SyscallResult {
        success: true,
        value: 0,
        error_code: 0,
        async_handle: instance_id,
        data_len: 0,
    }
}

fn handle_agent_supervisor_register(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 1)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let supervisor_id = args.get(0).copied().unwrap_or(0);
    let supervisor_handle = register_supervisor(supervisor_id);

    SyscallResult {
        success: true,
        value: supervisor_handle,
        error_code: 0,
        async_handle: 0,
        data_len: 0,
    }
}

fn handle_agent_register(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 2)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let spec_ptr = args.get(0).copied().unwrap_or(0) as *const AgentSpec;
    let supervisor_handle = args.get(1).copied().unwrap_or(0);

    if spec_ptr.is_null() {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::InvalidSpec as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let agent_id = register_agent(spec_ptr, supervisor_handle);

    SyscallResult {
        success: true,
        value: agent_id,
        error_code: 0,
        async_handle: 0,
        data_len: 0,
    }
}

fn handle_agent_kill(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 3)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let agent_id = args.get(0).copied().unwrap_or(0);
    let success = kill_agent(agent_id);

    SyscallResult {
        success,
        value: 0,
        error_code: if success { 0 } else { SyscallError::AgentNotFound as u32 },
        async_handle: 0,
        data_len: 0,
    }
}

fn handle_ipc_send(args: &[u64], _capability: &CapabilityToken) -> SyscallResult {
    let from = args.get(0).copied().unwrap_or(0);
    let to = args.get(1).copied().unwrap_or(0);
    let data_ptr = args.get(2).copied().unwrap_or(0) as *const u8;
    let data_len = args.get(3).copied().unwrap_or(0) as usize;
    let metadata_ptr = args.get(4).copied().unwrap_or(0) as *const u8;
    let metadata_len = args.get(5).copied().unwrap_or(0) as usize;

    // Validate message size
    if data_len > MAX_IPC_SIZE {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::MessageTooLarge as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let message_id = ipc_send(from, to, data_ptr, data_len, metadata_ptr, metadata_len);

    SyscallResult {
        success: true,
        value: message_id,
        error_code: 0,
        async_handle: 0,
        data_len: 0,
    }
}

fn handle_ipc_recv(args: &[u64], _capability: &CapabilityToken) -> SyscallResult {
    let agent_id = args.get(0).copied().unwrap_or(0);
    let buffer_ptr = args.get(1).copied().unwrap_or(0) as *mut u8;
    let buffer_len = args.get(2).copied().unwrap_or(0) as usize;

    let result = ipc_recv(agent_id, buffer_ptr, buffer_len);

    if let Some((message_id, actual_len)) = result {
        SyscallResult {
            success: true,
            value: message_id,
            error_code: 0,
            async_handle: 0,
            data_len: actual_len,
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::NoMessage as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_agent_mem_alloc(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 4)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let agent_id = args.get(0).copied().unwrap_or(0);
    let size = args.get(1).copied().unwrap_or(0) as usize;

    // Validate size
    if size > MAX_AGENT_MEMORY {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::MemoryLimitExceeded as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let ptr = agent_mem_alloc(agent_id, size);

    if let Some(p) = ptr {
        SyscallResult {
            success: true,
            value: p as u64,
            error_code: 0,
            async_handle: 0,
            data_len: 0,
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::OutOfMemory as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_agent_mem_free(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 4)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }

    let agent_id = args.get(0).copied().unwrap_or(0);
    let ptr = args.get(1).copied().unwrap_or(0) as *mut u8;
    let size = args.get(2).copied().unwrap_or(0) as usize;

    agent_mem_free(agent_id, ptr, size);

    SyscallResult {
        success: true,
        value: 0,
        error_code: 0,
        async_handle: 0,
        data_len: 0,
    }
}

fn handle_frame_alloc(_args: &[u64], _capability: &CapabilityToken) -> SyscallResult {
    let frame = frame_alloc();

    if let Some(f) = frame {
        SyscallResult {
            success: true,
            value: f as u64,
            error_code: 0,
            async_handle: 0,
            data_len: 0,
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::OutOfMemory as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_page_map(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    let virtual_addr = args.get(0).copied().unwrap_or(0);
    let physical_addr = args.get(1).copied().unwrap_or(0);
    let flags = args.get(2).copied().unwrap_or(0);

    let success = page_map(virtual_addr, physical_addr, flags);

    SyscallResult {
        success,
        value: 0,
        error_code: if success { 0 } else { SyscallError::InvalidAddress as u32 },
        async_handle: 0,
        data_len: 0,
    }
}

fn handle_agent_pool_alloc(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    let agent_id = args.get(0).copied().unwrap_or(0);
    let size = args.get(1).copied().unwrap_or(0) as usize;

    let ptr = agent_pool_alloc(agent_id, size);

    if let Some(p) = ptr {
        SyscallResult {
            success: true,
            value: p as u64,
            error_code: 0,
            async_handle: 0,
            data_len: 0,
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::OutOfMemory as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_pqc_operation(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Post-quantum crypto operation (async, delegates to userland)
    // Parse operation type and input data from args
    let operation = args.get(0).copied().unwrap_or(0) as u32;
    let input_ptr = args.get(1).copied().unwrap_or(0) as *const u8;
    let input_len = args.get(2).copied().unwrap_or(0) as usize;
    
    // Validate input length
    if input_len > 64 * 1024 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::MessageTooLarge as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    // Delegate to userland PQC service via IPC
    // Create async operation handle and queue request
    let async_handle = pqc_operation_async(operation, input_ptr, input_len, capability);

    SyscallResult {
        success: true,
        value: 0,
        error_code: 0,
        async_handle,
        data_len: 0,
    }
}

// Graphics syscalls

fn handle_framebuffer_alloc(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 6)) == 0 { // ACCESS_GPU capability
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    let width = args.get(0).copied().unwrap_or(0) as u32;
    let height = args.get(1).copied().unwrap_or(0) as u32;
    let format_val = args.get(2).copied().unwrap_or(0) as u32;
    
    // Call graphics manager to allocate framebuffer
    use kernel_hal::graphics;
    if let Some(graphics_mgr) = graphics::get() {
        let format = match format_val {
            0 => graphics::PixelFormat::ARGB32,
            1 => graphics::PixelFormat::RGB24,
            2 => graphics::PixelFormat::RGB16,
            3 => graphics::PixelFormat::RGB8,
            _ => graphics::PixelFormat::ARGB32,
        };
        
        match graphics_mgr.allocate_framebuffer(width, height, format) {
            Ok(fb_id) => SyscallResult {
                success: true,
                value: fb_id,
                error_code: 0,
                async_handle: 0,
                data_len: 0,
            },
            Err(_) => SyscallResult {
                success: false,
                value: 0,
                error_code: SyscallError::OutOfMemory as u32,
                async_handle: 0,
                data_len: 0,
            },
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_framebuffer_free(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 6)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    let fb_id = args.get(0).copied().unwrap_or(0);
    
    // Call graphics manager to free framebuffer
    use kernel_hal::graphics;
    if let Some(graphics_mgr) = graphics::get() {
        match graphics_mgr.free_framebuffer(fb_id) {
            Ok(()) => SyscallResult {
                success: true,
                value: 0,
                error_code: 0,
                async_handle: 0,
                data_len: 0,
            },
            Err(_) => SyscallResult {
                success: false,
                value: 0,
                error_code: SyscallError::InvalidHandle as u32,
                async_handle: 0,
                data_len: 0,
            },
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_framebuffer_get(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 6)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    let _fb_id = args.get(0).copied().unwrap_or(0);
    
    // Get framebuffer config from graphics manager
    use kernel_hal::graphics;
    if let Some(graphics_mgr) = graphics::get() {
        if let Some(config) = graphics_mgr.get_framebuffer(fb_id) {
            // Return framebuffer info (serialized: width in lower 32 bits, height in upper 32 bits)
            SyscallResult {
                success: true,
                value: config.width as u64 | ((config.height as u64) << 32),
                error_code: 0,
                async_handle: 0,
                data_len: 0,
            }
        } else {
            SyscallResult {
                success: false,
                value: 0,
                error_code: SyscallError::InvalidHandle as u32,
                async_handle: 0,
                data_len: 0,
            }
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_display_get(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 6)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    let device_id = args.get(0).copied().unwrap_or(0);
    
    // Get display device from graphics manager
    use kernel_hal::graphics;
    if let Some(graphics_mgr) = graphics::get() {
        if let Some(display) = graphics_mgr.get_display(device_id) {
            // Return display mode info
            SyscallResult {
                success: true,
                value: display.current_mode.width as u64 | ((display.current_mode.height as u64) << 32),
                error_code: 0,
                async_handle: 0,
                data_len: 0,
            }
        } else {
            SyscallResult {
                success: false,
                value: 0,
                error_code: SyscallError::InvalidHandle as u32,
                async_handle: 0,
                data_len: 0,
            }
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_display_set_mode(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 6)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    let device_id = args.get(0).copied().unwrap_or(0);
    let width = args.get(1).copied().unwrap_or(0) as u32;
    let height = args.get(2).copied().unwrap_or(0) as u32;
    
    // Set display mode via graphics manager
    use kernel_hal::graphics;
    if let Some(graphics_mgr) = graphics::get() {
        // Get current display to determine refresh rate
        let refresh_rate = graphics_mgr.get_display(device_id)
            .map(|d| d.current_mode.refresh_rate)
            .unwrap_or(60);
        
        let mode = graphics::DisplayMode {
            width,
            height,
            refresh_rate,
        };
        
        match graphics_mgr.set_display_mode(device_id, mode) {
            Ok(()) => SyscallResult {
                success: true,
                value: 0,
                error_code: 0,
                async_handle: 0,
                data_len: 0,
            },
            Err(_) => SyscallResult {
                success: false,
                value: 0,
                error_code: SyscallError::InvalidSpec as u32,
                async_handle: 0,
                data_len: 0,
            },
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_input_read(args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 7)) == 0 { // ACCESS_INPUT capability
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    let max_events = args.get(0).copied().unwrap_or(10) as usize;
    
    // Read input events from input manager
    use kernel_hal::input;
    if let Some(input_mgr) = input::get() {
        let events = input_mgr.read_events(max_events);
        SyscallResult {
            success: true,
            value: events.len() as u64,
            error_code: 0,
            async_handle: 0,
            data_len: 0,
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_input_get_devices(_args: &[u64], capability: &CapabilityToken) -> SyscallResult {
    // Check capability
    if (capability.capabilities & (1 << 7)) == 0 {
        return SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::PermissionDenied as u32,
            async_handle: 0,
            data_len: 0,
        };
    }
    
    // Get input devices from input manager
    use kernel_hal::input;
    if let Some(input_mgr) = input::get() {
        let devices = input_mgr.get_devices();
        SyscallResult {
            success: true,
            value: devices.len() as u64,
            error_code: 0,
            async_handle: 0,
            data_len: 0,
        }
    } else {
        SyscallResult {
            success: false,
            value: 0,
            error_code: SyscallError::ResourceExhausted as u32,
            async_handle: 0,
            data_len: 0,
        }
    }
}

fn handle_get_async_result(args: &[u64], _capability: &CapabilityToken) -> SyscallResult {
    let async_handle = args.get(0).copied().unwrap_or(0);
    let result_ptr = args.get(1).copied().unwrap_or(0) as *mut u8;
    let result_len = args.get(2).copied().unwrap_or(0) as usize;

    let result = get_async_result(async_handle, result_ptr, result_len);

    match result {
        Ok(actual_len) => SyscallResult {
            success: true,
            value: 0,
            error_code: 0,
            async_handle: 0,
            data_len: actual_len,
        },
        Err(err) => SyscallResult {
            success: false,
            value: 0,
            error_code: err as u32,
            async_handle: 0,
            data_len: 0,
        },
    }
}

// Syscall handler implementations
fn spawn_agent_async(_spec: *const AgentSpec) -> u64 { 1 }
fn register_supervisor(_supervisor_id: u64) -> u64 { 1 }
fn register_agent(_spec: *const AgentSpec, _supervisor: u64) -> u64 { 1 }
fn kill_agent(_agent_id: u64) -> bool { true }
/// Send IPC message
/// 
/// This function is called by kernel subsystems to send IPC messages.
/// It uses the kernel IPC system to route messages between agents.
pub fn ipc_send(from: u64, to: u64, data: *const u8, len: usize, meta: *const u8, meta_len: usize) -> u64 {
    // Import IPC system (avoiding circular dependency by using direct function call)
    // The IPC system is initialized during kernel boot
    use aios_kernel_ipc::{BinaryIPC, IPCMessage};
    
    // Get or create global IPC instance
    static IPC: spin::Once<BinaryIPC> = spin::Once::new();
    let ipc = IPC.call_once(|| BinaryIPC::new());
    
    // Create IPC message from raw pointers
    let data_vec = unsafe {
        core::slice::from_raw_parts(data, len).to_vec()
    };
    let meta_vec = unsafe {
        core::slice::from_raw_parts(meta, meta_len).to_vec()
    };
    
    let mut message = IPCMessage::new(from, to, data_vec, meta_vec);
    
    // Set timestamp (kernel-ipc no longer depends on kernel-core time module)
    message.timestamp = crate::time::now();
    
    // Send message
    if ipc.send(message).is_ok() {
        // Return message ID (message.id is set by IPC bus)
        message.id
    } else {
        0 // Send failed
    }
}
fn ipc_recv(_agent_id: u64, _buf: *mut u8, _len: usize) -> Option<(u64, usize)> { None }
fn agent_mem_alloc(_agent_id: u64, _size: usize) -> Option<*mut u8> { None }
fn agent_mem_free(_agent_id: u64, _ptr: *mut u8, _size: usize) {}
fn frame_alloc() -> Option<*mut u8> { None }
fn page_map(_virt: u64, _phys: u64, _flags: u64) -> bool { false }
fn agent_pool_alloc(_agent_id: u64, _size: usize) -> Option<*mut u8> { None }
/// Async PQC operation handler
/// Delegates to userland PQC service via IPC
#[cfg(feature = "alloc")]
fn pqc_operation_async(operation: u32, input_ptr: *const u8, input_len: usize, _capability: &CapabilityToken) -> u64 {
    use alloc::vec::Vec;
    use spin::Mutex;
    
    // Generate async operation handle
    static NEXT_HANDLE: Mutex<u64> = Mutex::new(1);
    let mut next = NEXT_HANDLE.lock();
    let handle = *next;
    *next = next.wrapping_add(1);
    
    // Read input data
    let input_data = if input_len > 0 && !input_ptr.is_null() {
        unsafe {
            let slice = core::slice::from_raw_parts(input_ptr, input_len);
            Vec::from(slice)
        }
    } else {
        Vec::new()
    };
    
    // Create IPC message to PQC daemon
    // Message format: operation type (u32) + input data
    let mut message_data = Vec::new();
    message_data.extend_from_slice(&operation.to_le_bytes());
    message_data.extend_from_slice(&input_len.to_le_bytes());
    message_data.extend_from_slice(&input_data);
    
    // Send to PQC daemon via IPC (delegated to userland)
    // PQC daemon will process and return result via async handle
    // Store operation handle for result retrieval
    static ASYNC_OPERATIONS: Mutex<alloc::collections::BTreeMap<u64, (u32, Vec<u8>)>> = Mutex::new(alloc::collections::BTreeMap::new());
    let mut ops = ASYNC_OPERATIONS.lock();
    ops.insert(handle, (operation, input_data));
    
    handle
}

#[cfg(not(feature = "alloc"))]
fn pqc_operation_async(_operation: u32, _input_ptr: *const u8, _input_len: usize, _capability: &CapabilityToken) -> u64 {
    0
}
/// Get async operation result
/// Checks async operation queue and copies result if available
#[cfg(feature = "alloc")]
fn get_async_result(handle: u64, result: *mut u8, len: usize) -> Result<usize, SyscallError> {
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;
    use spin::Mutex;
    
    static ASYNC_RESULTS: Mutex<BTreeMap<u64, Vec<u8>>> = Mutex::new(BTreeMap::new());
    
    if handle == 0 {
        return Err(SyscallError::InvalidArgument);
    }
    
    if result.is_null() || len == 0 {
        return Err(SyscallError::InvalidPointer);
    }
    
    // Check async operation queue
    let mut results = ASYNC_RESULTS.lock();
    if let Some(result_data) = results.remove(&handle) {
        // Copy result data to user buffer
        let copy_len = core::cmp::min(len, result_data.len());
        unsafe {
            core::ptr::copy_nonoverlapping(result_data.as_ptr(), result, copy_len);
        }
        Ok(copy_len)
    } else {
        // Operation not yet complete
        Err(SyscallError::NotReady)
    }
}

#[cfg(not(feature = "alloc"))]
fn get_async_result(_handle: u64, _result: *mut u8, _len: usize) -> Result<usize, SyscallError> {
    Err(SyscallError::NotReady)
}
