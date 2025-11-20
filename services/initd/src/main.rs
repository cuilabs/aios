//! Init Daemon (initd) - PID 1
//! 
//! System initialization and service management daemon

#![no_std]
#![no_main]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::panic::PanicInfo;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use spin::Mutex;

/// Service state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

/// Service manager
pub struct ServiceManager {
    #[cfg(feature = "alloc")]
    services: Mutex<BTreeMap<alloc::string::String, ServiceState>>,
    #[cfg(not(feature = "alloc"))]
    services: Mutex<()>,
}

static SERVICE_MANAGER: Mutex<Option<ServiceManager>> = Mutex::new(None);

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            services: Mutex::new(BTreeMap::new()),
            #[cfg(not(feature = "alloc"))]
            services: Mutex::new(()),
        }
    }

    /// Register service
    #[cfg(feature = "alloc")]
    pub fn register_service(&self, name: alloc::string::String) {
        let mut services = self.services.lock();
        services.insert(name, ServiceState::Stopped);
    }

    #[cfg(not(feature = "alloc"))]
    pub fn register_service(&self, _name: &str) {
        // No-op
    }

    /// Start service
    #[cfg(feature = "alloc")]
    pub fn start_service(&self, name: &str) -> Result<(), ServiceError> {
        let mut services = self.services.lock();
        if let Some(state) = services.get_mut(name) {
            *state = ServiceState::Starting;
            // Spawn service process
            *state = ServiceState::Running;
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn start_service(&self, _name: &str) -> Result<(), ServiceError> {
        Err(ServiceError::ServiceNotFound)
    }

    /// Stop service
    #[cfg(feature = "alloc")]
    pub fn stop_service(&self, name: &str) -> Result<(), ServiceError> {
        let mut services = self.services.lock();
        if let Some(state) = services.get_mut(name) {
            *state = ServiceState::Stopping;
            // Terminate service process
            *state = ServiceState::Stopped;
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn stop_service(&self, _name: &str) -> Result<(), ServiceError> {
        Err(ServiceError::ServiceNotFound)
    }

    /// Get service state
    #[cfg(feature = "alloc")]
    pub fn get_service_state(&self, name: &str) -> Option<ServiceState> {
        let services = self.services.lock();
        services.get(name).copied()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_service_state(&self, _name: &str) -> Option<ServiceState> {
        None
    }
}

/// Service error
#[derive(Debug, Clone, Copy)]
pub enum ServiceError {
    ServiceNotFound,
    AlreadyRunning,
    AlreadyStopped,
    StartFailed,
}

/// Initialize initd
pub fn init() {
    let mut manager = SERVICE_MANAGER.lock();
    *manager = Some(ServiceManager::new());
    
    #[cfg(feature = "alloc")]
    {
        // Register core services
        if let Some(ref manager) = *manager {
            manager.register_service(alloc::string::String::from("memoryd"));
            manager.register_service(alloc::string::String::from("planner"));
            manager.register_service(alloc::string::String::from("semantic-ipcd"));
        }
    }
}

/// Start all services
#[cfg(feature = "alloc")]
pub fn start_all_services() {
    let manager = SERVICE_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let services = manager.services.lock();
        let service_names: Vec<alloc::string::String> = services.keys().cloned().collect();
        drop(services);
        
        for name in service_names {
            let _ = manager.start_service(&name);
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub fn start_all_services() {
    // No-op
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    start_all_services();
    loop {}
}

