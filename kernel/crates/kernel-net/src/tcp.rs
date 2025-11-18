//! TCP layer
//! 
//! TCP connection management and data transfer

use crate::ip::IpAddress;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

/// TCP state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

/// TCP connection
pub struct TcpConnection {
    pub src_addr: IpAddress,
    pub src_port: u16,
    pub dst_addr: IpAddress,
    pub dst_port: u16,
    pub state: TcpState,
    pub seq: u32,
    pub ack: u32,
    pub window: u16,
}

impl TcpConnection {
    pub fn new(src_addr: IpAddress, src_port: u16, dst_addr: IpAddress, dst_port: u16) -> Self {
        Self {
            src_addr,
            src_port,
            dst_addr,
            dst_port,
            state: TcpState::Closed,
            seq: 0,
            ack: 0,
            window: 0,
        }
    }
    
    /// Connect (initiate TCP handshake)
    pub fn connect(&mut self) -> Result<(), TcpError> {
        // TODO: Send SYN packet
        self.state = TcpState::SynSent;
        Ok(())
    }
    
    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, TcpError> {
        if self.state != TcpState::Established {
            return Err(TcpError::NotConnected);
        }
        // TODO: Send TCP segment
        Ok(data.len())
    }
    
    /// Receive data
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, TcpError> {
        if self.state != TcpState::Established {
            return Err(TcpError::NotConnected);
        }
        // TODO: Receive TCP segment
        Ok(0)
    }
}

/// TCP error
#[derive(Debug, Clone, Copy)]
pub enum TcpError {
    NotConnected,
    ConnectionReset,
    Timeout,
    InvalidState,
}

