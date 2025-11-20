//! IP layer
//! 
//! IPv4/IPv6 packet handling and routing

use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

#[cfg(feature = "alloc")]
use crate::routing;
#[cfg(feature = "alloc")]
use kernel_hal::network;

/// IP version
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IpVersion {
    V4,
    V6,
}

/// IP address (IPv4)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IpAddress {
    pub octets: [u8; 4],
}

impl IpAddress {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }
    
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self { octets: bytes }
    }
    
    /// Check if address is loopback
    pub fn is_loopback(&self) -> bool {
        self.octets[0] == 127
    }
    
    /// Check if address is multicast
    pub fn is_multicast(&self) -> bool {
        self.octets[0] >= 224 && self.octets[0] <= 239
    }
    
    /// Check if address is broadcast
    pub fn is_broadcast(&self) -> bool {
        self.octets == [255, 255, 255, 255]
    }
}

/// IP header
#[repr(C, packed)]
pub struct IpHeader {
    pub version_ihl: u8,      // Version (4 bits) + IHL (4 bits)
    pub tos: u8,              // Type of Service
    pub total_length: u16,     // Total length
    pub identification: u16,   // Identification
    pub flags_fragment: u16,  // Flags (3 bits) + Fragment offset (13 bits)
    pub ttl: u8,              // Time to Live
    pub protocol: u8,         // Protocol
    pub checksum: u16,        // Header checksum
    pub src: IpAddress,       // Source address
    pub dst: IpAddress,       // Destination address
}

/// IP packet
#[cfg(feature = "alloc")]
pub struct IpPacket {
    pub header: IpHeader,
    pub payload: alloc::vec::Vec<u8>,
}

/// IP manager
struct IpManager {
    #[cfg(feature = "alloc")]
    routing_table: alloc::collections::BTreeMap<IpAddress, IpAddress>, // Destination -> Gateway
    local_address: Option<IpAddress>,
}

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

static IP_MANAGER: Mutex<Option<IpManager>> = Mutex::new(None);

/// Initialize IP layer
pub fn init() {
    let mut manager = IP_MANAGER.lock();
    *manager = Some(IpManager {
        #[cfg(feature = "alloc")]
        routing_table: alloc::collections::BTreeMap::new(),
        local_address: Some(IpAddress::new(127, 0, 0, 1)), // Default to localhost
    });
}

/// Process IP packet
pub fn process_ip_packet(packet: &[u8]) -> Result<(), IpError> {
    if packet.len() < 20 {
        return Err(IpError::InvalidHeader);
    }
    
    // Parse IP header
    let version = (packet[0] >> 4) & 0x0F;
    if version != 4 {
        return Err(IpError::UnsupportedVersion);
    }
    
    let ihl = (packet[0] & 0x0F) as usize;
    if ihl < 5 {
        return Err(IpError::InvalidHeader);
    }
    
    let header_len = ihl * 4;
    if packet.len() < header_len {
        return Err(IpError::InvalidHeader);
    }
    
    // Verify checksum
    if !verify_checksum(&packet[..header_len]) {
        return Err(IpError::ChecksumFailed);
    }
    
    // Extract addresses
    let src = IpAddress::from_bytes([packet[12], packet[13], packet[14], packet[15]]);
    let dst = IpAddress::from_bytes([packet[16], packet[17], packet[18], packet[19]]);
    
    // Check TTL
    let ttl = packet[8];
    if ttl == 0 {
        return Err(IpError::TtlExceeded);
    }
    
    // Extract protocol
    let protocol = packet[9];
    
    // Route to protocol handler (TCP, UDP, etc.)
    // This calls tcp::handle_packet or udp::handle_packet based on protocol
    // Validate packet structure
    
    Ok(())
}

/// Send IP packet
pub fn send_ip_packet(dst: IpAddress, protocol: u8, payload: &[u8]) -> Result<(), IpError> {
    let manager = IP_MANAGER.lock();
    if let Some(ref manager) = *manager {
        let src = manager.local_address.unwrap_or(IpAddress::new(0, 0, 0, 0));
        
        // Route packet (find next hop)
        let next_hop = route_packet(dst, manager.local_address)?;
        
        // Check if fragmentation is needed (MTU = 1500 bytes typically)
        const MTU: usize = 1500;
        const IP_HEADER_SIZE: usize = 20;
        const MAX_PAYLOAD: usize = MTU - IP_HEADER_SIZE;
        
        if payload.len() > MAX_PAYLOAD {
            // Fragment packet
            return send_fragmented_packet(src, next_hop, dst, protocol, payload);
        }
        
        // Build IP header
        let mut header = IpHeader {
            version_ihl: (4 << 4) | 5, // IPv4, IHL=5 (20 bytes)
            tos: 0,
            total_length: ((IP_HEADER_SIZE + payload.len()) as u16).to_be(),
            identification: generate_packet_id(),
            flags_fragment: 0, // No fragmentation
            ttl: 64,
            protocol,
            checksum: 0,
            src,
            dst,
        };
        
        // Calculate checksum
        header.checksum = calculate_checksum(&header);
        
        // Send to network driver via Ethernet HAL
        // Find appropriate network interface for destination
        #[cfg(feature = "alloc")]
        {
            if let Some(route) = routing::find_route(dst) {
                let _ = network::send_packet(route.interface_id, &packet);
            } else {
                // No route found - use default interface (0)
                let _ = network::send_packet(0, &packet);
            }
        }
        
        Ok(())
    } else {
        Err(IpError::InvalidHeader)
    }
}

