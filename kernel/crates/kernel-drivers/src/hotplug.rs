//! Hotplug support

use super::device::Device;

/// Hotplug event
pub enum HotplugEvent {
    DeviceAdded(Device),
    DeviceRemoved(u64),
}

