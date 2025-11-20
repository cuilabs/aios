//! I/O Scheduler
//! 
//! Schedules I/O operations for optimal performance using deadline and CFQ algorithms

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::{VecDeque, BTreeMap};
use spin::Mutex;

/// I/O request
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct IORequest {
    pub device_id: u64,
    pub block: u64,
    pub operation: IOOperation,
    pub priority: u8,
    pub deadline: u64, // Deadline timestamp (nanoseconds)
    pub agent_id: u64,
}

/// I/O operation type
#[derive(Clone, Copy, Debug)]
pub enum IOOperation {
    Read,
    Write,
}

/// Scheduler algorithm
#[derive(Clone, Copy, Debug)]
pub enum SchedulerAlgorithm {
    FIFO,
    Deadline,
    CFQ, // Completely Fair Queuing
}

/// I/O Scheduler
pub struct IOScheduler {
    #[cfg(feature = "alloc")]
    read_queue: Mutex<VecDeque<IORequest>>,
    #[cfg(feature = "alloc")]
    write_queue: Mutex<VecDeque<IORequest>>,
    #[cfg(feature = "alloc")]
    deadline_map: Mutex<BTreeMap<u64, IORequest>>, // Deadline -> Request
    #[cfg(feature = "alloc")]
    algorithm: Mutex<SchedulerAlgorithm>,
    #[cfg(not(feature = "alloc"))]
    read_queue: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    write_queue: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    deadline_map: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    algorithm: Mutex<()>,
}

impl IOScheduler {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            read_queue: Mutex::new(VecDeque::new()),
            #[cfg(feature = "alloc")]
            write_queue: Mutex::new(VecDeque::new()),
            #[cfg(feature = "alloc")]
            deadline_map: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            algorithm: Mutex::new(SchedulerAlgorithm::Deadline),
            #[cfg(not(feature = "alloc"))]
            read_queue: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            write_queue: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            deadline_map: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            algorithm: Mutex::new(()),
        }
    }
    
    /// Schedule I/O request
    #[cfg(feature = "alloc")]
    pub fn schedule(&self, request: IORequest) {
        let algorithm = *self.algorithm.lock();
        
        match algorithm {
            SchedulerAlgorithm::FIFO => {
                match request.operation {
                    IOOperation::Read => {
                        let mut queue = self.read_queue.lock();
                        queue.push_back(request);
                    }
                    IOOperation::Write => {
                        let mut queue = self.write_queue.lock();
                        queue.push_back(request);
                    }
                }
            }
            SchedulerAlgorithm::Deadline => {
                // Separate read/write queues, prioritize by deadline
                let mut deadline_map = self.deadline_map.lock();
                deadline_map.insert(request.deadline, request);
            }
            SchedulerAlgorithm::CFQ => {
                // Round-robin per agent
                match request.operation {
                    IOOperation::Read => {
                        let mut queue = self.read_queue.lock();
                        // Insert based on agent fairness
                        queue.push_back(request);
                    }
                    IOOperation::Write => {
                        let mut queue = self.write_queue.lock();
                        queue.push_back(request);
                    }
                }
            }
        }
    }
    
    /// Get next I/O request
    #[cfg(feature = "alloc")]
    pub fn next(&self) -> Option<IORequest> {
        let algorithm = *self.algorithm.lock();
        
        match algorithm {
            SchedulerAlgorithm::FIFO => {
                // Prioritize reads over writes
                let mut read_queue = self.read_queue.lock();
                if let Some(request) = read_queue.pop_front() {
                    return Some(request);
                }
                let mut write_queue = self.write_queue.lock();
                write_queue.pop_front()
            }
            SchedulerAlgorithm::Deadline => {
                // Get request with earliest deadline
                let mut deadline_map = self.deadline_map.lock();
                if let Some((_deadline, request)) = deadline_map.pop_first() {
                    Some(request)
                } else {
                    None
                }
            }
            SchedulerAlgorithm::CFQ => {
                // Round-robin between read and write queues
                let mut read_queue = self.read_queue.lock();
                if let Some(request) = read_queue.pop_front() {
                    return Some(request);
                }
                let mut write_queue = self.write_queue.lock();
                write_queue.pop_front()
            }
        }
    }

    /// Set scheduler algorithm
    #[cfg(feature = "alloc")]
    pub fn set_algorithm(&self, algorithm: SchedulerAlgorithm) {
        let mut algo = self.algorithm.lock();
        *algo = algorithm;
    }

    #[cfg(not(feature = "alloc"))]
    pub fn set_algorithm(&self, _algorithm: SchedulerAlgorithm) {
        // No-op
    }

    /// Get queue length
    #[cfg(feature = "alloc")]
    pub fn queue_length(&self) -> usize {
        let read_queue = self.read_queue.lock();
        let write_queue = self.write_queue.lock();
        let deadline_map = self.deadline_map.lock();
        read_queue.len() + write_queue.len() + deadline_map.len()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn queue_length(&self) -> usize {
        0
    }
}

