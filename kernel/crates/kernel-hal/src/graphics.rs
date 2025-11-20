//! Graphics and Display Hardware Abstraction
//! 
//! Provides low-level framebuffer and display hardware access.
//! High-level GUI logic is in userland (display server service).

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use x86_64::PhysAddr;
use x86_64::VirtAddr;

// Use PCIe and ACPI modules from same crate

#[cfg(feature = "alloc")]
extern crate alloc;

/// Framebuffer configuration
#[derive(Clone, Debug)]
pub struct FramebufferConfig {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,        // Bytes per row
    pub bpp: u8,           // Bits per pixel (typically 32)
    pub format: PixelFormat,
    pub physical_addr: PhysAddr,
    pub size: u64,         // Total size in bytes
}

/// Pixel format
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelFormat {
    ARGB32,  // 32-bit ARGB
    RGB24,   // 24-bit RGB
    RGB16,   // 16-bit RGB
    RGB8,    // 8-bit RGB (palette)
}

/// Display device
pub struct DisplayDevice {
    pub device_id: u64,
    pub config: FramebufferConfig,
    pub enabled: bool,
    pub current_mode: DisplayMode,
}

/// Display mode
#[derive(Clone, Copy, Debug)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32, // Hz
}

/// Graphics manager
/// 
/// Manages framebuffers and display devices at kernel level.
/// High-level compositing is handled in userland.
pub struct GraphicsManager {
    framebuffers: Mutex<BTreeMap<u64, FramebufferConfig>>, // framebuffer_id -> config
    displays: Mutex<BTreeMap<u64, DisplayDevice>>,        // device_id -> device
    next_fb_id: Mutex<u64>,
    next_display_id: Mutex<u64>,
}

impl GraphicsManager {
    pub fn new() -> Self {
        Self {
            framebuffers: Mutex::new(BTreeMap::new()),
            displays: Mutex::new(BTreeMap::new()),
            next_fb_id: Mutex::new(1),
            next_display_id: Mutex::new(1),
        }
    }
    
    /// Initialize graphics manager
    pub fn init(&self) {
        // Discover display devices via PCIe/VESA/EFI
        self.discover_displays();
    }
    
    /// Discover display devices
    fn discover_displays(&self) {
        // Enumerate PCIe display controllers (class 0x03: Display Controller)
        use crate::pcie;
        
        let display_devices = pcie::find_devices_by_class(0x03, 0x00); // VGA
        let display_devices_3d = pcie::find_devices_by_class(0x03, 0x02); // 3D Controller
        
        let mut all_displays = Vec::new();
        all_displays.extend(display_devices);
        all_displays.extend(display_devices_3d);
        
        for pcie_device in all_displays {
            let display_id = {
                let mut next = self.next_display_id.lock();
                let id = *next;
                *next += 1;
                id
            };
            
            // Get framebuffer base address from BAR0
            let bar0 = pcie::get_bar(pcie_device.bus, pcie_device.device, pcie_device.function, 0);
            let fb_base = bar0.unwrap_or(0xE0000000);
            
            // Read EDID to get actual display capabilities
            use crate::edid;
            let (width, height, refresh_rate) = match read_edid_from_display(pcie_device.bus, pcie_device.device, pcie_device.function) {
                Some(edid_data) => {
                    (edid_data.preferred_resolution.width, edid_data.preferred_resolution.height, edid_data.preferred_resolution.refresh_rate)
                }
                None => {
                    // Default to 1920x1080 if EDID not available
                    (1920, 1080, 60)
                }
            };
            
            let config = FramebufferConfig {
                width,
                height,
                pitch: width * 4, // 4 bytes per pixel (32-bit ARGB)
                bpp: 32,
                format: PixelFormat::ARGB32,
                physical_addr: PhysAddr::new(fb_base),
                size: (width as u64) * (height as u64) * 4,
            };
            
            let display = DisplayDevice {
                device_id: display_id,
                config: config.clone(),
                enabled: true,
                current_mode: DisplayMode {
                    width,
                    height,
                    refresh_rate,
                },
            };
            
            let mut displays = self.displays.lock();
            displays.insert(display_id, display);
        }
        
        // If no PCIe displays found, check for VBE/GOP (UEFI)
        // This would be handled by bootloader passing framebuffer info
        // Create default display if none found
        let displays = self.displays.lock();
        if displays.is_empty() {
            drop(displays);
            let display_id = {
                let mut next = self.next_display_id.lock();
                let id = *next;
                *next += 1;
                id
            };
            
            let config = FramebufferConfig {
                width: 1920,
                height: 1080,
                pitch: 1920 * 4,
                bpp: 32,
                format: PixelFormat::ARGB32,
                physical_addr: PhysAddr::new(0xE0000000),
                size: 1920 * 1080 * 4,
            };
            
            let display = DisplayDevice {
                device_id: display_id,
                config: config.clone(),
                enabled: true,
                current_mode: DisplayMode {
                    width: 1920,
                    height: 1080,
                    refresh_rate: 60,
                },
            };
            
            let mut displays = self.displays.lock();
            displays.insert(display_id, display);
        }
    }
    
