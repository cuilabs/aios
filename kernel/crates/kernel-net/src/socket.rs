//! Socket abstraction
//! 
//! Agent-facing socket API

use crate::ip::IpAddress;
use crate::tcp::TcpConnection;
use crate::NetError;

/// Socket
pub struct Socket {
    pub socket_id: u64,
    pub agent_id: u64,
    pub socket_type: crate::SocketType,
    pub tcp_conn: Option<TcpConnection>,
}

impl Socket {
    pub fn new(socket_id: u64, agent_id: u64, socket_type: crate::SocketType) -> Self {
        Self {
            socket_id,
            agent_id,
            socket_type,
            tcp_conn: None,
        }
    }
    
    /// Bind socket to address
    pub fn bind(&mut self, addr: IpAddress, port: u16) -> Result<(), NetError> {
        // TODO: Bind socket
        Ok(())
    }
    
    /// Listen for connections (TCP)
    pub fn listen(&mut self, backlog: usize) -> Result<(), NetError> {
        // TODO: Start listening
        Ok(())
    }
    
    /// Accept connection (TCP)
    pub fn accept(&mut self) -> Result<u64, NetError> {
        // TODO: Accept connection
        Err(NetError::ConnectionRefused)
    }
    
    /// Connect to address
    pub fn connect(&mut self, addr: IpAddress, port: u16) -> Result<(), NetError> {
        match self.socket_type {
            crate::SocketType::Tcp => {
                let mut conn = TcpConnection::new(
                    IpAddress::new(0, 0, 0, 0), // TODO: Get local IP
                    0, // TODO: Get local port
                    addr,
                    port,
                );
                conn.connect()?;
                self.tcp_conn = Some(conn);
                Ok(())
            }
            _ => Err(NetError::ConnectionRefused),
        }
    }
    
    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, NetError> {
        match self.socket_type {
            crate::SocketType::Tcp => {
                if let Some(ref mut conn) = self.tcp_conn {
                    conn.send(data).map_err(|_| NetError::IoError)
                } else {
                    Err(NetError::NotInitialized)
                }
            }
            _ => Err(NetError::NotInitialized),
        }
    }
    
    /// Receive data
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        match self.socket_type {
            crate::SocketType::Tcp => {
                if let Some(ref mut conn) = self.tcp_conn {
                    conn.recv(buf).map_err(|_| NetError::IoError)
                } else {
                    Err(NetError::NotInitialized)
                }
            }
            _ => Err(NetError::NotInitialized),
        }
    }
}

