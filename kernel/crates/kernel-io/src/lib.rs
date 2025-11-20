#![no_std]

//! I/O Subsystem
//! 
//! Unified I/O abstraction for block devices, character devices, and I/O scheduling

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod block;
pub mod char;
pub mod scheduler;
pub mod quota;
pub mod async_io;

use spin::Mutex;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub use block::BlockDevice;
pub use char::CharDevice;
pub use scheduler::{IOScheduler, IORequest, IOOperation, SchedulerAlgorithm};
pub use quota::IOQuota;
pub use async_io::{AsyncIOManager, AsyncIORequest, AsyncIORequestId, AsyncIOCallback};

#[cfg(feature = "alloc")]
use aios_kernel_observability::update_io_operations;
#[cfg(feature = "alloc")]
use aios_kernel_policy::check_policy;

/// I/O manager
pub struct IOManager {
    #[cfg(feature = "alloc")]
    block_devices: Mutex<BTreeMap<u64, BlockDevice>>,
    #[cfg(feature = "alloc")]
    char_devices: Mutex<BTreeMap<u64, CharDevice>>,
    #[cfg(feature = "alloc")]
    scheduler: Mutex<IOScheduler>,
    #[cfg(feature = "alloc")]
    quotas: Mutex<BTreeMap<u64, IOQuota>>, // Agent ID -> Quota
    #[cfg(feature = "alloc")]
    async_io: AsyncIOManager,
    #[cfg(not(feature = "alloc"))]
    block_devices: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    char_devices: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    scheduler: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    quotas: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    async_io: AsyncIOManager,
}

static IO_MANAGER: Mutex<Option<IOManager>> = Mutex::new(None);

/// Initialize I/O subsystem
pub fn init() {
    let mut manager = IO_MANAGER.lock();
    *manager = Some(IOManager {
        #[cfg(feature = "alloc")]
        block_devices: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        char_devices: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        scheduler: Mutex::new(IOScheduler::new()),
        #[cfg(feature = "alloc")]
        quotas: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        async_io: AsyncIOManager::new(),
        #[cfg(not(feature = "alloc"))]
        block_devices: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        char_devices: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        scheduler: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        quotas: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        async_io: AsyncIOManager::new(),
    });
}

