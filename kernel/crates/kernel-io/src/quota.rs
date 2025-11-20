//! I/O Quota Management
//! 
//! Per-agent I/O quotas and limits

use spin::Mutex;

/// I/O Quota
pub struct IOQuota {
    agent_id: u64,
    read_bytes: Mutex<u64>,
    write_bytes: Mutex<u64>,
    read_ops: Mutex<u64>,
    write_ops: Mutex<u64>,
    limit_bytes: u64,
    limit_ops: u64,
}

impl IOQuota {
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            read_bytes: Mutex::new(0),
            write_bytes: Mutex::new(0),
            read_ops: Mutex::new(0),
            write_ops: Mutex::new(0),
            limit_bytes: 1024 * 1024 * 1024, // 1GB default
            limit_ops: 10000, // 10K ops default
        }
    }
    
    /// Check if operation is within quota
    pub fn check_limit(&self, size: u64) -> bool {
        let read_bytes = self.read_bytes.lock();
        let write_bytes = self.write_bytes.lock();
        let read_ops = self.read_ops.lock();
        let write_ops = self.write_ops.lock();
        
        let total_bytes = *read_bytes + *write_bytes;
        let total_ops = *read_ops + *write_ops;
        
        total_bytes + size <= self.limit_bytes && total_ops < self.limit_ops
    }
    
    /// Consume quota
    pub fn consume(&self, size: u64) {
        let mut read_bytes = self.read_bytes.lock();
        *read_bytes += size;
        let mut read_ops = self.read_ops.lock();
        *read_ops += 1;
    }
    
    /// Get current usage
    pub fn get_usage(&self) -> (u64, u64) {
        let read_bytes = self.read_bytes.lock();
        let write_bytes = self.write_bytes.lock();
        (*read_bytes + *write_bytes, *read_bytes + *write_bytes)
    }
}

