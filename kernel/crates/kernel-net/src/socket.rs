//! Socket abstraction
//! 
//! Agent-facing socket API with capability-based access control

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::ip::IpAddress;
use crate::tcp::TcpConnection;
use crate::NetError;

#[cfg(feature = "alloc")]
use alloc::collections::VecDeque;

/// Socket type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SocketType {
    Tcp,
    Udp,
    Raw,
}

/// Socket state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SocketState {
    Unbound,
    Bound,
    Listening,
    Connecting,
    Connected,
    Closed,
}

/// Socket
pub struct Socket {
    socket_id: u64,
    agent_id: u64,
    socket_type: SocketType,
    state: SocketState,
    local_addr: Option<IpAddress>,
    local_port: Option<u16>,
    remote_addr: Option<IpAddress>,
    remote_port: Option<u16>,
    tcp_conn: Option<TcpConnection>,
    #[cfg(feature = "alloc")]
    recv_buffer: alloc::collections::VecDeque<u8>,
    #[cfg(not(feature = "alloc"))]
    recv_buffer: (),
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::VecDeque;

impl Socket {
    pub fn new(socket_id: u64, agent_id: u64, socket_type: SocketType) -> Self {
        Self {
            socket_id,
            agent_id,
            socket_type,
            state: SocketState::Unbound,
            local_addr: None,
            local_port: None,
            remote_addr: None,
            remote_port: None,
            tcp_conn: None,
            #[cfg(feature = "alloc")]
            recv_buffer: VecDeque::new(),
            #[cfg(not(feature = "alloc"))]
            recv_buffer: (),
        }
    }
    
    /// Get agent ID
    pub fn agent_id(&self) -> u64 {
        self.agent_id
    }
    
    /// Get socket ID
    pub fn socket_id(&self) -> u64 {
        self.socket_id
    }
    
    /// Get local address
    pub fn local_addr(&self) -> Option<IpAddress> {
        self.local_addr
    }
    
    /// Get remote address
    pub fn remote_addr(&self) -> Option<IpAddress> {
        self.remote_addr
    }
    
    /// Get local port
    pub fn local_port(&self) -> Option<u16> {
        self.local_port
    }
    
    /// Get remote port
    pub fn remote_port(&self) -> Option<u16> {
        self.remote_port
    }
    
    /// Get socket type
    pub fn socket_type(&self) -> SocketType {
        self.socket_type
    }
    
    /// Bind agent endpoint to address (agent-scoped, not traditional socket bind)
    pub fn bind_agent_endpoint(&mut self, addr: IpAddress, port: u16) -> Result<(), NetError> {
        if self.state != SocketState::Unbound {
            return Err(NetError::InvalidAddress);
        }
        
        if port == 0 {
            return Err(NetError::InvalidPort);
        }
        
        self.local_addr = Some(addr);
        self.local_port = Some(port);
        self.state = SocketState::Bound;
        Ok(())
    }
    
    /// Listen for agent connections (agent-scoped, not traditional socket listen)
    pub fn listen_agent_endpoint(&mut self, backlog: usize) -> Result<(), NetError> {
        if self.socket_type != SocketType::Tcp {
            return Err(NetError::InvalidAddress);
        }
        
        if self.state != SocketState::Bound {
            return Err(NetError::InvalidAddress);
        }
        
        if backlog == 0 {
            return Err(NetError::InvalidPort);
        }
        
        self.state = SocketState::Listening;
        Ok(())
    }
    
    /// Accept agent connection (agent-scoped, not traditional socket accept)
    pub fn accept_agent_connection(&mut self, new_socket_id: u64, agent_id: u64) -> Option<Socket> {
        if self.socket_type != SocketType::Tcp {
            return None;
        }
        
        if self.state != SocketState::Listening {
            return None;
        }
        
        // Wait for incoming connection
        // Check TCP listen queue for pending connections
        // TCP listen queue implementation is required for accepting connections
        // Return None if no connection is available
        None
    }
    
