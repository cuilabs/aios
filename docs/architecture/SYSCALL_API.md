# AIOS Syscall API

**Production-grade syscall interface with Rust signatures, C ABI numbers, errors, and userland wrappers**

## Syscall Numbers (C ABI)

```c
#define AIOS_SYSCALL_AGENT_SPAWN           1
#define AIOS_SYSCALL_AGENT_SUPERVISOR_REG  2
#define AIOS_SYSCALL_AGENT_REGISTER        3
#define AIOS_SYSCALL_AGENT_KILL            4
#define AIOS_SYSCALL_IPC_SEND              5
#define AIOS_SYSCALL_IPC_RECV              6
#define AIOS_SYSCALL_AGENT_MEM_ALLOC       7
#define AIOS_SYSCALL_AGENT_MEM_FREE        8
#define AIOS_SYSCALL_FRAME_ALLOC           9
#define AIOS_SYSCALL_PAGE_MAP             10
#define AIOS_SYSCALL_AGENT_POOL_ALLOC     11
#define AIOS_SYSCALL_PQC_OPERATION        12
#define AIOS_SYSCALL_GET_ASYNC_RESULT     13
```

## Rust Kernel Signatures

### Agent Operations

#### AgentSpawn
```rust
/// Spawn a new agent (async)
/// 
/// # Arguments
/// - `spec_ptr`: Pointer to AgentSpec structure
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with async_handle (instance ID)
/// 
/// # Errors
/// - `InvalidCapability`: Invalid or expired capability token
/// - `PermissionDenied`: Missing spawn capability
/// - `ResourceExhausted`: System resource limits exceeded
/// - `InvalidSpec`: Invalid agent specification
pub fn syscall_agent_spawn(
    spec_ptr: *const AgentSpec,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### AgentSupervisorRegister
```rust
/// Register agent supervisor with kernel
/// 
/// # Arguments
/// - `supervisor_id`: Supervisor identifier
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with supervisor handle
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `PermissionDenied`: Missing supervisor capability
/// - `AlreadyRegistered`: Supervisor already registered
pub fn syscall_agent_supervisor_register(
    supervisor_id: u64,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### AgentRegister
```rust
/// Register an agent with kernel
/// 
/// # Arguments
/// - `agent_spec_ptr`: Pointer to AgentSpec
/// - `supervisor_handle`: Supervisor handle
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with agent_id
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `InvalidSupervisor`: Invalid supervisor handle
/// - `ResourceExhausted`: Agent limit exceeded
pub fn syscall_agent_register(
    agent_spec_ptr: *const AgentSpec,
    supervisor_handle: u64,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### AgentKill
```rust
/// Kill an agent (immediate termination)
/// 
/// # Arguments
/// - `agent_id`: Agent identifier
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with success status
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `PermissionDenied`: Missing kill capability
/// - `AgentNotFound`: Agent does not exist
pub fn syscall_agent_kill(
    agent_id: u64,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

### IPC Operations

#### IPCSend
```rust
/// Send binary IPC message
/// 
/// # Arguments
/// - `from`: Source agent ID
/// - `to`: Destination agent ID
/// - `data_ptr`: Pointer to message data
/// - `data_len`: Message data length (max 64KB)
/// - `metadata_ptr`: Pointer to metadata (optional)
/// - `metadata_len`: Metadata length
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with message_id
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `MessageTooLarge`: Message exceeds 64KB limit
/// - `AgentNotFound`: Destination agent not found
pub fn syscall_ipc_send(
    from: u64,
    to: u64,
    data_ptr: *const u8,
    data_len: usize,
    metadata_ptr: *const u8,
    metadata_len: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### IPCRecv
```rust
/// Receive binary IPC message (non-blocking)
/// 
/// # Arguments
/// - `agent_id`: Agent identifier
/// - `buffer_ptr`: Pointer to receive buffer
/// - `buffer_len`: Buffer length
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with message_id and actual length
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `BufferTooSmall`: Buffer too small for message
/// - `NoMessage`: No message available
pub fn syscall_ipc_recv(
    agent_id: u64,
    buffer_ptr: *mut u8,
    buffer_len: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

### Memory Operations

#### AgentMemAlloc
```rust
/// Allocate agent memory
/// 
/// # Arguments
/// - `agent_id`: Agent identifier
/// - `size`: Allocation size (max 1GB per agent)
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with memory pointer
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `MemoryLimitExceeded`: Exceeds 1GB per agent limit
/// - `OutOfMemory`: System out of memory
pub fn syscall_agent_mem_alloc(
    agent_id: u64,
    size: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### AgentMemFree
```rust
/// Free agent memory
/// 
/// # Arguments
/// - `agent_id`: Agent identifier
/// - `ptr`: Memory pointer to free
/// - `size`: Allocation size
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with success status
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `InvalidPointer`: Invalid memory pointer
pub fn syscall_agent_mem_free(
    agent_id: u64,
    ptr: *mut u8,
    size: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### FrameAlloc
```rust
/// Allocate physical frame (low-level primitive)
/// 
/// # Arguments
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with frame address
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `OutOfMemory`: No frames available
pub fn syscall_frame_alloc(
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### PageMap
```rust
/// Map virtual page to physical page (low-level primitive)
/// 
/// # Arguments
/// - `virtual_addr`: Virtual address
/// - `physical_addr`: Physical address
/// - `flags`: Page flags
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with success status
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `InvalidAddress`: Invalid address
pub fn syscall_page_map(
    virtual_addr: u64,
    physical_addr: u64,
    flags: u64,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

#### AgentPoolAlloc
```rust
/// Allocate from agent memory pool (low-level primitive)
/// 
/// # Arguments
/// - `agent_id`: Agent identifier
/// - `size`: Allocation size
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with memory pointer
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `OutOfMemory`: Pool exhausted
pub fn syscall_agent_pool_alloc(
    agent_id: u64,
    size: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

### Crypto Operations

#### PQCOperation
```rust
/// Post-quantum crypto operation (async)
/// 
/// # Arguments
/// - `operation`: Operation type (keygen, sign, verify, etc.)
/// - `input_ptr`: Pointer to input data
/// - `input_len`: Input data length
/// - `output_ptr`: Pointer to output buffer
/// - `output_len`: Output buffer length
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with async_handle
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `InvalidOperation`: Invalid operation type
pub fn syscall_pqc_operation(
    operation: u32,
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

### Async Operations

#### GetAsyncResult
```rust
/// Get result of async operation
/// 
/// # Arguments
/// - `async_handle`: Async operation handle
/// - `result_ptr`: Pointer to result buffer
/// - `result_len`: Result buffer length
/// - `capability_token_ptr`: Pointer to CapabilityToken
/// 
/// # Returns
/// - `SyscallResult` with completion status
/// 
/// # Errors
/// - `InvalidCapability`: Invalid capability token
/// - `InvalidHandle`: Invalid async handle
/// - `NotReady`: Operation not yet complete
pub fn syscall_get_async_result(
    async_handle: u64,
    result_ptr: *mut u8,
    result_len: usize,
    capability_token_ptr: *const CapabilityToken,
) -> SyscallResult;
```

## Data Structures

### CapabilityToken
```rust
#[repr(C)]
pub struct CapabilityToken {
    pub token_id: u64,
    pub agent_id: u64,
    pub capabilities: u64,        // Bitmask
    pub expires_at: u64,           // Unix timestamp
    pub signature: [u8; 64],       // Cryptographic signature
}
```

### AgentSpec
```rust
#[repr(C)]
pub struct AgentSpec {
    pub agent_type: u32,
    pub memory_limit: u64,         // Max memory in bytes
    pub cpu_limit: u8,             // Max CPU percentage
    pub capabilities: u64,         // Required capabilities
    pub image_hash: [u8; 32],      // Agent image hash
    pub manifest_ptr: *const u8,   // Pointer to manifest
    pub manifest_len: usize,       // Manifest length
}
```

### SyscallResult
```rust
#[repr(C)]
pub struct SyscallResult {
    pub success: bool,
    pub value: u64,                // Return value
    pub error_code: u32,           // Error code (0 = success)
    pub async_handle: u64,         // Async operation handle
    pub data_len: usize,           // Additional data length
}
```

## Error Codes

```rust
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
```

## Rust Userland Wrapper

```rust
// sdk/rust/src/syscalls.rs

use core::ffi::c_void;

pub struct Syscall;

impl Syscall {
    /// Spawn agent
    pub fn agent_spawn(
        spec: &AgentSpec,
        capability: &CapabilityToken,
    ) -> Result<u64, SyscallError> {
        let result = unsafe {
            syscall_agent_spawn(
                spec as *const AgentSpec,
                capability as *const CapabilityToken,
            )
        };
        
        if result.success {
            Ok(result.async_handle)
        } else {
            Err(SyscallError::from(result.error_code))
        }
    }
    
    /// Send IPC message
    pub fn ipc_send(
        from: u64,
        to: u64,
        data: &[u8],
        metadata: Option<&[u8]>,
        capability: &CapabilityToken,
    ) -> Result<u64, SyscallError> {
        let (metadata_ptr, metadata_len) = if let Some(m) = metadata {
            (m.as_ptr(), m.len())
        } else {
            (core::ptr::null(), 0)
        };
        
        let result = unsafe {
            syscall_ipc_send(
                from,
                to,
                data.as_ptr(),
                data.len(),
                metadata_ptr,
                metadata_len,
                capability as *const CapabilityToken,
            )
        };
        
        if result.success {
            Ok(result.value)
        } else {
            Err(SyscallError::from(result.error_code))
        }
    }
    
    /// Receive IPC message
    pub fn ipc_recv(
        agent_id: u64,
        buffer: &mut [u8],
        capability: &CapabilityToken,
    ) -> Result<(u64, usize), SyscallError> {
        let result = unsafe {
            syscall_ipc_recv(
                agent_id,
                buffer.as_mut_ptr(),
                buffer.len(),
                capability as *const CapabilityToken,
            )
        };
        
        if result.success {
            Ok((result.value, result.data_len))
        } else {
            Err(SyscallError::from(result.error_code))
        }
    }
    
    /// Allocate agent memory
    pub fn agent_mem_alloc(
        agent_id: u64,
        size: usize,
        capability: &CapabilityToken,
    ) -> Result<*mut u8, SyscallError> {
        let result = unsafe {
            syscall_agent_mem_alloc(
                agent_id,
                size,
                capability as *const CapabilityToken,
            )
        };
        
        if result.success {
            Ok(result.value as *mut u8)
        } else {
            Err(SyscallError::from(result.error_code))
        }
    }
    
    /// Get async result
    pub fn get_async_result(
        async_handle: u64,
        result: &mut [u8],
        capability: &CapabilityToken,
    ) -> Result<usize, SyscallError> {
        let result = unsafe {
            syscall_get_async_result(
                async_handle,
                result.as_mut_ptr(),
                result.len(),
                capability as *const CapabilityToken,
            )
        };
        
        if result.success {
            Ok(result.data_len)
        } else {
            Err(SyscallError::from(result.error_code))
        }
    }
}

// External syscall functions (implemented in kernel)
extern "C" {
    fn syscall_agent_spawn(
        spec: *const AgentSpec,
        capability: *const CapabilityToken,
    ) -> SyscallResult;
    
    fn syscall_ipc_send(
        from: u64,
        to: u64,
        data: *const u8,
        data_len: usize,
        metadata: *const u8,
        metadata_len: usize,
        capability: *const CapabilityToken,
    ) -> SyscallResult;
    
    fn syscall_ipc_recv(
        agent_id: u64,
        buffer: *mut u8,
        buffer_len: usize,
        capability: *const CapabilityToken,
    ) -> SyscallResult;
    
    fn syscall_agent_mem_alloc(
        agent_id: u64,
        size: usize,
        capability: *const CapabilityToken,
    ) -> SyscallResult;
    
    fn syscall_get_async_result(
        async_handle: u64,
        result: *mut u8,
        result_len: usize,
        capability: *const CapabilityToken,
    ) -> SyscallResult;
}
```

## TypeScript Userland Wrapper

```typescript
// sdk/typescript/src/syscalls.ts

import { ffi } from '@napi-rs/ffi';

export interface CapabilityToken {
  tokenId: bigint;
  agentId: bigint;
  capabilities: bigint;
  expiresAt: bigint;
  signature: Uint8Array;
}

export interface AgentSpec {
  agentType: number;
  memoryLimit: bigint;
  cpuLimit: number;
  capabilities: bigint;
  imageHash: Uint8Array;
  manifest: Uint8Array;
}

export interface SyscallResult {
  success: boolean;
  value: bigint;
  errorCode: number;
  asyncHandle: bigint;
  dataLen: number;
}

export class Syscalls {
  private static lib = ffi.Library('libaios', {
    syscall_agent_spawn: ['SyscallResult', ['AgentSpec', 'CapabilityToken']],
    syscall_ipc_send: ['SyscallResult', ['bigint', 'bigint', 'Buffer', 'number', 'Buffer', 'number', 'CapabilityToken']],
    syscall_ipc_recv: ['SyscallResult', ['bigint', 'Buffer', 'number', 'CapabilityToken']],
    syscall_agent_mem_alloc: ['SyscallResult', ['bigint', 'number', 'CapabilityToken']],
    syscall_get_async_result: ['SyscallResult', ['bigint', 'Buffer', 'number', 'CapabilityToken']],
  });

  static agentSpawn(
    spec: AgentSpec,
    capability: CapabilityToken,
  ): Promise<bigint> {
    const result = this.lib.syscall_agent_spawn(spec, capability);
    
    if (result.success) {
      return Promise.resolve(result.asyncHandle);
    }
    
    throw new Error(`Syscall failed: ${result.errorCode}`);
  }

  static ipcSend(
    from: bigint,
    to: bigint,
    data: Uint8Array,
    metadata: Uint8Array | null,
    capability: CapabilityToken,
  ): Promise<bigint> {
    const metadataBuf = metadata ? Buffer.from(metadata) : Buffer.alloc(0);
    const result = this.lib.syscall_ipc_send(
      from,
      to,
      Buffer.from(data),
      data.length,
      metadataBuf,
      metadataBuf.length,
      capability,
    );
    
    if (result.success) {
      return Promise.resolve(result.value);
    }
    
    throw new Error(`Syscall failed: ${result.errorCode}`);
  }

  static ipcRecv(
    agentId: bigint,
    buffer: Uint8Array,
    capability: CapabilityToken,
  ): Promise<{ messageId: bigint; length: number }> {
    const buf = Buffer.from(buffer);
    const result = this.lib.syscall_ipc_recv(agentId, buf, buf.length, capability);
    
    if (result.success) {
      return Promise.resolve({
        messageId: result.value,
        length: result.dataLen,
      });
    }
    
    throw new Error(`Syscall failed: ${result.errorCode}`);
  }

  static agentMemAlloc(
    agentId: bigint,
    size: number,
    capability: CapabilityToken,
  ): Promise<Buffer> {
    const result = this.lib.syscall_agent_mem_alloc(agentId, size, capability);
    
    if (result.success) {
      return Promise.resolve(Buffer.allocUnsafe(size));
    }
    
    throw new Error(`Syscall failed: ${result.errorCode}`);
  }

  static async getAsyncResult(
    asyncHandle: bigint,
    result: Uint8Array,
    capability: CapabilityToken,
  ): Promise<number> {
    const buf = Buffer.from(result);
    const result = this.lib.syscall_get_async_result(asyncHandle, buf, buf.length, capability);
    
    if (result.success) {
      return Promise.resolve(result.dataLen);
    }
    
    if (result.errorCode === 19) { // NotReady
      // Poll again
      await new Promise(resolve => setTimeout(resolve, 10));
      return this.getAsyncResult(asyncHandle, result, capability);
    }
    
    throw new Error(`Syscall failed: ${result.errorCode}`);
  }
}
```

---

**Version:** 0.1.0  
**Last Updated:** November 2025

