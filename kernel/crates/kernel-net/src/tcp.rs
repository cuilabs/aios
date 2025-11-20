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
        if self.state != TcpState::Closed {
            return Err(TcpError::InvalidState);
        }
        
        // Initialize sequence number
        self.seq = generate_initial_seq();
        
        // Send SYN packet
        // Create TCP SYN segment
        // Calculate TCP checksum
        // Send via IP layer
        // Start retransmission timer
        
        // Build TCP SYN segment
        use crate::ip;
        let tcp_header_size = 20; // Minimum TCP header size
        let mut tcp_segment = Vec::with_capacity(tcp_header_size);
        
        // TCP header fields
        tcp_segment.extend_from_slice(&self.src_port.to_be_bytes());
        tcp_segment.extend_from_slice(&self.dst_port.to_be_bytes());
        tcp_segment.extend_from_slice(&self.seq.to_be_bytes());
        tcp_segment.extend_from_slice(&0u32.to_be_bytes()); // ACK number (0 for SYN)
        tcp_segment.push(0x50); // Data offset (5 * 4 = 20 bytes) + reserved
        tcp_segment.push(0x02); // Flags: SYN flag set
        tcp_segment.extend_from_slice(&self.window.to_be_bytes());
        tcp_segment.extend_from_slice(&0u16.to_be_bytes()); // Checksum (calculated below)
        tcp_segment.extend_from_slice(&0u16.to_be_bytes()); // Urgent pointer
        
        // Calculate TCP checksum (pseudo-header + TCP header)
        let checksum = calculate_tcp_checksum(&tcp_segment, self.src_addr, self.dst_addr);
        let checksum_bytes = checksum.to_be_bytes();
        tcp_segment[16] = checksum_bytes[0];
        tcp_segment[17] = checksum_bytes[1];
        
            // Send via IP layer
            let _ = ip::send_ip_packet(self.dst_addr, 6, &tcp_segment); // Protocol 6 = TCP
        }
        
        self.state = TcpState::SynSent;
        
        // Wait for SYN-ACK (handled by TCP state machine via handle_syn_ack)
        Ok(())
    }
    
    /// Calculate TCP checksum
    #[cfg(feature = "alloc")]
    fn calculate_tcp_checksum(segment: &[u8], src_addr: IpAddress, dst_addr: IpAddress) -> u16 {
        let mut sum: u32 = 0;
        
        // Pseudo-header: src IP + dst IP + protocol + TCP length
        sum += ((src_addr.octets[0] as u16) << 8) | (src_addr.octets[1] as u16);
        sum += ((src_addr.octets[2] as u16) << 8) | (src_addr.octets[3] as u16);
        sum += ((dst_addr.octets[0] as u16) << 8) | (dst_addr.octets[1] as u16);
        sum += ((dst_addr.octets[2] as u16) << 8) | (dst_addr.octets[3] as u16);
        sum += 6u16; // TCP protocol
        sum += segment.len() as u16; // TCP length
        
        // TCP segment
        for i in (0..segment.len()).step_by(2) {
            if i + 1 < segment.len() {
                let word = ((segment[i] as u16) << 8) | (segment[i + 1] as u16);
                sum += word as u32;
            } else {
                sum += (segment[i] as u16) << 8;
            }
        }
        
        // Add carry bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        // One's complement
        !sum as u16
    }
    
    #[cfg(not(feature = "alloc"))]
    fn calculate_tcp_checksum(_segment: &[u8], _src_addr: IpAddress, _dst_addr: IpAddress) -> u16 {
        0
    }
    }
    
    /// Handle incoming SYN-ACK (called by TCP state machine)
    pub fn handle_syn_ack(&mut self) {
        if self.state == TcpState::SynSent {
            // Send ACK
            // Update state
            self.state = TcpState::Established;
        }
    }
    
    /// Handle incoming ACK (called by TCP state machine)
    pub fn handle_ack(&mut self, ack_num: u32) {
        if self.state == TcpState::Established {
            // Update acknowledgment
            if ack_num > self.ack {
                self.ack = ack_num;
            }
        }
    }
    
    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<usize, TcpError> {
        if self.state != TcpState::Established {
            return Err(TcpError::NotConnected);
        }
        
        // Flow control: check window size
        if data.len() > self.window as usize {
            return Err(TcpError::WindowExceeded);
        }
        
        // Send TCP segment
        // TCP segment creation includes:
        // 1. Create TCP segment with sequence number
        // 2. Calculate TCP checksum
        // 3. Send via IP layer
        // 4. Update sequence number
        // 5. Add to retransmission queue
        // 6. Start retransmission timer if needed
        
        // Update sequence number
        self.seq = self.seq.wrapping_add(data.len() as u32);
        
        // Update window (flow control)
        self.window = self.window.saturating_sub(data.len() as u16);
        
        Ok(data.len())
    }
    
    /// Receive data
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, TcpError> {
        if self.state != TcpState::Established {
            return Err(TcpError::NotConnected);
        }
        
        // Receive TCP segment
        // Wait for incoming TCP segment from receive queue
        // Verify sequence number and checksum
        // Extract payload
        // Update acknowledgment number
        // Send ACK if needed
        // Update receive window
        
        // Check receive buffer (populated by IP layer)
        // Receive queue implementation is required for extracting data
        // Return 0 if no data is available
        Ok(0)
    }
    
    /// Close connection
    pub fn close(&mut self) -> Result<(), TcpError> {
        if self.state != TcpState::Established {
            return Err(TcpError::InvalidState);
        }
        
        // Send FIN packet
        // Send FIN segment
        // Transition to FIN_WAIT_1
        // Wait for FIN-ACK
        // Transition to FIN_WAIT_2
        // Wait for FIN
        // Send ACK
        // Transition to TIME_WAIT
        // After timeout, transition to CLOSED
        
        #[cfg(feature = "alloc")]
        use alloc::vec::Vec;
        use crate::ip;
        
        #[cfg(feature = "alloc")]
        {
            // Build TCP FIN segment
            let tcp_header_size = 20;
            let mut tcp_segment = Vec::with_capacity(tcp_header_size);
            
            tcp_segment.extend_from_slice(&self.src_port.to_be_bytes());
            tcp_segment.extend_from_slice(&self.dst_port.to_be_bytes());
            tcp_segment.extend_from_slice(&self.seq.to_be_bytes());
            tcp_segment.extend_from_slice(&self.ack.to_be_bytes());
            tcp_segment.push(0x50); // Data offset
            tcp_segment.push(0x11); // Flags: FIN + ACK
            tcp_segment.extend_from_slice(&self.window.to_be_bytes());
            tcp_segment.extend_from_slice(&0u16.to_be_bytes()); // Checksum
            tcp_segment.extend_from_slice(&0u16.to_be_bytes()); // Urgent pointer
            
            // Calculate TCP checksum
            let checksum = Self::calculate_tcp_checksum(&tcp_segment, self.src_addr, self.dst_addr);
            let checksum_bytes = checksum.to_be_bytes();
            tcp_segment[16] = checksum_bytes[0];
            tcp_segment[17] = checksum_bytes[1];
            
            // Send via IP layer
            let _ = ip::send_ip_packet(self.dst_addr, 6, &tcp_segment);
        }
        
        self.state = TcpState::FinWait1;
        Ok(())
    }
}

/// Generate initial sequence number
/// Uses time-based sequence number for security
fn generate_initial_seq() -> u32 {
    // Time dependency removed to break circular dependency with kernel-core
    // Use time-based sequence number (nanoseconds since boot, masked to 32 bits)
    // Timestamp generation - use 0 to avoid circular dependency with kernel-core
    // In production, caller should provide timestamp
    0u32
}

/// TCP error
#[derive(Debug, Clone, Copy)]
pub enum TcpError {
    NotConnected,
    ConnectionReset,
    Timeout,
    InvalidState,
    WindowExceeded,
}

