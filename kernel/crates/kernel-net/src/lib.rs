#![no_std]

//! Network stack
//! 
//! TCP/IP implementation with agent isolation and capability-based access control

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod ip;
pub mod tcp;
pub mod udp;
pub mod socket;
pub mod routing;
pub mod firewall;
pub mod namespace;

use spin::Mutex;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use socket::{Socket, SocketType};

#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
#[cfg(feature = "alloc")]
// Time dependency removed to break circular dependency with kernel-core
#[cfg(feature = "alloc")]
use aios_kernel_observability::update_network_bytes;
#[cfg(feature = "alloc")]
use aios_kernel_policy::check_policy;

/// Network manager
pub struct NetworkManager {
    #[cfg(feature = "alloc")]
    sockets: Mutex<BTreeMap<u64, Socket>>, // Socket ID -> Socket
    #[cfg(feature = "alloc")]
    next_socket_id: Mutex<u64>,
    #[cfg(not(feature = "alloc"))]
    sockets: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_socket_id: Mutex<()>,
}

static NET_MANAGER: Mutex<Option<NetworkManager>> = Mutex::new(None);

/// Initialize network manager
pub fn init() {
    let mut manager = NET_MANAGER.lock();
    *manager = Some(NetworkManager {
        #[cfg(feature = "alloc")]
        sockets: Mutex::new(BTreeMap::new()),
        #[cfg(feature = "alloc")]
        next_socket_id: Mutex::new(1),
        #[cfg(not(feature = "alloc"))]
        sockets: Mutex::new(()),
        #[cfg(not(feature = "alloc"))]
        next_socket_id: Mutex::new(()),
    });
    
    // Initialize IP layer
    ip::init();
    
    // Initialize routing table
    routing::RoutingTable::init();
    
    // Initialize firewall
    firewall::FirewallManager::init();
    
    // Initialize namespace manager
    namespace::NamespaceManager::init();
}

