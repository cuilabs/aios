//! Minimal serialization for kernel
//! 
//! No external dependencies - pure kernel serialization
//! For IPC messages and kernel data structures

use core::fmt::Write;

/// Serialize a u64 to bytes (little-endian)
pub fn serialize_u64(value: u64, buf: &mut [u8]) -> usize {
    if buf.len() < 8 {
        return 0;
    }
    buf[0] = value as u8;
    buf[1] = (value >> 8) as u8;
    buf[2] = (value >> 16) as u8;
    buf[3] = (value >> 24) as u8;
    buf[4] = (value >> 32) as u8;
    buf[5] = (value >> 40) as u8;
    buf[6] = (value >> 48) as u8;
    buf[7] = (value >> 56) as u8;
    8
}

/// Deserialize a u64 from bytes (little-endian)
pub fn deserialize_u64(buf: &[u8]) -> Option<u64> {
    if buf.len() < 8 {
        return None;
    }
    Some(
        (buf[0] as u64) |
        ((buf[1] as u64) << 8) |
        ((buf[2] as u64) << 16) |
        ((buf[3] as u64) << 24) |
        ((buf[4] as u64) << 32) |
        ((buf[5] as u64) << 40) |
        ((buf[6] as u64) << 48) |
        ((buf[7] as u64) << 56)
    )
}

/// Serialize a u32 to bytes (little-endian)
pub fn serialize_u32(value: u32, buf: &mut [u8]) -> usize {
    if buf.len() < 4 {
        return 0;
    }
    buf[0] = value as u8;
    buf[1] = (value >> 8) as u8;
    buf[2] = (value >> 16) as u8;
    buf[3] = (value >> 24) as u8;
    4
}

/// Deserialize a u32 from bytes (little-endian)
pub fn deserialize_u32(buf: &[u8]) -> Option<u32> {
    if buf.len() < 4 {
        return None;
    }
    Some(
        (buf[0] as u32) |
        ((buf[1] as u32) << 8) |
        ((buf[2] as u32) << 16) |
        ((buf[3] as u32) << 24)
    )
}

/// Serialize a string to bytes (length-prefixed)
#[cfg(feature = "alloc")]
pub fn serialize_string(value: &str, buf: &mut [u8]) -> Option<usize> {
    use alloc::vec::Vec;
    let bytes = value.as_bytes();
    let len = bytes.len();
    
    if buf.len() < 4 + len {
        return None;
    }
    
    // Write length
    let written = serialize_u32(len as u32, buf);
    if written == 0 {
        return None;
    }
    
    // Write string bytes
    buf[4..4 + len].copy_from_slice(bytes);
    
    Some(4 + len)
}

/// Deserialize a string from bytes (length-prefixed)
#[cfg(feature = "alloc")]
pub fn deserialize_string(buf: &[u8]) -> Option<(alloc::string::String, usize)> {
    use alloc::string::String;
    
    if buf.len() < 4 {
        return None;
    }
    
    let len = deserialize_u32(&buf[0..4])? as usize;
    if buf.len() < 4 + len {
        return None;
    }
    
    let bytes = &buf[4..4 + len];
    let string = String::from_utf8(bytes.to_vec()).ok()?;
    
    Some((string, 4 + len))
}

/// Simple message format for IPC
#[repr(C)]
pub struct SimpleMessage {
    pub message_type: u32,
    pub sender_id: u64,
    pub receiver_id: u64,
    pub data_len: u32,
    pub data: *const u8,
}

/// Serialize a simple message to bytes
pub fn serialize_message(msg: &SimpleMessage, buf: &mut [u8]) -> Option<usize> {
    let mut offset = 0;
    
    // Serialize message_type
    if offset + 4 > buf.len() {
        return None;
    }
    offset += serialize_u32(msg.message_type, &mut buf[offset..]);
    
    // Serialize sender_id
    if offset + 8 > buf.len() {
        return None;
    }
    offset += serialize_u64(msg.sender_id, &mut buf[offset..]);
    
    // Serialize receiver_id
    if offset + 8 > buf.len() {
        return None;
    }
    offset += serialize_u64(msg.receiver_id, &mut buf[offset..]);
    
    // Serialize data_len
    if offset + 4 > buf.len() {
        return None;
    }
    offset += serialize_u32(msg.data_len, &mut buf[offset..]);
    
    // Copy data
    if offset + msg.data_len as usize > buf.len() {
        return None;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(
            msg.data,
            buf.as_mut_ptr().add(offset),
            msg.data_len as usize
        );
    }
    offset += msg.data_len as usize;
    
    Some(offset)
}

/// Deserialize a simple message from bytes
pub fn deserialize_message(buf: &[u8]) -> Option<(SimpleMessage, usize)> {
    let mut offset = 0;
    
    // Deserialize message_type
    let message_type = deserialize_u32(&buf[offset..])?;
    offset += 4;
    
    // Deserialize sender_id
    let sender_id = deserialize_u64(&buf[offset..])?;
    offset += 8;
    
    // Deserialize receiver_id
    let receiver_id = deserialize_u64(&buf[offset..])?;
    offset += 8;
    
    // Deserialize data_len
    let data_len = deserialize_u32(&buf[offset..])?;
    offset += 4;
    
    // Get data pointer
    if offset + data_len as usize > buf.len() {
        return None;
    }
    let data = buf.as_ptr().add(offset);
    offset += data_len as usize;
    
    Some((
        SimpleMessage {
            message_type,
            sender_id,
            receiver_id,
            data_len,
            data,
        },
        offset
    ))
}