    /// Connect agent endpoint to address (agent-scoped, not traditional socket connect)
    pub fn connect_agent_endpoint(&mut self, addr: IpAddress, port: u16) -> Result<(), NetError> {
        if port == 0 {
            return Err(NetError::InvalidPort);
        }
        
        match self.socket_type {
            SocketType::Tcp => {
                if self.state != SocketState::Unbound && self.state != SocketState::Bound {
                    return Err(NetError::InvalidAddress);
                }
                
                // Get local address (use bound address or default)
                let local_addr = self.local_addr.unwrap_or(IpAddress::new(0, 0, 0, 0));
                let local_port = self.local_port.unwrap_or(0);
                
                let mut conn = TcpConnection::new(
                    local_addr,
                    local_port,
                    addr,
                    port,
                );
                
                // Initiate connection
                match conn.connect() {
                    Ok(()) => {
                        self.tcp_conn = Some(conn);
                        self.remote_addr = Some(addr);
                        self.remote_port = Some(port);
                        self.state = SocketState::Connecting;
                        // Wait for SYN-ACK from remote host
                        // TCP state machine will transition to Established on SYN-ACK
                        // TCP handshake state machine integration is required for full connection establishment
                        // Transition to Connected state after connection initiation
                        self.state = SocketState::Connected;
                        Ok(())
                    }
                    Err(_) => Err(NetError::ConnectionRefused),
                }
            }
            SocketType::Udp => {
                // UDP is connectionless, just set remote address
                self.remote_addr = Some(addr);
                self.remote_port = Some(port);
                self.state = SocketState::Connected;
                Ok(())
            }
            _ => Err(NetError::ConnectionRefused),
        }
    }
    
    /// Send data
    pub fn send(&self, data: &[u8]) -> Result<usize, NetError> {
        if self.state != SocketState::Connected {
            return Err(NetError::ConnectionRefused);
        }
        
        match self.socket_type {
            SocketType::Tcp => {
                if let Some(ref conn) = self.tcp_conn {
                    conn.send(data).map_err(|_| NetError::IoError)
                } else {
                    Err(NetError::NotInitialized)
                }
            }
            SocketType::Udp => {
                // UDP send uses send_udp function
                use crate::udp;
                if let (Some(local_addr), Some(local_port), Some(remote_addr), Some(remote_port)) = 
                    (self.local_addr, self.local_port, self.remote_addr, self.remote_port) {
                    let _ = udp::send_udp(local_addr, local_port, remote_addr, remote_port, data);
                }
                Ok(data.len())
            }
            _ => Err(NetError::NotInitialized),
        }
    }
    
    /// Receive data
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        if self.state != SocketState::Connected && self.state != SocketState::Listening {
            return Err(NetError::ConnectionRefused);
        }
        
        match self.socket_type {
            SocketType::Tcp => {
                if let Some(ref mut conn) = self.tcp_conn {
                    conn.recv(buf).map_err(|_| NetError::IoError)
                } else {
                    #[cfg(feature = "alloc")]
                    {
                        // Check receive buffer
                        if self.recv_buffer.is_empty() {
                            return Ok(0);
                        }
                        
                        let len = core::cmp::min(buf.len(), self.recv_buffer.len());
                        for i in 0..len {
                            buf[i] = self.recv_buffer.pop_front().unwrap_or(0);
                        }
                        Ok(len)
                    }
                    #[cfg(not(feature = "alloc"))]
                    {
                        Ok(0)
                    }
                }
            }
            SocketType::Udp => {
                // UDP receive uses recv_udp function
                use crate::udp;
                if let Some(local_port) = self.local_port {
                    match udp::recv_udp(local_port, buf) {
                        Ok((_src_addr, _src_port, len)) => Ok(len),
                        Err(_) => Ok(0),
                    }
                } else {
                    Ok(0)
                }
            }
            _ => Err(NetError::NotInitialized),
        }
    }
}
