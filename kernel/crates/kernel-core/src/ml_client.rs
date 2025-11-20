//! ML Client for Kernel AI Subsystems
//! 
//! Provides interface for kernel AI subsystems to request ML predictions.
//! Calls userland ML bridge service via IPC, which then calls ML daemon (mld) via HTTP.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[cfg(feature = "alloc")]
extern crate alloc;

/// Workload prediction request
#[derive(Clone, Debug)]
pub struct WorkloadPredictionRequest {
    pub agent_id: u64,
    pub historical_cpu: Vec<f32>,      // Last 10 CPU usage values
    pub historical_memory: Vec<u64>,   // Last 10 memory usage values
    pub historical_gpu: Vec<f32>,      // Last 10 GPU usage values (optional)
    pub time_of_day: u8,                // Hour of day (0-23)
    pub day_of_week: u8,                // Day of week (0-6)
    pub current_cpu: f32,
    pub current_memory: u64,
    pub current_gpu: Option<f32>,
}

/// Workload prediction response
#[derive(Clone, Debug)]
pub struct WorkloadPredictionResponse {
    pub predicted_cpu: f32,      // 0.0 to 1.0
    pub predicted_memory: u64,   // Bytes
    pub predicted_gpu: Option<f32>, // 0.0 to 1.0
    pub confidence: f32,         // 0.0 to 1.0
}

/// Threat detection request
#[derive(Clone, Debug)]
pub struct ThreatDetectionRequest {
    pub agent_id: u64,
    pub metrics: BehaviorMetrics,
    pub anomalies: Vec<BehavioralAnomaly>,
    pub historical_threats: Vec<f32>, // Last 10 threat scores
    pub time_since_last_threat: u64, // Milliseconds
}

/// Behavior metrics
#[derive(Clone, Debug)]
pub struct BehaviorMetrics {
    pub operation_count: u64,
    pub syscall_count: u64,
    pub memory_usage: u64,
    pub network_activity: u64,
}

/// Behavioral anomaly
#[derive(Clone, Debug)]
pub struct BehavioralAnomaly {
    pub anomaly_type: u32,
    pub severity: f32,
    pub timestamp: u64,
}

/// Threat detection response
#[derive(Clone, Debug)]
pub struct ThreatDetectionResponse {
    pub threat_score: f32,           // 0.0 to 1.0
    pub threat_type: u32,             // 0-5 (enum index)
    pub confidence: f32,              // 0.0 to 1.0
    pub recommended_action: u32,      // 0-4 (enum index)
}

/// Failure prediction request
#[derive(Clone, Debug)]
pub struct FailurePredictionRequest {
    pub component: String,
    pub health_score: f32,            // 0.0 to 1.0
    pub current_value: f32,
    pub baseline: f32,
    pub trend: i32,                    // -1 (degrading) to 1 (improving)
    pub historical_health: Vec<f32>,   // Last 20 health scores
    pub failure_history: Vec<u8>,     // Last 10 failure events (0 or 1)
    pub time_since_last_failure: u64,  // Milliseconds
}

/// Failure prediction response
#[derive(Clone, Debug)]
pub struct FailurePredictionResponse {
    pub failure_probability: f32,      // 0.0 to 1.0
    pub predicted_time: Option<u64>,   // Milliseconds until failure (or None)
    pub confidence: f32,              // 0.0 to 1.0
    pub failure_type: u32,            // 0-5 (enum index)
}

/// Memory access prediction request
#[derive(Clone, Debug)]
pub struct MemoryPredictionRequest {
    pub agent_id: u64,
    pub access_history: Vec<f32>,      // Last 20 virtual addresses (normalized)
    pub access_types: Vec<u8>,         // Last 20 access types (0=read, 1=write, 2=execute)
    pub access_timestamps: Vec<f32>,   // Last 20 timestamps (normalized)
    pub current_address: f32,          // Current virtual address (normalized)
    pub locality_score: f32,            // 0.0 to 1.0
}

