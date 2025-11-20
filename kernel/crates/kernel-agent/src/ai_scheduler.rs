//! AI-Powered Resource Scheduler
//! 
//! Provides predictive workload analytics and intelligent resource allocation
//! for optimizing CPU, GPU, and memory usage based on agent behavior patterns.

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
extern crate alloc;

/// Workload snapshot for historical analysis
#[derive(Clone, Debug)]
pub struct WorkloadSnapshot {
    pub timestamp: u64,
    pub agent_id: u64,
    pub cpu_usage: f32,        // Percentage (0.0 - 1.0)
    pub memory_usage: u64,      // Bytes
    pub gpu_usage: Option<f32>, // Percentage (0.0 - 1.0), None if no GPU
    pub io_ops: u64,            // I/O operations per second
    pub network_ops: u64,       // Network operations per second
}

/// Agent workload pattern
#[derive(Clone, Debug)]
pub struct AgentPattern {
    pub agent_id: u64,
    pub peak_hours: Vec<u8>,    // Hours of day (0-23) when agent is most active
    pub typical_cpu: f32,       // Typical CPU usage
    pub typical_memory: u64,    // Typical memory usage
    pub burst_pattern: BurstPattern,
    pub access_pattern: AccessPattern,
}

/// Burst pattern type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BurstPattern {
    Steady,    // Consistent workload
    Periodic,  // Regular bursts (e.g., every hour)
    Sporadic,  // Irregular bursts
    Growing,   // Increasing workload over time
}

/// Memory access pattern
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessPattern {
    Sequential, // Sequential access (good for prefetching)
    Random,     // Random access
    Temporal,   // Temporal locality (recently accessed)
    Spatial,    // Spatial locality (nearby pages)
}

/// Workload predictor
/// 
/// Analyzes historical workload data and predicts future resource needs
pub struct WorkloadPredictor {
    history: Mutex<BTreeMap<u64, Vec<WorkloadSnapshot>>>, // agent_id -> snapshots
    patterns: Mutex<BTreeMap<u64, AgentPattern>>,        // agent_id -> pattern
    prediction_window: u64, // Prediction window in nanoseconds (default: 5 seconds)
    max_history_size: usize, // Maximum snapshots per agent (default: 1000)
}