/// Create agent network endpoint (agent-scoped, not traditional socket)
#[cfg(feature = "alloc")]
pub fn create_agent_endpoint(agent_id: u64, socket_type: SocketType, capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<u64, NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        // Check capability if provided
        if let Some(cap) = capability {
            use aios_kernel_capability::{capability::Capabilities, has_capability};
            if !has_capability(Some(cap), Capabilities::ACCESS_NET) {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec![("action".into(), "net_access_denied".into())],
                    });
                }
                return Err(NetError::PermissionDenied);
            }
        } else {
            // Capability required for network access
            #[cfg(feature = "alloc")]
            {
                publish_event(KernelEvent {
                    event_type: EventType::Security,
                    timestamp: 0, // Timestamp should be set by caller
                    agent_id: Some(agent_id),
                    data: alloc::vec![("action".into(), "net_no_capability".into())],
                });
            }
            return Err(NetError::PermissionDenied);
        }
        
        // Check policy
        #[cfg(feature = "alloc")]
        {
            if !check_policy(agent_id, "net_create_endpoint", None) {
                return Err(NetError::PermissionDenied);
            }
        }
        
        // Create network namespace for agent if it doesn't exist
        #[cfg(feature = "alloc")]
        {
            if let Some(namespace_mgr) = namespace::NamespaceManager::get() {
                if namespace_mgr.get_namespace(agent_id).is_none() {
                    namespace_mgr.create_namespace(agent_id);
                }
            }
        }
        
        let mut next_id = manager.next_socket_id.lock();
        let socket_id = *next_id;
        *next_id += 1;
        
        let socket = Socket::new(socket_id, agent_id, socket_type);
        let mut sockets = manager.sockets.lock();
        sockets.insert(socket_id, socket);
        
        // Publish network event
        #[cfg(feature = "alloc")]
        {
            publish_event(KernelEvent {
                event_type: EventType::Network,
                timestamp: 0, // Timestamp should be set by caller
                agent_id: Some(agent_id),
                data: alloc::vec::Vec::new(),
            });
        }
        
        Ok(socket_id)
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn create_agent_endpoint(_agent_id: u64, _socket_type: SocketType, _capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> Result<u64, NetError> {
    Err(NetError::NotInitialized)
}

/// Bind agent endpoint to address (agent-scoped, not traditional socket bind)
#[cfg(feature = "alloc")]
pub fn bind_agent_endpoint(socket_id: u64, agent_id: u64, addr: &ip::IpAddress, port: u16) -> Result<(), NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get_mut(&socket_id) {
            // Check agent ownership
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            let result = socket.bind_agent_endpoint(*addr, port);
            if result.is_ok() {
                // Publish network event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Network,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
            }
            result
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn bind_agent_endpoint(_socket_id: u64, _agent_id: u64, _addr: &ip::IpAddress, _port: u16) -> Result<(), NetError> {
    Err(NetError::NotInitialized)
}

/// Listen on agent endpoint (agent-scoped, not traditional socket listen)
#[cfg(feature = "alloc")]
pub fn listen_agent_endpoint(socket_id: u64, agent_id: u64, backlog: usize) -> Result<(), NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get_mut(&socket_id) {
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            let result = socket.listen_agent_endpoint(backlog);
            if result.is_ok() {
                // Publish network event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Network,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
            }
            result
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn listen_agent_endpoint(_socket_id: u64, _agent_id: u64, _backlog: usize) -> Result<(), NetError> {
    Err(NetError::NotInitialized)
}

/// Accept agent connection (agent-scoped, not traditional socket accept)
#[cfg(feature = "alloc")]
pub fn accept_agent_connection(socket_id: u64, agent_id: u64) -> Result<u64, NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get_mut(&socket_id) {
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            // Accept connection and create new socket
            let mut next_id = manager.next_socket_id.lock();
            let new_socket_id = *next_id;
            *next_id += 1;
            
            if let Some(new_socket) = socket.accept_agent_connection(new_socket_id, agent_id) {
                sockets.insert(new_socket_id, new_socket);
                // Publish network event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Network,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                Ok(new_socket_id)
            } else {
                Err(NetError::ConnectionRefused)
            }
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn accept_agent_connection(_socket_id: u64, _agent_id: u64) -> Result<u64, NetError> {
    Err(NetError::NotInitialized)
}

/// Connect agent endpoint (agent-scoped, not traditional socket connect)
#[cfg(feature = "alloc")]
pub fn connect_agent_endpoint(socket_id: u64, agent_id: u64, addr: &ip::IpAddress, port: u16) -> Result<(), NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get_mut(&socket_id) {
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            let result = socket.connect_agent_endpoint(*addr, port);
            if result.is_ok() {
                // Publish network event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Network,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
            }
            result
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn connect_agent_endpoint(_socket_id: u64, _agent_id: u64, _addr: &ip::IpAddress, _port: u16) -> Result<(), NetError> {
    Err(NetError::NotInitialized)
}

/// Send data
#[cfg(feature = "alloc")]
pub fn send_data(socket_id: u64, agent_id: u64, data: &[u8]) -> Result<usize, NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get(&socket_id) {
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            // Check firewall rules before sending
            #[cfg(feature = "alloc")]
            {
                if let Some(firewall) = firewall::FirewallManager::get() {
                    let local_addr = socket.local_addr().unwrap_or(ip::IpAddress::new(0, 0, 0, 0));
                    let remote_addr = socket.remote_addr().unwrap_or(ip::IpAddress::new(0, 0, 0, 0));
                    let local_port = socket.local_port();
                    let remote_port = socket.remote_port();
                    let protocol = match socket.socket_type() {
                        socket::SocketType::Tcp => Some(6), // TCP
                        socket::SocketType::Udp => Some(17), // UDP
                        _ => None,
                    };
                    
                    if !firewall.check_packet(
                        firewall::FirewallDirection::Outbound,
                        local_addr,
                        remote_addr,
                        local_port,
                        remote_port,
                        protocol,
                        Some(agent_id),
                    ) {
                        return Err(NetError::PermissionDenied);
                    }
                }
            }
            
            let result = socket.send(data);
            if let Ok(bytes_sent) = result {
                // Update observability counters
                #[cfg(feature = "alloc")]
                {
                    update_network_bytes(agent_id, bytes_sent as u64, 0);
                }
                // Publish network event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Network,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
            }
            result
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn send_data(_socket_id: u64, _agent_id: u64, _data: &[u8]) -> Result<usize, NetError> {
    Err(NetError::NotInitialized)
}

/// Receive data
#[cfg(feature = "alloc")]
pub fn recv_data(socket_id: u64, agent_id: u64, buf: &mut [u8]) -> Result<usize, NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get_mut(&socket_id) {
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            
            // Check firewall rules before receiving
            #[cfg(feature = "alloc")]
            {
                if let Some(firewall) = firewall::FirewallManager::get() {
                    let local_addr = socket.local_addr().unwrap_or(ip::IpAddress::new(0, 0, 0, 0));
                    let remote_addr = socket.remote_addr().unwrap_or(ip::IpAddress::new(0, 0, 0, 0));
                    let local_port = socket.local_port();
                    let remote_port = socket.remote_port();
                    let protocol = match socket.socket_type() {
                        socket::SocketType::Tcp => Some(6), // TCP
                        socket::SocketType::Udp => Some(17), // UDP
                        _ => None,
                    };
                    
                    if !firewall.check_packet(
                        firewall::FirewallDirection::Inbound,
                        remote_addr,
                        local_addr,
                        remote_port,
                        local_port,
                        protocol,
                        Some(agent_id),
                    ) {
                        return Err(NetError::PermissionDenied);
                    }
                }
            }
            
            let result = socket.recv(buf);
            if let Ok(bytes_received) = result {
                // Update observability counters
                #[cfg(feature = "alloc")]
                {
                    update_network_bytes(agent_id, 0, bytes_received as u64);
                }
                // Publish network event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Network,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
            }
            result
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn recv_data(_socket_id: u64, _agent_id: u64, _buf: &mut [u8]) -> Result<usize, NetError> {
    Err(NetError::NotInitialized)
}

/// Close agent endpoint (agent-scoped, not traditional socket close)
#[cfg(feature = "alloc")]
pub fn close_agent_endpoint(socket_id: u64, agent_id: u64) -> Result<(), NetError> {
    let manager = NET_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let mut sockets = manager.sockets.lock();
        if let Some(socket) = sockets.get(&socket_id) {
            if socket.agent_id() != agent_id {
                // Publish security violation event
                #[cfg(feature = "alloc")]
                {
                    publish_event(KernelEvent {
                        event_type: EventType::Security,
                        timestamp: 0, // Timestamp should be set by caller to avoid circular dependency
                        agent_id: Some(agent_id),
                        data: alloc::vec::Vec::new(),
                    });
                }
                return Err(NetError::PermissionDenied);
            }
            sockets.remove(&socket_id);
            // Publish network event
            #[cfg(feature = "alloc")]
            {
                publish_event(KernelEvent {
                    event_type: EventType::Network,
                    timestamp: 0, // Timestamp should be set by caller
                    agent_id: Some(agent_id),
                    data: alloc::vec::Vec::new(),
                });
            }
            Ok(())
        } else {
            Err(NetError::NotFound)
        }
    } else {
        Err(NetError::NotInitialized)
    }
}

#[cfg(not(feature = "alloc"))]
pub fn close_agent_endpoint(_socket_id: u64, _agent_id: u64) -> Result<(), NetError> {
    Err(NetError::NotInitialized)
}

/// Network error
#[derive(Debug, Clone, Copy)]
pub enum NetError {
    NotInitialized,
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    Timeout,
    NetworkUnreachable,
    IoError,
    InvalidAddress,
    InvalidPort,
}
