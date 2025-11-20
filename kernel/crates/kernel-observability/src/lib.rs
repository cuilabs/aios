#![no_std]

//! Performance Counters / Observability Hooks
//! 
//! CPU, memory, network, FS, scheduler metrics; trace agent behavior

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod counters;
pub mod metrics;
pub mod tracing;

use counters::PerformanceCounters;
use metrics::MetricsCollector;
use tracing::TracingSystem;

/// Observability system
pub struct ObservabilitySystem {
    counters: spin::Mutex<PerformanceCounters>,
    metrics: spin::Mutex<MetricsCollector>,
    tracing: spin::Mutex<TracingSystem>,
}

/// Global observability system instance
static OBSERVABILITY_SYSTEM: spin::Once<ObservabilitySystem> = spin::Once::new();

impl ObservabilitySystem {
    pub fn new() -> Self {
        Self {
            counters: spin::Mutex::new(PerformanceCounters::new()),
            metrics: spin::Mutex::new(MetricsCollector::new()),
            tracing: spin::Mutex::new(TracingSystem::new()),
            #[cfg(feature = "alloc")]
            event_subscription_id: spin::Mutex::new(None),
            #[cfg(not(feature = "alloc"))]
            event_subscription_id: spin::Mutex::new(()),
        }
    }

    /// Initialize global observability system
    pub fn init() {
        OBSERVABILITY_SYSTEM.call_once(|| ObservabilitySystem::new());
    }

    /// Get global observability system instance
    pub fn get() -> Option<&'static ObservabilitySystem> {
        OBSERVABILITY_SYSTEM.get()
    }

    /// Get performance counters for agent
    pub fn get_agent_counters(&self, agent_id: u64) -> Option<AgentCounters> {
        let counters = self.counters.lock();
        counters.get_agent_counters(agent_id)
    }

    /// Collect metrics
    pub fn collect_metrics(&self) -> SystemMetrics {
        let metrics = self.metrics.lock();
        metrics.collect()
    }

    /// Start tracing
    pub fn start_trace(&self, agent_id: u64) {
        let mut tracing = self.tracing.lock();
        tracing.start_trace(agent_id);
    }

    /// Stop tracing
    pub fn stop_trace(&self, agent_id: u64) -> TraceData {
        let mut tracing = self.tracing.lock();
        tracing.stop_trace(agent_id)
    }

    /// Subscribe to event bus for automatic metrics collection
    #[cfg(feature = "alloc")]
    pub fn subscribe_to_events(&self) {
        use aios_kernel_events::{event::EventType, subscribe, KernelEventSystem};
        use aios_kernel_events::event::KernelEvent;
        use aios_kernel_events::subscription::EventSubscription;
        
        // Subscribe to all event types for metrics collection
        let event_types = alloc::vec![
            EventType::AgentLifecycle,
            EventType::Network,
            EventType::VirtualMemory,
            EventType::FileSystem,
            EventType::IO,
            EventType::Performance,
        ];
        
        if let Some(system) = KernelEventSystem::get() {
            let subscription = EventSubscription::new(
                event_types,
                None,
                |event: &KernelEvent| {
                    // Update metrics based on event
                    if let Some(agent_id) = event.agent_id {
                        match event.event_type {
                            EventType::Network => {
                                // Network events update network metrics
                                // Parse event data for bytes sent/received
                                if !event.data.is_empty() {
                                    // Event data format: [bytes_sent(8)][bytes_received(8)]
                                    if event.data.len() >= 16 {
                                        let bytes_sent = u64::from_le_bytes([
                                            event.data[0], event.data[1], event.data[2], event.data[3],
                                            event.data[4], event.data[5], event.data[6], event.data[7],
                                        ]);
                                        let bytes_received = u64::from_le_bytes([
                                            event.data[8], event.data[9], event.data[10], event.data[11],
                                            event.data[12], event.data[13], event.data[14], event.data[15],
                                        ]);
                                        update_network_bytes(agent_id, bytes_sent, bytes_received);
                                    }
                                }
                            }
                            EventType::VirtualMemory => {
                                // Memory events update memory metrics
                            }
                            EventType::IO => {
                                // I/O events update I/O metrics
                                update_io_operations(agent_id);
                            }
                            _ => {}
                        }
                    }
                },
            );
            
            let subscription_id = system.subscribe(subscription);
            let mut sub_id = self.event_subscription_id.lock();
            *sub_id = Some(subscription_id);
        }
    }

    /// Stream events to userland (for external monitoring)
    #[cfg(feature = "alloc")]
    pub fn stream_events_to_userland(&self, callback: fn(&KernelEvent)) -> u64 {
        use aios_kernel_events::{event::EventType, KernelEventSystem};
        use aios_kernel_events::event::KernelEvent;
        use aios_kernel_events::subscription::EventSubscription;
        
        // Subscribe to all events and forward to userland callback
        let event_types = alloc::vec![
            EventType::Interrupt,
            EventType::Device,
            EventType::AgentLifecycle,
            EventType::FileSystem,
            EventType::Network,
            EventType::VirtualMemory,
            EventType::Security,
            EventType::Performance,
            EventType::IO,
        ];
        
        if let Some(system) = KernelEventSystem::get() {
            let subscription = EventSubscription::new(event_types, None, callback);
            system.subscribe(subscription)
        } else {
            0
        }
    }

    /// Get agent behavior profile
    #[cfg(feature = "alloc")]
    pub fn get_agent_profile(&self, agent_id: u64) -> Option<AgentProfile> {
        use aios_kernel_hal::timer;
        
        let counters = self.counters.lock();
        let agent_counters = counters.get_agent_counters(agent_id)?;
        
        let tracing = self.tracing.lock();
        let trace_count = tracing.trace_event_count(agent_id);
        
        Some(AgentProfile {
            agent_id,
            cpu_cycles: agent_counters.cpu_cycles,
            memory_allocated: agent_counters.memory_allocated,
            network_bytes: agent_counters.network_bytes_sent + agent_counters.network_bytes_received,
            io_operations: agent_counters.io_operations,
            trace_events: trace_count,
            last_updated: timer::now_nanos() / 1_000_000_000, // Convert to seconds
        })
    }
}

