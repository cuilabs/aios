//! Driver bus implementation

use super::driver::Driver;
use super::device::Device;
use super::hotplug::HotplugEvent;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// Driver bus
/// 
/// Manages drivers and devices
pub struct DriverBus {
    drivers: Mutex<BTreeMap<u64, Driver>>,
    devices: Mutex<BTreeMap<u64, Device>>,
    bindings: Mutex<BTreeMap<u64, u64>>, // device_id -> driver_id
    next_driver_id: Mutex<u64>,
    next_device_id: Mutex<u64>,
}

impl DriverBus {
    pub fn new() -> Self {
        Self {
            drivers: Mutex::new(BTreeMap::new()),
            devices: Mutex::new(BTreeMap::new()),
            bindings: Mutex::new(BTreeMap::new()),
            next_driver_id: Mutex::new(1),
            next_device_id: Mutex::new(1),
        }
    }

    /// Register driver
    pub fn register_driver(&mut self, mut driver: Driver) -> Result<u64, super::DriverError> {
        // Validate driver
        if driver.name.is_empty() {
            return Err(super::DriverError::InvalidDriver);
        }

        // Assign driver ID
        let mut next_id = self.next_driver_id.lock();
        let driver_id = *next_id;
        *next_id = next_id.wrapping_add(1);
        driver.driver_id = driver_id;

        // Register driver
        let mut drivers = self.drivers.lock();
        drivers.insert(driver_id, driver);

        // Try to bind to devices
        self.try_bind_driver(driver_id);

        Ok(driver_id)
    }

    /// Enumerate devices
    pub fn enumerate_devices(&self) -> Vec<Device> {
        let devices = self.devices.lock();
        devices.values().cloned().collect()
    }

    /// Handle hotplug event
    pub fn handle_hotplug(&mut self, event: HotplugEvent) {
        match event {
            HotplugEvent::DeviceAdded(device) => {
                let mut devices = self.devices.lock();
                let mut next_id = self.next_device_id.lock();
                let device_id = *next_id;
                *next_id = next_id.wrapping_add(1);
                
                // TODO: Set device ID
                devices.insert(device_id, device);

                // Try to bind to driver
                self.try_bind_device(device_id);
            }
            HotplugEvent::DeviceRemoved(device_id) => {
                let mut devices = self.devices.lock();
                devices.remove(&device_id);

                let mut bindings = self.bindings.lock();
                bindings.remove(&device_id);
            }
        }
    }

    /// Try to bind driver to devices
    fn try_bind_driver(&self, driver_id: u64) {
        let drivers = self.drivers.lock();
        let driver = drivers.get(&driver_id);
        if driver.is_none() {
            return;
        }
        let driver = driver.unwrap();

        let devices = self.devices.lock();
        for (device_id, device) in devices.iter() {
            if self.can_bind(driver, device) {
                let mut bindings = self.bindings.lock();
                bindings.insert(*device_id, driver_id);
            }
        }
    }

    /// Try to bind device to driver
    fn try_bind_device(&self, device_id: u64) {
        let devices = self.devices.lock();
        let device = devices.get(&device_id);
        if device.is_none() {
            return;
        }
        let device = device.unwrap();

        let drivers = self.drivers.lock();
        for (driver_id, driver) in drivers.iter() {
            if self.can_bind(driver, device) {
                let mut bindings = self.bindings.lock();
                bindings.insert(device_id, *driver_id);
                break;
            }
        }
    }

    /// Check if driver can bind to device
    fn can_bind(&self, driver: &Driver, device: &Device) -> bool {
        // Check if already bound
        let bindings = self.bindings.lock();
        if bindings.values().any(|&did| did == driver.driver_id) {
            return false;
        }

        // Check device class matches driver
        driver.device_class == device.device_class
    }
}