impl WorkloadPredictor {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(BTreeMap::new()),
            patterns: Mutex::new(BTreeMap::new()),
            prediction_window: 5_000_000_000, // 5 seconds
            max_history_size: 1000,
        }
    }
    
    /// Record workload snapshot
    pub fn record_snapshot(&self, snapshot: WorkloadSnapshot) {
        let mut history = self.history.lock();
        let agent_history = history.entry(snapshot.agent_id).or_insert_with(Vec::new);
        
        // Add snapshot
        agent_history.push(snapshot.clone());
        
        // Trim history if too large
        if agent_history.len() > self.max_history_size {
            agent_history.remove(0);
        }
        
        // Update pattern if enough data
        if agent_history.len() >= 10 {
            self.update_pattern(snapshot.agent_id, &agent_history);
        }
    }
    
    /// Update agent pattern based on history
    fn update_pattern(&self, agent_id: u64, history: &[WorkloadSnapshot]) {
        if history.is_empty() {
            return;
        }
        
        // Calculate typical usage
        let mut total_cpu = 0.0;
        let mut total_memory = 0u64;
        let mut hour_counts = [0u32; 24];
        
        for snapshot in history {
            total_cpu += snapshot.cpu_usage;
            total_memory += snapshot.memory_usage;
            
            // Extract hour from timestamp
            // Convert nanoseconds to hours since epoch, then modulo 24
            let hour = ((snapshot.timestamp / 3_600_000_000_000) % 24) as usize;
            if hour < 24 {
                hour_counts[hour] += 1;
            }
        }
        
        let count = history.len() as f32;
        let typical_cpu = total_cpu / count;
        let typical_memory = total_memory / count as u64;
        
        // Find peak hours (hours with above-average activity)
        let avg_activity = history.len() as f32 / 24.0;
        let mut peak_hours = Vec::new();
        for (hour, &count) in hour_counts.iter().enumerate() {
            if count as f32 > avg_activity * 1.5 {
                peak_hours.push(hour as u8);
            }
        }
        
        // Determine burst pattern
        let burst_pattern = self.detect_burst_pattern(history);
        
        // Determine access pattern by analyzing memory access patterns
        // Analyze memory access locality from history
        let access_pattern = if history.len() >= 10 {
            // Check for sequential access (increasing addresses)
            let mut sequential_count = 0;
            for i in 1..history.len().min(10) {
                if history[i].memory_usage > history[i-1].memory_usage {
                    sequential_count += 1;
                }
            }
            if sequential_count > 5 {
                AccessPattern::Sequential
            } else {
                // Check for temporal locality (recent access patterns)
                AccessPattern::Temporal
            }
        } else {
            AccessPattern::Temporal // Default for insufficient data
        };
        
        let pattern = AgentPattern {
            agent_id,
            peak_hours,
            typical_cpu,
            typical_memory,
            burst_pattern,
            access_pattern,
        };
        
        let mut patterns = self.patterns.lock();
        patterns.insert(agent_id, pattern);
    }
    
    /// Detect burst pattern from history
    fn detect_burst_pattern(&self, history: &[WorkloadSnapshot]) -> BurstPattern {
        if history.len() < 10 {
            return BurstPattern::Steady;
        }
        
        // Calculate variance in CPU usage
        let mut cpu_values = Vec::new();
        for snapshot in history {
            cpu_values.push(snapshot.cpu_usage);
        }
        
        let mean = cpu_values.iter().sum::<f32>() / cpu_values.len() as f32;
        let variance = cpu_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / cpu_values.len() as f32;
        let std_dev = variance.sqrt();
        
        // Low variance = steady
        if std_dev < 0.1 {
            return BurstPattern::Steady;
        }
        
        // Check for periodic pattern using autocorrelation
        // Calculate autocorrelation to detect periodic patterns
        let autocorr_threshold = 0.7;
        let mut max_autocorr = 0.0;
        
        // Sample autocorrelation at different lags
        for lag in 1..cpu_values.len().min(20) {
            let mut autocorr_sum = 0.0;
            let mut count = 0;
            for i in lag..cpu_values.len() {
                autocorr_sum += (cpu_values[i] - mean) * (cpu_values[i - lag] - mean);
                count += 1;
            }
            if count > 0 {
                let autocorr = autocorr_sum / (count as f32 * variance);
                if autocorr > max_autocorr {
                    max_autocorr = autocorr;
                }
            }
        }
        
        // If high autocorrelation, pattern is periodic
        if max_autocorr > autocorr_threshold {
            return BurstPattern::Periodic;
        }
        
        // Check if variance is consistent
        if std_dev > 0.3 {
            // High variance - check if growing
            let first_half_mean: f32 = cpu_values[..cpu_values.len()/2].iter().sum::<f32>() / (cpu_values.len()/2) as f32;
            let second_half_mean: f32 = cpu_values[cpu_values.len()/2..].iter().sum::<f32>() / (cpu_values.len()/2) as f32;
            
            if second_half_mean > first_half_mean * 1.2 {
                return BurstPattern::Growing;
            }
            
            return BurstPattern::Sporadic;
        }
        
        BurstPattern::Periodic
    }
    
    /// Predict workload for agent at future time
    /// 
    /// First tries ML prediction via ML client, falls back to rule-based prediction.
    pub fn predict_workload(&self, agent_id: u64, time_ahead: u64) -> Option<WorkloadPrediction> {
        // Try ML prediction first (via ML client)
        #[cfg(feature = "alloc")]
        {
            use aios_kernel_core::ml_client::{get_ml_client, WorkloadPredictionRequest};
            
            let history = self.history.lock();
            let agent_history = history.get(&agent_id)?;
            
            if agent_history.len() >= 10 {
                // Prepare ML prediction request
                let latest = agent_history.last()?;
                let historical_cpu: Vec<f32> = agent_history.iter()
                    .rev()
                    .take(10)
                    .map(|s| s.cpu_usage)
                    .collect();
                let historical_memory: Vec<u64> = agent_history.iter()
                    .rev()
                    .take(10)
                    .map(|s| s.memory_usage)
                    .collect();
                let historical_gpu: Vec<f32> = agent_history.iter()
                    .rev()
                    .take(10)
                    .filter_map(|s| s.gpu_usage)
                    .collect();
                
                // Get current time for time_of_day and day_of_week
                let now = crate::time::now();
                // Extract time of day (0-23) from timestamp
                let time_of_day = ((now / 3_600_000_000_000) % 24) as u8;
                // Extract day of week (0-6, where 0 = Sunday) from timestamp
                let day_of_week = ((now / 86_400_000_000_000) % 7) as u8;
                
                let ml_request = WorkloadPredictionRequest {
                    agent_id,
                    historical_cpu,
                    historical_memory,
                    historical_gpu,
                    time_of_day,
                    day_of_week,
                    current_cpu: latest.cpu_usage,
                    current_memory: latest.memory_usage,
                    current_gpu: latest.gpu_usage,
                };
                
                if let Some(ml_prediction) = get_ml_client().predict_workload(ml_request) {
                    // Use ML prediction
                    return Some(WorkloadPrediction {
                        agent_id,
                        time_ahead,
                        predicted_cpu: ml_prediction.predicted_cpu,
                        predicted_memory: ml_prediction.predicted_memory,
                        predicted_gpu: ml_prediction.predicted_gpu,
                        confidence: ml_prediction.confidence,
                    });
                }
            }
        }
        
        // Fallback to rule-based prediction
        let patterns = self.patterns.lock();
        let history = self.history.lock();
        
        let pattern = patterns.get(&agent_id)?;
        let agent_history = history.get(&agent_id)?;
        
        if agent_history.is_empty() {
            return None;
        }
        
        // Get most recent snapshot
        let latest = agent_history.last()?;
        
        // Predict based on pattern
        let predicted_cpu = self.predict_cpu(pattern, latest, time_ahead);
        let predicted_memory = self.predict_memory(pattern, latest, time_ahead);
        // Predict GPU usage based on pattern
        let predicted_gpu = match pattern.burst_pattern {
            BurstPattern::Steady => latest.gpu_usage,
            BurstPattern::Growing => {
                // Predict growth in GPU usage
                let growth_rate = 1.0 + (time_ahead as f32 / 1_000_000_000.0) * 0.1;
                latest.gpu_usage.map(|gpu| (gpu * growth_rate).min(1.0))
            }
            _ => latest.gpu_usage, // Use latest for periodic/sporadic
        };
        
        Some(WorkloadPrediction {
            agent_id,
            time_ahead,
            predicted_cpu,
            predicted_memory,
            predicted_gpu,
            confidence: self.calculate_confidence(agent_history.len()),
        })
    }
    
    /// Predict CPU usage
    fn predict_cpu(&self, pattern: &AgentPattern, latest: &WorkloadSnapshot, time_ahead: u64) -> f32 {
        // Prediction algorithm: use typical if steady, otherwise use latest with trend
        match pattern.burst_pattern {
            BurstPattern::Steady => pattern.typical_cpu,
            BurstPattern::Growing => {
                // Predict growth
                let growth_rate = 1.0 + (time_ahead as f32 / 1_000_000_000.0) * 0.1; // 10% per second
                (latest.cpu_usage * growth_rate).min(1.0)
            }
            _ => latest.cpu_usage, // Use latest for periodic/sporadic
        }
    }
    
    /// Predict memory usage
    fn predict_memory(&self, pattern: &AgentPattern, latest: &WorkloadSnapshot, time_ahead: u64) -> u64 {
        match pattern.burst_pattern {
            BurstPattern::Steady => pattern.typical_memory,
            BurstPattern::Growing => {
                // Predict growth
                let growth_rate = 1.0 + (time_ahead as f32 / 1_000_000_000.0) * 0.05; // 5% per second
                ((latest.memory_usage as f32) * growth_rate) as u64
            }
            _ => latest.memory_usage, // Use latest for periodic/sporadic
        }
    }
    
    /// Calculate prediction confidence
    fn calculate_confidence(&self, history_size: usize) -> f32 {
        // More history = higher confidence
        // Confidence ranges from 0.0 to 1.0
        if history_size < 10 {
            0.3
        } else if history_size < 100 {
            0.5 + (history_size as f32 / 100.0) * 0.3
        } else {
            0.8 + ((history_size.min(1000) - 100) as f32 / 900.0) * 0.2
        }
    }
    
    /// Get agent pattern
    pub fn get_pattern(&self, agent_id: u64) -> Option<AgentPattern> {
        let patterns = self.patterns.lock();
        patterns.get(&agent_id).cloned()
    }
}

