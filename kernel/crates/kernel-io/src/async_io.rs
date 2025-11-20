//! Async I/O support
//! 
//! Provides asynchronous I/O operations with completion callbacks

use crate::IOError;
use crate::IOOperation;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use spin::Mutex;

/// Async I/O request ID
pub type AsyncIORequestId = u64;

/// Async I/O completion callback
#[cfg(feature = "alloc")]
pub type AsyncIOCallback = fn(AsyncIORequestId, Result<Vec<u8>, IOError>);

/// Async I/O request
#[cfg(feature = "alloc")]
pub struct AsyncIORequest {
    pub request_id: AsyncIORequestId,
    pub device_id: u64,
    pub block: u64,
    pub operation: IOOperation,
    pub data: Option<Vec<u8>>, // For write operations
    pub callback: AsyncIOCallback,
    pub agent_id: u64,
}

/// Async I/O manager
pub struct AsyncIOManager {
    #[cfg(feature = "alloc")]
    pending_requests: Mutex<BTreeMap<AsyncIORequestId, AsyncIORequest>>,
    #[cfg(feature = "alloc")]
    next_request_id: Mutex<AsyncIORequestId>,
    #[cfg(not(feature = "alloc"))]
    pending_requests: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_request_id: Mutex<()>,
}

impl AsyncIOManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            pending_requests: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            next_request_id: Mutex::new(1),
            #[cfg(not(feature = "alloc"))]
            pending_requests: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            next_request_id: Mutex::new(()),
        }
    }

    /// Submit async I/O request
    #[cfg(feature = "alloc")]
    pub fn submit(&self, request: AsyncIORequest) -> AsyncIORequestId {
        let request_id = request.request_id;
        let mut pending = self.pending_requests.lock();
        pending.insert(request_id, request);
        request_id
    }

    #[cfg(not(feature = "alloc"))]
    pub fn submit(&self, _request: AsyncIORequest) -> AsyncIORequestId {
        0
    }

    /// Complete async I/O request
    #[cfg(feature = "alloc")]
    pub fn complete(&self, request_id: AsyncIORequestId, result: Result<Vec<u8>, IOError>) {
        let mut pending = self.pending_requests.lock();
        if let Some(request) = pending.remove(&request_id) {
            (request.callback)(request_id, result);
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn complete(&self, _request_id: AsyncIORequestId, _result: Result<Vec<u8>, IOError>) {
        // No-op
    }

    /// Generate next request ID
    #[cfg(feature = "alloc")]
    pub fn next_request_id(&self) -> AsyncIORequestId {
        let mut next_id = self.next_request_id.lock();
        let id = *next_id;
        *next_id += 1;
        id
    }

    #[cfg(not(feature = "alloc"))]
    pub fn next_request_id(&self) -> AsyncIORequestId {
        0
    }

    /// Get pending request count
    #[cfg(feature = "alloc")]
    pub fn pending_count(&self) -> usize {
        let pending = self.pending_requests.lock();
        pending.len()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn pending_count(&self) -> usize {
        0
    }
}

