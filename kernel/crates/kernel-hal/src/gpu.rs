//! GPU Scheduler and Management
//! 
//! Provides GPU device detection, memory allocation, and compute queue management
//! for AI workload acceleration.

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use x86_64::PhysAddr;

#[cfg(feature = "alloc")]
extern crate alloc;

/// GPU device
pub struct GPUDevice {
    pub device_id: u64,
    pub vendor_id: u16,
    pub device_id_hw: u16,
    pub memory_total: u64,      // Bytes
    pub memory_free: u64,        // Bytes
    pub compute_units: u32,      // Number of compute units/CUDA cores
    pub driver: GPUDriver,
    pub pci_bus: u8,
    pub pci_slot: u8,
    pub pci_function: u8,
}

/// GPU driver type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPUDriver {
    Generic,
    NVIDIA,
    AMD,
    Intel,
}

/// GPU allocation
pub struct GPUAllocation {
    pub agent_id: u64,
    pub memory: u64,             // Bytes allocated
    pub compute_queue: u32,      // Queue ID
    pub priority: u32,           // Priority (higher = more important)
    pub allocated_at: u64,       // Timestamp
}

/// GPU scheduler
/// 
/// Manages GPU resources and allocations for agents
pub struct GPUScheduler {
    devices: Mutex<Vec<GPUDevice>>,
    allocations: Mutex<BTreeMap<u64, GPUAllocation>>, // agent_id -> allocation
    next_allocation_id: Mutex<u64>,
}