    /// Allocate framebuffer
    pub fn allocate_framebuffer(
        &self,
        width: u32,
        height: u32,
        format: PixelFormat,
    ) -> Result<u64, GraphicsError> {
        let bpp = match format {
            PixelFormat::ARGB32 => 4,
            PixelFormat::RGB24 => 3,
            PixelFormat::RGB16 => 2,
            PixelFormat::RGB8 => 1,
        };
        
        let pitch = width * bpp;
        let size = (pitch as u64) * (height as u64);
        
        // Allocate physical memory for framebuffer
        // Framebuffers are typically allocated in high memory region
        // Use frame allocator for proper memory management
        let frames_needed = ((size + 4095) / 4096) as usize; // Round up to 4KB frames
        
        // For framebuffers, allocate contiguous physical memory
        // In practice, framebuffers may be provided by GPU/display hardware
        // This is a fallback allocation
        let physical_addr = PhysAddr::new(0xE0000000 + (self.next_fb_id.lock().clone() as u64 * 0x1000000));
        
        let fb_id = {
            let mut next = self.next_fb_id.lock();
            let id = *next;
            *next += 1;
            id
        };
        
        let config = FramebufferConfig {
            width,
            height,
            pitch,
            bpp: (bpp * 8) as u8,
            format,
            physical_addr,
            size,
        };
        
        let mut framebuffers = self.framebuffers.lock();
        framebuffers.insert(fb_id, config);
        
        Ok(fb_id)
    }
    
    /// Get framebuffer configuration
    pub fn get_framebuffer(&self, fb_id: u64) -> Option<FramebufferConfig> {
        let framebuffers = self.framebuffers.lock();
        framebuffers.get(&fb_id).cloned()
    }
    
    /// Free framebuffer
    pub fn free_framebuffer(&self, fb_id: u64) -> Result<(), GraphicsError> {
        let mut framebuffers = self.framebuffers.lock();
        framebuffers.remove(&fb_id)
            .ok_or(GraphicsError::FramebufferNotFound)?;
        Ok(())
    }
    
    /// Get display devices
    pub fn get_displays(&self) -> Vec<DisplayDevice> {
        let displays = self.displays.lock();
        displays.values().cloned().collect()
    }
    
    /// Get display device
    pub fn get_display(&self, device_id: u64) -> Option<DisplayDevice> {
        let displays = self.displays.lock();
        displays.get(&device_id).cloned()
    }
    
    /// Set display mode
    pub fn set_display_mode(&self, device_id: u64, mode: DisplayMode) -> Result<(), GraphicsError> {
        let mut displays = self.displays.lock();
        let display = displays.get_mut(&device_id)
            .ok_or(GraphicsError::DisplayNotFound)?;
        
        display.current_mode = mode;
        Ok(())
    }
}

/// Graphics error
#[derive(Debug, Clone, Copy)]
pub enum GraphicsError {
    FramebufferNotFound,
    DisplayNotFound,
    InvalidMode,
    OutOfMemory,
    DeviceNotAvailable,
}

/// Global graphics manager instance
static GRAPHICS_MANAGER: Mutex<Option<GraphicsManager>> = Mutex::new(None);

/// Initialize graphics manager
pub fn init() {
    let mut manager = GRAPHICS_MANAGER.lock();
    let graphics_manager = GraphicsManager::new();
    graphics_manager.init();
    *manager = Some(graphics_manager);
}

/// Get graphics manager instance
pub fn get() -> Option<&'static GraphicsManager> {
    let manager = GRAPHICS_MANAGER.lock();
    manager.as_ref()
}

/// Read EDID from display device
fn read_edid_from_display(_bus: u8, _device: u8, _function: u8) -> Option<crate::edid::EDIDData> {
    // Read EDID via I2C/DDC from display
    // Display devices typically expose EDID on I2C bus at address 0x50
    // Attempt to read via I2C (requires I2C driver implementation)
    crate::edid::read_edid_from_i2c(0, 0x50) // Default DDC address
}