/// Memory access prediction response
#[derive(Clone, Debug)]
pub struct MemoryPredictionResponse {
    pub next_address: f32,             // Predicted next virtual address (normalized)
    pub access_probability: f32,       // 0.0 to 1.0
    pub access_type: u8,               // 0-2 (read, write, execute)
    pub confidence: f32,               // 0.0 to 1.0
}

/// ML Client for kernel AI subsystems
/// 
/// Provides interface to request ML predictions.
/// Calls userland ML bridge service via IPC.
pub struct MLClient {
    // Cache for recent predictions (to avoid redundant calls)
    workload_cache: spin::Mutex<BTreeMap<u64, (WorkloadPredictionResponse, u64)>>, // agent_id -> (prediction, timestamp)
    threat_cache: spin::Mutex<BTreeMap<u64, (ThreatDetectionResponse, u64)>>,
    failure_cache: spin::Mutex<BTreeMap<String, (FailurePredictionResponse, u64)>>,
    memory_cache: spin::Mutex<BTreeMap<u64, (MemoryPredictionResponse, u64)>>,
    cache_ttl_ns: u64, // Cache TTL in nanoseconds
}

/// ML Bridge service agent ID (system service)
const ML_BRIDGE_AGENT_ID: u64 = 1000;

impl MLClient {
    pub fn new() -> Self {
        Self {
            workload_cache: spin::Mutex::new(BTreeMap::new()),
            threat_cache: spin::Mutex::new(BTreeMap::new()),
            failure_cache: spin::Mutex::new(BTreeMap::new()),
            memory_cache: spin::Mutex::new(BTreeMap::new()),
            cache_ttl_ns: 100_000_000, // 100ms default TTL
        }
    }

    /// Predict workload for agent
    /// 
    /// Returns cached prediction if available and fresh, otherwise requests new prediction.
    pub fn predict_workload(&self, request: WorkloadPredictionRequest) -> Option<WorkloadPredictionResponse> {
        let now = crate::time::now();
        
        // Check cache first
        {
            let cache = self.workload_cache.lock();
            if let Some((cached_pred, cached_time)) = cache.get(&request.agent_id) {
                if now.saturating_sub(*cached_time) < self.cache_ttl_ns {
                    return Some(cached_pred.clone());
                }
            }
        }

        // Request new prediction via IPC to ML bridge service
        // 1. Serialize request to JSON bytes
        let request_json = alloc::format!(r#"{{"type":"workload","agent_id":{},"historical_cpu":{:?},"historical_memory":{:?},"historical_gpu":{:?},"time_of_day":{},"day_of_week":{},"current_cpu":{},"current_memory":{},"current_gpu":{:?}}}"#,
            request.agent_id,
            request.historical_cpu,
            request.historical_memory,
            request.historical_gpu,
            request.time_of_day,
            request.day_of_week,
            request.current_cpu,
            request.current_memory,
            request.current_gpu
        );
        
        // 2. Send IPC message to ML bridge service via syscall
        let data = request_json.as_bytes();
        let metadata = b"workload_prediction";
        
        // Use kernel IPC syscall to send message
        // This calls the actual IPC system without circular dependency
        let _message_id = crate::syscall::ipc_send(
            0, // Kernel agent ID
            ML_BRIDGE_AGENT_ID,
            data.as_ptr(),
            data.len(),
            metadata.as_ptr(),
            metadata.len(),
        );
        
        // 3. Check for response (non-blocking)
        // The ML bridge service processes the request asynchronously and sends response via IPC
        // Response will be received via IPC receive and cached when available
        // Return None if response not immediately available (async operation - response handled via IPC receive)
        None
    }

