//! UDP layer
//! 
//! UDP datagram handling

use crate::ip::IpAddress;

/// UDP header
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

/// Send UDP datagram
pub fn send_udp(src_addr: IpAddress, src_port: u16, dst_addr: IpAddress, dst_port: u16, data: &[u8]) -> Result<(), UdpError> {
    // TODO: Send UDP datagram
    Ok(())
}

/// Receive UDP datagram
pub fn recv_udp(buf: &mut [u8]) -> Result<(IpAddress, u16, usize), UdpError> {
    // TODO: Receive UDP datagram
    Err(UdpError::NoData)
}

/// UDP error
#[derive(Debug, Clone, Copy)]
pub enum UdpError {
    NoData,
    InvalidHeader,
    ChecksumFailed,
}

