#![no_std]

//! Device & Driver Bus
//! 
//! Full driver model with hotplug, enumeration, versioning, ACPI/DT tree

extern crate alloc;

pub mod bus;
pub mod driver;
pub mod device;
pub mod hotplug;
pub mod acpi_tree;
pub mod sata;
pub mod nvme;
pub mod ethernet;
pub mod tpm;
pub mod sgx;
pub mod enclave;

use bus::DriverBus;
use driver::Driver;
use device::Device;

/// Driver system
pub struct DriverSystem {
    bus: spin::Mutex<DriverBus>,
}

impl DriverSystem {
    pub fn new() -> Self {
        Self {
            bus: spin::Mutex::new(DriverBus::new()),
        }
    }

    /// Initialize driver system
    pub fn init(&self) {
        // Enumerate devices from ACPI/DT
        // Load drivers
        // Bind drivers to devices
        
        // Initialize TPM driver
        tpm::init();
        
        // Initialize SGX driver
        sgx::init();
        
        // Initialize enclave driver
        enclave::init();
    }

    /// Register driver
    pub fn register_driver(&self, driver: Driver) -> Result<u64, DriverError> {
        let mut bus = self.bus.lock();
        bus.register_driver(driver)
    }

    /// Enumerate devices
    pub fn enumerate_devices(&self) -> alloc::vec::Vec<Device> {
        let bus = self.bus.lock();
        bus.enumerate_devices()
    }

    /// Handle hotplug event
    pub fn handle_hotplug(&self, event: hotplug::HotplugEvent) {
        let mut bus = self.bus.lock();
        bus.handle_hotplug(event);
    }
}

#[derive(Debug)]
pub enum DriverError {
    InvalidDriver,
    DeviceNotFound,
    AlreadyBound,
    VersionMismatch,
}