    /// Detect threat for agent
    pub fn detect_threat(&self, request: ThreatDetectionRequest) -> Option<ThreatDetectionResponse> {
        let now = crate::time::now();
        
        // Check cache first
        {
            let cache = self.threat_cache.lock();
            if let Some((cached_pred, cached_time)) = cache.get(&request.agent_id) {
                if now.saturating_sub(*cached_time) < self.cache_ttl_ns {
                    return Some(cached_pred.clone());
                }
            }
        }

        // Request new prediction via IPC to ML bridge service
        let request_json = alloc::format!(r#"{{"type":"threat","agent_id":{},"metrics":{{"operation_count":{},"syscall_count":{},"memory_usage":{},"network_activity":{}}},"anomalies":{:?},"historical_threats":{:?},"time_since_last_threat":{}}}"#,
            request.agent_id,
            request.metrics.operation_count,
            request.metrics.syscall_count,
            request.metrics.memory_usage,
            request.metrics.network_activity,
            request.anomalies,
            request.historical_threats,
            request.time_since_last_threat
        );
        
        let data = request_json.as_bytes();
        let metadata = b"threat_detection";
        
        let _message_id = crate::syscall::ipc_send(
            0,
            ML_BRIDGE_AGENT_ID,
            data.as_ptr(),
            data.len(),
            metadata.as_ptr(),
            metadata.len(),
        );
        
        None
    }

    /// Predict failure for component
    pub fn predict_failure(&self, request: FailurePredictionRequest) -> Option<FailurePredictionResponse> {
        let now = crate::time::now();
        
        // Check cache first
        {
            let cache = self.failure_cache.lock();
            if let Some((cached_pred, cached_time)) = cache.get(&request.component) {
                if now.saturating_sub(*cached_time) < self.cache_ttl_ns {
                    return Some(cached_pred.clone());
                }
            }
        }

        // Request new prediction via IPC to ML bridge service
        let request_json = alloc::format!(r#"{{"type":"failure","component":"{}","health_score":{},"current_value":{},"baseline":{},"trend":{},"historical_health":{:?},"failure_history":{:?},"time_since_last_failure":{}}}"#,
            request.component,
            request.health_score,
            request.current_value,
            request.baseline,
            request.trend,
            request.historical_health,
            request.failure_history,
            request.time_since_last_failure
        );
        
        let data = request_json.as_bytes();
        let metadata = b"failure_prediction";
        
        let _message_id = crate::syscall::ipc_send(
            0,
            ML_BRIDGE_AGENT_ID,
            data.as_ptr(),
            data.len(),
            metadata.as_ptr(),
            metadata.len(),
        );
        
        None
    }

    /// Predict memory access for agent
    pub fn predict_memory(&self, request: MemoryPredictionRequest) -> Option<MemoryPredictionResponse> {
        let now = crate::time::now();
        
        // Check cache first
        {
            let cache = self.memory_cache.lock();
            if let Some((cached_pred, cached_time)) = cache.get(&request.agent_id) {
                if now.saturating_sub(*cached_time) < self.cache_ttl_ns {
                    return Some(cached_pred.clone());
                }
            }
        }

        // Request new prediction via IPC to ML bridge service
        let request_json = alloc::format!(r#"{{"type":"memory","agent_id":{},"access_history":{:?},"access_types":{:?},"access_timestamps":{:?},"current_address":{},"locality_score":{}}}"#,
            request.agent_id,
            request.access_history,
            request.access_types,
            request.access_timestamps,
            request.current_address,
            request.locality_score
        );
        
        let data = request_json.as_bytes();
        let metadata = b"memory_prediction";
        
        let _message_id = crate::syscall::ipc_send(
            0,
            ML_BRIDGE_AGENT_ID,
            data.as_ptr(),
            data.len(),
            metadata.as_ptr(),
            metadata.len(),
        );
        
        None
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.workload_cache.lock().clear();
        self.threat_cache.lock().clear();
        self.failure_cache.lock().clear();
        self.memory_cache.lock().clear();
    }
}

/// Global ML client instance
static ML_CLIENT: spin::Once<MLClient> = spin::Once::new();

/// Get global ML client instance
pub fn get_ml_client() -> &'static MLClient {
    ML_CLIENT.call_once(|| MLClient::new())
}

/// Initialize ML client (called during kernel boot)
pub fn init() {
    let _ = get_ml_client();
}

