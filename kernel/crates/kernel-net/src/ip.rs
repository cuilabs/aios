//! IP layer
//! 
//! IPv4/IPv6 packet handling

/// IP version
#[derive(Clone, Copy, Debug)]
pub enum IpVersion {
    V4,
    V6,
}

/// IP address (IPv4)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

/// IP header
pub struct IpHeader {
    pub version: IpVersion,
    pub src: IpAddress,
    pub dst: IpAddress,
    pub protocol: u8,
    pub ttl: u8,
}

/// Process IP packet
pub fn process_ip_packet(packet: &[u8]) -> Result<(), IpError> {
    // TODO: Parse IP header and route to appropriate protocol handler
    Ok(())
}

/// IP error
#[derive(Debug, Clone, Copy)]
pub enum IpError {
    InvalidHeader,
    ChecksumFailed,
    TtlExceeded,
    UnsupportedVersion,
}