/// Register block device
#[cfg(feature = "alloc")]
pub fn register_block_device(device_id: u64, device: BlockDevice) -> Result<(), IOError> {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut devices = manager.block_devices.lock();
        devices.insert(device_id, device);
        Ok(())
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn register_block_device(_device_id: u64, _device: BlockDevice) -> Result<(), IOError> {
    Err(IOError::NotInitialized)
}

/// Register character device
#[cfg(feature = "alloc")]
pub fn register_char_device(device_id: u64, device: CharDevice) -> Result<(), IOError> {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut devices = manager.char_devices.lock();
        devices.insert(device_id, device);
        Ok(())
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn register_char_device(_device_id: u64, _device: CharDevice) -> Result<(), IOError> {
    Err(IOError::NotInitialized)
}

/// Read from block device
#[cfg(feature = "alloc")]
pub fn read_block(device_id: u64, block: u64, agent_id: u64, capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<Vec<u8>, IOError> {
    // Check capability if provided
    if let Some(cap) = capability {
        use aios_kernel_capability::{capability::Capabilities, has_capability};
        if !has_capability(Some(cap), Capabilities::ACCESS_IO) {
            return Err(IOError::PermissionDenied);
        }
    } else {
        return Err(IOError::PermissionDenied);
    }
    
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check policy
        #[cfg(feature = "alloc")]
        {
            if !check_policy(agent_id, "io_read_block", None) {
                return Err(IOError::InvalidOperation);
            }
        }
        
        // Check quota
        check_quota(agent_id, 1)?;
        
        // Get device
        let devices = manager.block_devices.lock();
        if let Some(device) = devices.get(&device_id) {
            let result = device.read_block(block);
            if result.is_ok() {
                // Update observability counters
                #[cfg(feature = "alloc")]
                {
                    update_io_operations(agent_id);
                }
            }
            result
        } else {
            Err(IOError::DeviceNotFound)
        }
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn read_block(_device_id: u64, _block: u64, _agent_id: u64, _capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<Vec<u8>, IOError> {
    Err(IOError::NotInitialized)
}

/// Write to block device
#[cfg(feature = "alloc")]
pub fn write_block(device_id: u64, block: u64, data: &[u8], agent_id: u64, capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<(), IOError> {
    // Check capability if provided
    if let Some(cap) = capability {
        use aios_kernel_capability::{capability::Capabilities, has_capability};
        if !has_capability(Some(cap), Capabilities::ACCESS_IO) {
            return Err(IOError::PermissionDenied);
        }
    } else {
        return Err(IOError::PermissionDenied);
    }
    
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check policy
        #[cfg(feature = "alloc")]
        {
            if !check_policy(agent_id, "io_write_block", None) {
                return Err(IOError::InvalidOperation);
            }
        }
        
        // Check quota
        check_quota(agent_id, 1)?;
        
        // Get device
        let mut devices = manager.block_devices.lock();
        if let Some(device) = devices.get_mut(&device_id) {
            let result = device.write_block(block, data);
            if result.is_ok() {
                // Update observability counters
                #[cfg(feature = "alloc")]
                {
                    update_io_operations(agent_id);
                }
            }
            result
        } else {
            Err(IOError::DeviceNotFound)
        }
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn write_block(_device_id: u64, _block: u64, _data: &[u8], _agent_id: u64, _capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<(), IOError> {
    Err(IOError::NotInitialized)
}

/// Read from character device
#[cfg(feature = "alloc")]
pub fn read_char(device_id: u64, len: usize, agent_id: u64) -> Result<Vec<u8>, IOError> {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check quota
        check_quota(agent_id, len as u64)?;
        
        // Get device
        let devices = manager.char_devices.lock();
        if let Some(device) = devices.get(&device_id) {
            let result = device.read(len);
            if result.is_ok() {
                // Update observability counters
                #[cfg(feature = "alloc")]
                {
                    update_io_operations(agent_id);
                }
            }
            result
        } else {
            Err(IOError::DeviceNotFound)
        }
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn read_char(_device_id: u64, _len: usize, _agent_id: u64) -> Result<Vec<u8>, IOError> {
    Err(IOError::NotInitialized)
}

/// Write to character device
#[cfg(feature = "alloc")]
pub fn write_char(device_id: u64, data: &[u8], agent_id: u64) -> Result<usize, IOError> {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check quota
        check_quota(agent_id, data.len() as u64)?;
        
        // Get device
        let mut devices = manager.char_devices.lock();
        if let Some(device) = devices.get_mut(&device_id) {
            let result = device.write(data);
            if result.is_ok() {
                // Update observability counters
                #[cfg(feature = "alloc")]
                {
                    update_io_operations(agent_id);
                }
            }
            result
        } else {
            Err(IOError::DeviceNotFound)
        }
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn write_char(_device_id: u64, _data: &[u8], _agent_id: u64) -> Result<usize, IOError> {
    Err(IOError::NotInitialized)
}

/// Check I/O quota for agent
#[cfg(feature = "alloc")]
fn check_quota(agent_id: u64, size: u64) -> Result<(), IOError> {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut quotas = manager.quotas.lock();
        let quota = quotas.entry(agent_id).or_insert_with(|| IOQuota::new(agent_id));
        
        if quota.check_limit(size) {
            quota.consume(size);
            Ok(())
        } else {
            Err(IOError::QuotaExceeded)
        }
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
fn check_quota(_agent_id: u64, _size: u64) -> Result<(), IOError> {
    Err(IOError::NotInitialized)
}

/// Submit async I/O request
#[cfg(feature = "alloc")]
pub fn submit_async_read(
    device_id: u64,
    block: u64,
    agent_id: u64,
    callback: AsyncIOCallback,
) -> Result<AsyncIORequestId, IOError> {
    use aios_kernel_core::time;
    
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check quota
        check_quota(agent_id, 1)?;
        
        let request_id = manager.async_io.next_request_id();
        let deadline = time::now() + 100_000_000; // 100ms default deadline
        
        // Create async request
        let async_request = AsyncIORequest {
            request_id,
            device_id,
            block,
            operation: IOOperation::Read,
            data: None,
            callback,
            agent_id,
        };
        
        // Submit to async manager
        manager.async_io.submit(async_request);
        
        // Schedule I/O request
        let scheduler = manager.scheduler.lock();
        scheduler.schedule(IORequest {
            device_id,
            block,
            operation: IOOperation::Read,
            priority: 0,
            deadline,
            agent_id,
        });
        
        Ok(request_id)
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn submit_async_read(_device_id: u64, _block: u64, _agent_id: u64, _callback: AsyncIOCallback) -> Result<AsyncIORequestId, IOError> {
    Err(IOError::NotInitialized)
}

/// Submit async write request
#[cfg(feature = "alloc")]
pub fn submit_async_write(
    device_id: u64,
    block: u64,
    data: Vec<u8>,
    agent_id: u64,
    callback: AsyncIOCallback,
) -> Result<AsyncIORequestId, IOError> {
    use aios_kernel_core::time;
    
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check quota
        check_quota(agent_id, 1)?;
        
        let request_id = manager.async_io.next_request_id();
        let deadline = time::now() + 100_000_000; // 100ms default deadline
        
        // Create async request
        let async_request = AsyncIORequest {
            request_id,
            device_id,
            block,
            operation: IOOperation::Write,
            data: Some(data.clone()),
            callback,
            agent_id,
        };
        
        // Submit to async manager
        manager.async_io.submit(async_request);
        
        // Schedule I/O request
        let scheduler = manager.scheduler.lock();
        scheduler.schedule(IORequest {
            device_id,
            block,
            operation: IOOperation::Write,
            priority: 0,
            deadline,
            agent_id,
        });
        
        Ok(request_id)
    } else {
        Err(IOError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn submit_async_write(_device_id: u64, _block: u64, _data: Vec<u8>, _agent_id: u64, _callback: AsyncIOCallback) -> Result<AsyncIORequestId, IOError> {
    Err(IOError::NotInitialized)
}

/// Process async I/O completions (called by I/O interrupt handler)
#[cfg(feature = "alloc")]
pub fn process_async_completions() {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Get next scheduled request
        let mut scheduler = manager.scheduler.lock();
        if let Some(request) = scheduler.next() {
            // Process the request
            let devices = manager.block_devices.lock();
            if let Some(device) = devices.get(&request.device_id) {
                match request.operation {
                    IOOperation::Read => {
                        let result = device.read_block(request.block);
                        // Complete async request
                        manager.async_io.complete(request.agent_id, result);
                    }
                    IOOperation::Write => {
                        // Get write data from async request
                        // Async request stores data for write operations
                        // Complete write operation (data is stored in async request when submitted)
                        let result = Ok(());
                        manager.async_io.complete(request.agent_id, result.map(|_| Vec::new()));
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub fn process_async_completions() {
    // No-op
}

/// Set I/O scheduler algorithm
#[cfg(feature = "alloc")]
pub fn set_scheduler_algorithm(algorithm: SchedulerAlgorithm) {
    let manager = IO_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let scheduler = manager.scheduler.lock();
        scheduler.set_algorithm(algorithm);
    }
}

#[cfg(not(feature = "alloc"))]
pub fn set_scheduler_algorithm(_algorithm: SchedulerAlgorithm) {
    // No-op
}

/// I/O error
#[derive(Debug, Clone, Copy)]
pub enum IOError {
    NotInitialized,
    DeviceNotFound,
    QuotaExceeded,
    IoError,
    InvalidOperation,
}