/// Workload prediction result
#[derive(Clone, Debug)]
pub struct WorkloadPrediction {
    pub agent_id: u64,
    pub time_ahead: u64,        // Nanoseconds into future
    pub predicted_cpu: f32,     // Predicted CPU usage (0.0 - 1.0)
    pub predicted_memory: u64,  // Predicted memory usage (bytes)
    pub predicted_gpu: Option<f32>, // Predicted GPU usage (0.0 - 1.0)
    pub confidence: f32,        // Prediction confidence (0.0 - 1.0)
}

/// Predictive resource allocator
/// 
/// Pre-allocates resources based on workload predictions
pub struct PredictiveAllocator {
    predictor: WorkloadPredictor,
    allocation_buffer: f32, // Percentage buffer (e.g., 0.2 = 20%)
}

impl PredictiveAllocator {
    pub fn new(predictor: WorkloadPredictor) -> Self {
        Self {
            predictor,
            allocation_buffer: 0.2, // 20% buffer
        }
    }
    
    /// Predict resource allocation needs
    pub fn predict_allocation(&self, agent_id: u64, time_ahead: u64) -> Option<ResourcePrediction> {
        let workload = self.predictor.predict_workload(agent_id, time_ahead)?;
        
        // Add buffer to predictions
        let cpu_with_buffer = (workload.predicted_cpu * (1.0 + self.allocation_buffer)).min(1.0);
        let memory_with_buffer = (workload.predicted_memory as f64 * (1.0 + self.allocation_buffer as f64)) as u64;
        let gpu_with_buffer = workload.predicted_gpu.map(|gpu| (gpu * (1.0 + self.allocation_buffer)).min(1.0));
        
        Some(ResourcePrediction {
            agent_id,
            cpu_cores: self.cpu_cores_needed(cpu_with_buffer),
            memory_bytes: memory_with_buffer,
            gpu_memory: gpu_with_buffer.map(|gpu| (gpu * 1024 * 1024 * 1024) as u64), // Assume 1GB GPU memory
            confidence: workload.confidence,
        })
    }
    
    /// Calculate CPU cores needed
    fn cpu_cores_needed(&self, cpu_usage: f32) -> u32 {
        // Assume 1 core = 100% usage
        // Round up to nearest core
        (cpu_usage * 100.0).ceil() as u32 / 100
    }
}

/// Resource prediction
#[derive(Clone, Debug)]
pub struct ResourcePrediction {
    pub agent_id: u64,
    pub cpu_cores: u32,
    pub memory_bytes: u64,
    pub gpu_memory: Option<u64>, // Bytes
    pub confidence: f32,
}

/// Global workload predictor instance
static WORKLOAD_PREDICTOR: Mutex<Option<WorkloadPredictor>> = Mutex::new(None);

/// Initialize workload predictor
pub fn init() {
    let mut predictor = WORKLOAD_PREDICTOR.lock();
    *predictor = Some(WorkloadPredictor::new());
}

/// Get workload predictor instance
pub fn get() -> Option<&'static WorkloadPredictor> {
    let predictor = WORKLOAD_PREDICTOR.lock();
    predictor.as_ref()
}