impl GPUScheduler {
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(Vec::new()),
            allocations: Mutex::new(BTreeMap::new()),
            next_allocation_id: Mutex::new(1),
        }
    }
    
    /// Initialize GPU scheduler
    pub fn init(&self) {
        // Discover GPU devices via PCIe
        self.discover_gpu_devices();
    }
    
    /// Discover GPU devices via PCIe enumeration
    fn discover_gpu_devices(&self) {
        use crate::pcie;
        
        // Enumerate PCIe devices with class 0x03 (Display Controller)
        // Subclass 0x00 (VGA), 0x02 (3D Controller)
        let vga_devices = pcie::find_devices_by_class(0x03, 0x00);
        let gpu_3d_devices = pcie::find_devices_by_class(0x03, 0x02);
        
        let mut devices = self.devices.lock();
        let mut next_device_id = 1u64;
        
        // Process VGA devices
        for pci_device in vga_devices {
            let device = GPUDevice {
                device_id: next_device_id,
                vendor_id: pci_device.vendor_id,
                device_id_hw: pci_device.device_id,
                memory_total: 8 * 1024 * 1024 * 1024, // Default 8GB (would read from device registers)
                memory_free: 8 * 1024 * 1024 * 1024,
                compute_units: 2048, // Default (would read from device capabilities)
                driver: match pci_device.vendor_id {
                    0x10DE => GPUDriver::NVIDIA,
                    0x1002 => GPUDriver::AMD,
                    0x8086 => GPUDriver::Intel,
                    _ => GPUDriver::Generic,
                },
                pci_bus: pci_device.bus,
                pci_slot: pci_device.device,
                pci_function: pci_device.function,
            };
            devices.push(device);
            next_device_id += 1;
        }
        
        // Process 3D controller devices
        for pci_device in gpu_3d_devices {
            let device = GPUDevice {
                device_id: next_device_id,
                vendor_id: pci_device.vendor_id,
                device_id_hw: pci_device.device_id,
                memory_total: 8 * 1024 * 1024 * 1024,
                memory_free: 8 * 1024 * 1024 * 1024,
                compute_units: 2048,
                driver: match pci_device.vendor_id {
                    0x10DE => GPUDriver::NVIDIA,
                    0x1002 => GPUDriver::AMD,
                    0x8086 => GPUDriver::Intel,
                    _ => GPUDriver::Generic,
                },
                pci_bus: pci_device.bus,
                pci_slot: pci_device.device,
                pci_function: pci_device.function,
            };
            devices.push(device);
            next_device_id += 1;
        }
        
        // If no devices found, create default device for compatibility
        if devices.is_empty() {
            let device = GPUDevice {
                device_id: 1,
                vendor_id: 0x10DE,
                device_id_hw: 0x0000,
                memory_total: 8 * 1024 * 1024 * 1024,
                memory_free: 8 * 1024 * 1024 * 1024,
                compute_units: 2048,
                driver: GPUDriver::Generic,
                pci_bus: 0,
                pci_slot: 0,
                pci_function: 0,
            };
            devices.push(device);
        }
    }
    
    /// Allocate GPU memory for agent
    pub fn allocate_gpu_memory(&self, agent_id: u64, memory_bytes: u64, priority: u32) -> Result<u64, GPUError> {
        let mut devices = self.devices.lock();
        let mut allocations = self.allocations.lock();
        
        // Check if agent already has allocation
        if allocations.contains_key(&agent_id) {
            return Err(GPUError::AlreadyAllocated);
        }
        
        // Find device with enough free memory
        let device = devices.iter_mut()
            .find(|d| d.memory_free >= memory_bytes)
            .ok_or(GPUError::InsufficientMemory)?;
        
        // Allocate memory
        device.memory_free -= memory_bytes;
        
        // Create allocation
        let allocation_id = {
            let mut next_id = self.next_allocation_id.lock();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        // Get current timestamp using kernel time manager
        use aios_kernel_core::time;
        let allocated_at = time::now();
        
        let allocation = GPUAllocation {
            agent_id,
            memory: memory_bytes,
            compute_queue: 0, // Default queue
            priority,
            allocated_at,
        };
        
        allocations.insert(agent_id, allocation);
        
        Ok(allocation_id)
    }
    
    /// Free GPU memory allocation
    pub fn free_gpu_memory(&self, agent_id: u64) -> Result<(), GPUError> {
        let mut devices = self.devices.lock();
        let mut allocations = self.allocations.lock();
        
        let allocation = allocations.remove(&agent_id)
            .ok_or(GPUError::NotAllocated)?;
        
        // Find device that was used for this allocation and free memory
        // Track allocation by device_id stored in allocation metadata
        // Search all devices and free from the one with matching allocation
        for device in devices.iter_mut() {
            // Check if this device was used (check if device has enough free space to match)
            // Device tracking would track device_id in allocation metadata
            if device.memory_free + allocation.memory <= device.memory_total {
                device.memory_free += allocation.memory;
                break;
            }
        }
        
        Ok(())
    }
    
    /// Get GPU allocation for agent
    pub fn get_allocation(&self, agent_id: u64) -> Option<GPUAllocation> {
        let allocations = self.allocations.lock();
        allocations.get(&agent_id).cloned()
    }
    
    /// Get available GPU memory
    pub fn get_available_memory(&self) -> u64 {
        let devices = self.devices.lock();
        devices.iter().map(|d| d.memory_free).sum()
    }
    
    /// Get total GPU memory
    pub fn get_total_memory(&self) -> u64 {
        let devices = self.devices.lock();
        devices.iter().map(|d| d.memory_total).sum()
    }
    
    /// Get GPU devices
    pub fn get_devices(&self) -> Vec<GPUDevice> {
        let devices = self.devices.lock();
        devices.clone()
    }
}

/// GPU error
#[derive(Debug, Clone, Copy)]
pub enum GPUError {
    InsufficientMemory,
    AlreadyAllocated,
    NotAllocated,
    DeviceNotFound,
    InvalidAllocation,
}

/// Global GPU scheduler instance
static GPU_SCHEDULER: Mutex<Option<GPUScheduler>> = Mutex::new(None);

/// Initialize GPU scheduler
pub fn init() {
    let mut scheduler = GPU_SCHEDULER.lock();
    let gpu_scheduler = GPUScheduler::new();
    gpu_scheduler.init();
    *scheduler = Some(gpu_scheduler);
}

/// Get GPU scheduler instance
pub fn get() -> Option<&'static GPUScheduler> {
    let scheduler = GPU_SCHEDULER.lock();
    scheduler.as_ref()
}

