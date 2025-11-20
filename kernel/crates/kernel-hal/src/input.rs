//! Input Device Hardware Abstraction
//! 
//! Provides low-level input device access (keyboard, mouse, touch).
//! High-level input handling is in userland (display server service).

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

// Use PCIe and ACPI modules from same crate

#[cfg(feature = "alloc")]
extern crate alloc;

/// Input device type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Touch,
    Gamepad,
    Other,
}

/// Input device
pub struct InputDevice {
    pub device_id: u64,
    pub device_type: InputDeviceType,
    pub enabled: bool,
    pub name: alloc::string::String,
}

/// Input event
#[derive(Clone, Debug)]
pub struct InputEvent {
    pub device_id: u64,
    pub event_type: InputEventType,
    pub timestamp: u64,
    pub data: InputEventData,
}

/// Input event type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEventType {
    KeyPress,
    KeyRelease,
    MouseMove,
    MouseButtonPress,
    MouseButtonRelease,
    MouseWheel,
    TouchDown,
    TouchUp,
    TouchMove,
}

/// Input event data
#[derive(Clone, Debug)]
pub enum InputEventData {
    Key { keycode: u32, scancode: u32 },
    MouseMove { x: i32, y: i32, dx: i32, dy: i32 },
    MouseButton { button: u8, pressed: bool },
    MouseWheel { dx: i32, dy: i32 },
    Touch { x: f32, y: f32, pressure: f32, id: u32 },
}

/// Input manager
/// 
/// Manages input devices and events at kernel level.
/// High-level input handling is in userland.
pub struct InputManager {
    devices: Mutex<BTreeMap<u64, InputDevice>>,
    event_queue: Mutex<Vec<InputEvent>>,
    next_device_id: Mutex<u64>,
    max_queue_size: usize,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(BTreeMap::new()),
            event_queue: Mutex::new(Vec::new()),
            next_device_id: Mutex::new(1),
            max_queue_size: 1000,
        }
    }
    
    /// Initialize input manager
    pub fn init(&self) {
        // Discover input devices
        self.discover_devices();
    }
    
    /// Discover input devices
    fn discover_devices(&self) {
        use super::pcie;
        
        // Enumerate USB HID devices (class 0x03: USB, subclass 0x01: HID)
        let usb_hid_devices = pcie::find_devices_by_class(0x03, 0x01);
        
        for device in usb_hid_devices {
            // Determine device type from USB HID usage page
            // Register as generic input device
            let device_name = alloc::format!("USB HID Device {:04x}:{:04x}", device.vendor_id, device.device_id);
            self.register_device(InputDeviceType::Other, device_name);
        }
        
        // Check for PS/2 keyboard/mouse (legacy devices)
        // PS/2 devices are typically at I/O ports 0x60/0x64
        // Register default PS/2 devices
        self.register_device(InputDeviceType::Keyboard, "PS/2 Keyboard".to_string());
        self.register_device(InputDeviceType::Mouse, "PS/2 Mouse".to_string());
        
        // Check ACPI for input devices
        use super::acpi;
        if acpi::is_available() {
            // ACPI may provide additional input device information
            // Register ACPI input devices if found
        }
    }
    
    /// Register input device
    pub fn register_device(&self, device_type: InputDeviceType, name: alloc::string::String) -> u64 {
        let device_id = {
            let mut next = self.next_device_id.lock();
            let id = *next;
            *next += 1;
            id
        };
        
        let device = InputDevice {
            device_id,
            device_type,
            enabled: true,
            name,
        };
        
        let mut devices = self.devices.lock();
        devices.insert(device_id, device);
        
        device_id
    }
    
    /// Get input devices
    pub fn get_devices(&self) -> Vec<InputDevice> {
        let devices = self.devices.lock();
        devices.values().cloned().collect()
    }
    
    /// Get input device
    pub fn get_device(&self, device_id: u64) -> Option<InputDevice> {
        let devices = self.devices.lock();
        devices.get(&device_id).cloned()
    }
    
    /// Queue input event
    pub fn queue_event(&self, event: InputEvent) {
        let mut queue = self.event_queue.lock();
        queue.push(event);
        
        // Trim queue if too large
        if queue.len() > self.max_queue_size {
            queue.remove(0);
        }
    }
    
    /// Read input events
    pub fn read_events(&self, max_events: usize) -> Vec<InputEvent> {
        let mut queue = self.event_queue.lock();
        let count = queue.len().min(max_events);
        queue.drain(0..count).collect()
    }
    
    /// Clear event queue
    pub fn clear_events(&self) {
        let mut queue = self.event_queue.lock();
        queue.clear();
    }
}

/// Global input manager instance
static INPUT_MANAGER: Mutex<Option<InputManager>> = Mutex::new(None);

/// Initialize input manager
pub fn init() {
    let mut manager = INPUT_MANAGER.lock();
    let input_manager = InputManager::new();
    input_manager.init();
    *manager = Some(input_manager);
}

/// Get input manager instance
pub fn get() -> Option<&'static InputManager> {
    let manager = INPUT_MANAGER.lock();
    manager.as_ref()
}

