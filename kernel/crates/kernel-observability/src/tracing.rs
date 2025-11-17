//! Tracing system

use super::{TraceData, TraceEvent};
use spin::Mutex;

#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;

    /// Tracing system
    pub struct TracingSystem {
        active_traces: Mutex<BTreeMap<u64, Vec<TraceEvent>>>,
    }

    impl TracingSystem {
        pub fn new() -> Self {
            Self {
                active_traces: Mutex::new(BTreeMap::new()),
            }
        }

        pub fn start_trace(&mut self, agent_id: u64) {
            let mut traces = self.active_traces.lock();
            traces.insert(agent_id, Vec::new());
        }

        pub fn stop_trace(&mut self, agent_id: u64) -> TraceData {
            let mut traces = self.active_traces.lock();
            let events = traces.remove(&agent_id).unwrap_or_default();

            TraceData { agent_id, events }
        }

        pub fn record_event(&self, agent_id: u64, event: TraceEvent) {
            let mut traces = self.active_traces.lock();
            if let Some(events) = traces.get_mut(&agent_id) {
                events.push(event);
            }
        }
    }
}

#[cfg(feature = "alloc")]
use alloc_impl::TracingSystem;

#[cfg(not(feature = "alloc"))]
struct TracingSystem;

#[cfg(not(feature = "alloc"))]
impl TracingSystem {
    pub fn new() -> Self {
        Self
    }
    pub fn start_trace(&mut self, _agent_id: u64) {}
    pub fn stop_trace(&mut self, agent_id: u64) -> TraceData {
        #[cfg(feature = "alloc")]
        let events = alloc::vec::Vec::new();
        #[cfg(not(feature = "alloc"))]
        let events = [];
        TraceData {
            agent_id,
            events,
        }
    }
    pub fn record_event(&self, _agent_id: u64, _event: TraceEvent) {}
}