/// Agent behavior profile
#[cfg(feature = "alloc")]
pub struct AgentProfile {
    pub agent_id: u64,
    pub cpu_cycles: u64,
    pub memory_allocated: u64,
    pub network_bytes: u64,
    pub io_operations: u64,
    pub trace_events: usize,
    pub last_updated: u64,
}

/// Agent performance counters
pub struct AgentCounters {
    pub cpu_cycles: u64,
    pub instructions: u64,
    pub cache_misses: u64,
    pub memory_allocated: u64,
    pub memory_freed: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub io_operations: u64,
}

/// System metrics
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_throughput: f64,
    pub io_throughput: f64,
    pub active_agents: u32,
}

/// Trace data
pub struct TraceData {
    pub agent_id: u64,
    pub events: alloc::vec::Vec<TraceEvent>,
}

/// Trace event
pub struct TraceEvent {
    pub timestamp: u64,
    pub event_type: alloc::string::String,
    pub data: alloc::vec::Vec<u8>,
}

/// Update memory allocated counter
#[cfg(feature = "alloc")]
pub fn update_memory_allocated(agent_id: u64, bytes: u64) {
    if let Some(system) = ObservabilitySystem::get() {
        let counters = system.counters.lock();
        counters.update_memory_allocated(agent_id, bytes);
    }
}

/// Update network bytes counter
#[cfg(feature = "alloc")]
pub fn update_network_bytes(agent_id: u64, sent: u64, received: u64) {
    if let Some(system) = ObservabilitySystem::get() {
        let counters = system.counters.lock();
        counters.update_network_bytes(agent_id, sent, received);
    }
}

/// Update I/O operations counter
#[cfg(feature = "alloc")]
pub fn update_io_operations(agent_id: u64) {
    if let Some(system) = ObservabilitySystem::get() {
        let counters = system.counters.lock();
        counters.update_io_operations(agent_id);
    }
}

