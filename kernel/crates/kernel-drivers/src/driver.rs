//! Driver structure

use alloc::string::String;

/// Driver
pub struct Driver {
    pub driver_id: u64,
    pub name: String,
    pub version: DriverVersion,
    pub device_class: u32,
    pub init: fn() -> Result<(), DriverError>,
    pub probe: fn(device_id: u64) -> Result<(), DriverError>,
    pub remove: fn(device_id: u64) -> Result<(), DriverError>,
}

/// Driver version
#[derive(Clone)]
pub struct DriverVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug)]
pub enum DriverError {
    InitFailed,
    ProbeFailed,
    RemoveFailed,
}