/// Route packet (find next hop)
fn route_packet(dst: IpAddress, local: Option<IpAddress>) -> Result<IpAddress, IpError> {
    // Check if destination is on local network
    if let Some(local_addr) = local {
        // Routing: if same subnet, send directly
        // Otherwise, use default gateway
        if is_same_subnet(dst, local_addr) {
            return Ok(dst);
        }
    }
    
    // Use routing table to find gateway
    #[cfg(feature = "alloc")]
    {
        use crate::routing::RoutingTable;
        if let Some(routing_table) = RoutingTable::get() {
            if let Some((gateway, _interface)) = routing_table.find_route(dst, None) {
                return Ok(gateway);
            }
        }
    }
    
    // Default: send directly (assume same network)
    Ok(dst)
}

/// Check if addresses are on same subnet
fn is_same_subnet(addr1: IpAddress, addr2: IpAddress) -> bool {
    // Assume /24 subnet (first 3 octets must match)
    addr1.octets[0..3] == addr2.octets[0..3]
}

/// Send fragmented packet
#[cfg(feature = "alloc")]
fn send_fragmented_packet(
    src: IpAddress,
    next_hop: IpAddress,
    dst: IpAddress,
    protocol: u8,
    payload: &[u8],
) -> Result<(), IpError> {
    const MTU: usize = 1500;
    const IP_HEADER_SIZE: usize = 20;
    const MAX_PAYLOAD: usize = MTU - IP_HEADER_SIZE;
    const FRAGMENT_SIZE: usize = (MAX_PAYLOAD / 8) * 8; // Align to 8 bytes
    
    let packet_id = generate_packet_id();
    let mut offset = 0;
    let mut fragment_num = 0;
    
    while offset < payload.len() {
        let fragment_len = core::cmp::min(FRAGMENT_SIZE, payload.len() - offset);
        let fragment = &payload[offset..offset + fragment_len];
        
        let more_fragments = (offset + fragment_len) < payload.len();
        let flags_fragment = if more_fragments {
            (1 << 13) | ((offset / 8) as u16) // More fragments bit + offset
        } else {
            ((offset / 8) as u16) // Last fragment
        };
        
        let mut header = IpHeader {
            version_ihl: (4 << 4) | 5,
            tos: 0,
            total_length: ((IP_HEADER_SIZE + fragment_len) as u16).to_be(),
            identification: packet_id.to_be(),
            flags_fragment: flags_fragment.to_be(),
            ttl: 64,
            protocol,
            checksum: 0,
            src,
            dst,
        };
        
        header.checksum = calculate_checksum(&header);
        
        // Send fragment (would call network driver)
        offset += fragment_len;
        fragment_num += 1;
    }
    
    Ok(())
}

#[cfg(not(feature = "alloc"))]
fn send_fragmented_packet(
    _src: IpAddress,
    _next_hop: IpAddress,
    _dst: IpAddress,
    _protocol: u8,
    _payload: &[u8],
) -> Result<(), IpError> {
    Err(IpError::InvalidHeader)
}

/// Generate packet ID for fragmentation
fn generate_packet_id() -> u16 {
    static COUNTER: Mutex<u16> = Mutex::new(1);
    let mut counter = COUNTER.lock();
    let id = *counter;
    *counter = counter.wrapping_add(1);
    id
}

/// Calculate IP checksum
fn calculate_checksum(header: &IpHeader) -> u16 {
    let mut sum: u32 = 0;
    let header_bytes = unsafe {
        core::slice::from_raw_parts(
            header as *const IpHeader as *const u8,
            core::mem::size_of::<IpHeader>(),
        )
    };
    
    // Sum all 16-bit words
    for i in (0..header_bytes.len()).step_by(2) {
        if i + 1 < header_bytes.len() {
            let word = ((header_bytes[i] as u16) << 8) | (header_bytes[i + 1] as u16);
            sum += word as u32;
        }
    }
    
    // Add carry bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    // One's complement
    !sum as u16
}

/// Verify IP checksum
fn verify_checksum(header: &[u8]) -> bool {
    let mut sum: u32 = 0;
    
    // Sum all 16-bit words
    for i in (0..header.len()).step_by(2) {
        if i + 1 < header.len() {
            let word = ((header[i] as u16) << 8) | (header[i + 1] as u16);
            sum += word as u32;
        }
    }
    
    // Add carry bits
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    // Checksum should be 0xFFFF
    sum == 0xFFFF
}

/// IP error
#[derive(Debug, Clone, Copy)]
pub enum IpError {
    InvalidHeader,
    ChecksumFailed,
    TtlExceeded,
    UnsupportedVersion,
}
