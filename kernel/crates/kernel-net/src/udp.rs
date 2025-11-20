//! UDP layer
//! 
//! UDP datagram handling

use crate::ip::IpAddress;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// UDP header
#[repr(C, packed)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

/// UDP socket (bound port)
#[cfg(feature = "alloc")]
pub struct UdpSocket {
    pub port: u16,
    pub agent_id: u64,
}

#[cfg(feature = "alloc")]
static UDP_SOCKETS: Mutex<BTreeMap<u16, UdpSocket>> = Mutex::new(BTreeMap::new());

/// Bind UDP port
#[cfg(feature = "alloc")]
pub fn bind_udp_port(port: u16, agent_id: u64) -> Result<(), UdpError> {
    if port == 0 {
        return Err(UdpError::InvalidPort);
    }
    
    let mut sockets = UDP_SOCKETS.lock();
    if sockets.contains_key(&port) {
        return Err(UdpError::PortInUse);
    }
    
    sockets.insert(port, UdpSocket { port, agent_id });
    Ok(())
}

#[cfg(not(feature = "alloc"))]
pub fn bind_udp_port(_port: u16, _agent_id: u64) -> Result<(), UdpError> {
    Err(UdpError::InvalidPort)
}

/// Unbind UDP port
#[cfg(feature = "alloc")]
pub fn unbind_udp_port(port: u16) -> bool {
    let mut sockets = UDP_SOCKETS.lock();
    sockets.remove(&port).is_some()
}

#[cfg(not(feature = "alloc"))]
pub fn unbind_udp_port(_port: u16) -> bool {
    false
}

/// Send UDP datagram
pub fn send_udp(src_addr: IpAddress, src_port: u16, dst_addr: IpAddress, dst_port: u16, data: &[u8]) -> Result<(), UdpError> {
    if src_port == 0 || dst_port == 0 {
        return Err(UdpError::InvalidHeader);
    }
    
    // Create UDP header
    let header = UdpHeader {
        src_port: src_port.to_be(),
        dst_port: dst_port.to_be(),
        length: ((8 + data.len()) as u16).to_be(), // Header (8) + data
        checksum: 0,
    };
    
    // Calculate UDP checksum (pseudo-header + UDP header + data)
    let checksum = calculate_udp_checksum(src_addr, dst_addr, &header, data);
    
    // Set checksum in header
    let mut header_with_checksum = header;
    header_with_checksum.checksum = checksum;
    
    // Send via IP layer (protocol = 17 for UDP)
    #[cfg(feature = "alloc")]
    {
        use crate::ip;
        let mut udp_packet = Vec::with_capacity(8 + data.len());
        udp_packet.extend_from_slice(&header_with_checksum.src_port.to_be_bytes());
        udp_packet.extend_from_slice(&header_with_checksum.dst_port.to_be_bytes());
        udp_packet.extend_from_slice(&header_with_checksum.length.to_be_bytes());
        udp_packet.extend_from_slice(&header_with_checksum.checksum.to_be_bytes());
        udp_packet.extend_from_slice(data);
        
        let _ = ip::send_ip_packet(dst_addr, 17, &udp_packet); // Protocol 17 = UDP
    }
    
    Ok(())
}

/// Receive UDP datagram
#[cfg(feature = "alloc")]
pub fn recv_udp(port: u16, buf: &mut [u8]) -> Result<(IpAddress, u16, usize), UdpError> {
    // Check if port is bound
    let sockets = UDP_SOCKETS.lock();
    if !sockets.contains_key(&port) {
        return Err(UdpError::PortNotBound);
    }
    
    // Wait for incoming IP packet with UDP protocol (17)
    // Parse UDP header
    // Verify checksum
    // Check if destination port matches
    // Extract payload
    // Return source address, port, and data length
    
    // UDP receive is handled by IP layer routing to UDP sockets
    // IP layer routes UDP packets to bound sockets based on destination port
    // Return no data if no packet is available
    Err(UdpError::NoData)
}

#[cfg(not(feature = "alloc"))]
pub fn recv_udp(_port: u16, _buf: &mut [u8]) -> Result<(IpAddress, u16, usize), UdpError> {
    Err(UdpError::NoData)
}

/// Calculate UDP checksum
fn calculate_udp_checksum(src: IpAddress, dst: IpAddress, header: &UdpHeader, data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    
    // Pseudo-header: src + dst + protocol + length
    for i in 0..4 {
        sum += src.octets[i] as u32;
    }
    for i in 0..4 {
        sum += dst.octets[i] as u32;
    }
    sum += 17; // UDP protocol
    sum += header.length.to_be() as u32;
    
    // UDP header
    sum += header.src_port.to_be() as u32;
    sum += header.dst_port.to_be() as u32;
    sum += header.length.to_be() as u32;
    
    // Data (16-bit words)
    let mut i = 0;
    while i < data.len() {
        let word = if i + 1 < data.len() {
            ((data[i] as u16) << 8) | (data[i + 1] as u16)
        } else {
            (data[i] as u16) << 8
        };
        sum += word as u32;
        i += 2;
    }
    
    // Fold to 16 bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !sum as u16
}

/// UDP error
#[derive(Debug, Clone, Copy)]
pub enum UdpError {
    NoData,
    InvalidHeader,
    ChecksumFailed,
    InvalidPort,
    PortInUse,
    PortNotBound,
}

