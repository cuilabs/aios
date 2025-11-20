//! Device structure

/// Device
#[derive(Clone)]
pub struct Device {
    pub device_id: u64,
    pub device_class: u32,
    pub class: u8,
    pub subclass: u8,
    pub vendor_id: u16,
    pub device_id_hw: u16,
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
}

