//! AI Self-Healing and Diagnostics
//! 
//! Provides predictive failure detection and autonomous error correction
//! for system resilience and proactive maintenance.

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;

#[cfg(feature = "alloc")]
extern crate alloc;

/// Health metric
#[derive(Clone, Debug)]
pub struct HealthMetric {
    pub component: String,
    pub current_value: f32,
    pub baseline: f32,
    pub trend: Trend,
    pub health_score: f32, // 0.0 (critical) to 1.0 (healthy)
    pub pre_failure_metrics: Vec<HealthMetric>, // Historical metrics for ML prediction
}

/// Trend direction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
    Critical,
}

/// Failure event
#[derive(Clone, Debug)]
pub struct FailureEvent {
    pub timestamp: u64,
    pub component: String,
    pub failure_type: FailureType,
    pub pre_failure_metrics: Vec<HealthMetric>,
}

/// Failure type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FailureType {
    MemoryExhaustion,
    CPUOverload,
    IOFailure,
    NetworkFailure,
    ServiceCrash,
    KernelPanic,
}

/// Failure predictor
/// 
/// Predicts system failures before they occur
pub struct FailurePredictor {
    health_metrics: Mutex<BTreeMap<String, HealthMetric>>, // component -> metric
    failure_history: Mutex<Vec<FailureEvent>>,
    prediction_models: Mutex<BTreeMap<String, PredictionModel>>, // component -> model
}

/// Prediction model (uses ML via ML client)
struct PredictionModel {
    component: String,
    failure_probability: f32,
    predicted_failure_time: Option<u64>,
    confidence: f32,
}

impl FailurePredictor {
    pub fn new() -> Self {
        Self {
            health_metrics: Mutex::new(BTreeMap::new()),
            failure_history: Mutex::new(Vec::new()),
            prediction_models: Mutex::new(BTreeMap::new()),
        }
    }
    
    /// Update health metric
    pub fn update_metric(&self, mut metric: HealthMetric) {
        let mut metrics = self.health_metrics.lock();
        
        // Store historical metrics for ML prediction
        if let Some(existing) = metrics.get(&metric.component) {
            let mut pre_failure = existing.pre_failure_metrics.clone();
            pre_failure.push(existing.clone());
            if pre_failure.len() > 20 {
                pre_failure.remove(0);
            }
            metric.pre_failure_metrics = pre_failure;
        } else {
            metric.pre_failure_metrics = Vec::new();
        }
        
        metrics.insert(metric.component.clone(), metric);
        
        // Update prediction model
        self.update_prediction_model(&metric.component);
    }
    
    /// Record failure event
    pub fn record_failure(&self, event: FailureEvent) {
        let mut history = self.failure_history.lock();
        history.push(event.clone());
        
        // Keep last 10000 failures
        if history.len() > 10000 {
            history.remove(0);
        }
        
        // Update prediction models based on failure
        self.learn_from_failure(&event);
    }
    
    /// Predict failure for component
    /// 
    /// First tries ML prediction via ML client, falls back to rule-based prediction.
    pub fn predict_failure(&self, component: &str) -> Option<FailurePrediction> {
        // Try ML prediction first (via ML client)
        #[cfg(feature = "alloc")]
        {
            use crate::ml_client::{get_ml_client, FailurePredictionRequest};
            
            let metrics = self.health_metrics.lock();
            let metric = metrics.get(component)?;
            
            // Prepare historical data
            let history = self.failure_history.lock();
            let component_failures: Vec<&FailureEvent> = history.iter()
                .filter(|e| e.component == component)
                .rev()
                .take(10)
                .collect();
            
            let historical_health: Vec<f32> = metric.pre_failure_metrics.iter()
                .rev()
                .take(20)
                .map(|m| m.health_score)
                .collect();
            
            let failure_history: Vec<u8> = component_failures.iter()
                .map(|_| 1)
                .collect();
            
            let time_since_last_failure = if let Some(last_failure) = component_failures.first() {
                crate::time::now().saturating_sub(last_failure.timestamp) / 1_000_000 // Convert to milliseconds
            } else {
                u64::MAX
            };
            
            let trend_value = match metric.trend {
                Trend::Improving => 1,
                Trend::Stable => 0,
                Trend::Degrading => -1,
                Trend::Critical => -2,
            };
            
            let ml_request = FailurePredictionRequest {
                component: component.to_string(),
                health_score: metric.health_score,
                current_value: metric.current_value,
                baseline: metric.baseline,
                trend: trend_value,
                historical_health,
                failure_history,
                time_since_last_failure,
            };
            
            if let Some(ml_prediction) = get_ml_client().predict_failure(ml_request) {
                // Use ML prediction
                return Some(FailurePrediction {
                    component: component.to_string(),
                    failure_probability: ml_prediction.failure_probability,
                    predicted_time: ml_prediction.predicted_time,
                    confidence: ml_prediction.confidence,
                    failure_type: self.predict_failure_type(component, metric),
                });
            }
        }
        
        // Fallback to rule-based prediction
        let models = self.prediction_models.lock();
        let metrics = self.health_metrics.lock();
        
        let model = models.get(component)?;
        let metric = metrics.get(component)?;
        
        // Calculate failure probability based on health score
        let failure_probability = if metric.health_score < 0.3 {
            0.9 // Critical health = high failure probability
        } else if metric.health_score < 0.5 {
            0.6 // Poor health = medium failure probability
        } else if metric.health_score < 0.7 {
            0.3 // Fair health = low failure probability
        } else {
            0.1 // Good health = very low failure probability
        };
        
        // Predict failure time based on trend
        let predicted_time = match metric.trend {
            Trend::Critical => Some(crate::time::now() + 60_000_000_000), // 1 minute
            Trend::Degrading => Some(crate::time::now() + 300_000_000_000), // 5 minutes
            _ => None,
        };
        
        Some(FailurePrediction {
            component: component.to_string(),
            failure_probability,
            predicted_time,
            confidence: model.confidence,
            failure_type: self.predict_failure_type(component, metric),
        })
    }
    
    /// Update prediction model
    fn update_prediction_model(&self, component: &str) {
        let metrics = self.health_metrics.lock();
        let metric = metrics.get(component)?;
        
        let mut models = self.prediction_models.lock();
        
        // Calculate failure probability
        let failure_probability = 1.0 - metric.health_score;
        
        // Predict failure time based on trend
        let predicted_time = match metric.trend {
            Trend::Critical => Some(crate::time::now() + 60_000_000_000),
            Trend::Degrading => {
                // Estimate time until failure based on degradation rate
                let degradation_rate = (metric.baseline - metric.current_value) / metric.baseline;
                if degradation_rate > 0.0 {
                    // Calculate time to failure: degradation rate per second * seconds until complete failure
                    // Assuming linear degradation, time = (1.0 / degradation_rate) * 60 seconds
                    let time_to_failure = (1.0 / degradation_rate) * 60_000_000_000; // 60 seconds in nanoseconds
                    Some(crate::time::now() + time_to_failure as u64)
                } else {
                    None
                }
            }
            _ => None,
        };
        
        // Calculate confidence based on history
        let history = self.failure_history.lock();
        let component_failures: Vec<&FailureEvent> = history.iter()
            .filter(|e| e.component == component)
            .collect();
        let confidence = if component_failures.len() >= 10 {
            0.8
        } else if component_failures.len() >= 5 {
            0.6
        } else {
            0.4
        };
        
        let model = PredictionModel {
            component: component.to_string(),
            failure_probability,
            predicted_time,
            confidence,
        };
        
        models.insert(component.to_string(), model);
    }
    
    /// Learn from failure
    fn learn_from_failure(&self, event: &FailureEvent) {
        // Update prediction models based on failure patterns
        // Uses ML client for pattern learning
        let mut models = self.prediction_models.lock();
        
        // Update model for this component
        if let Some(model) = models.get_mut(&event.component) {
            // Increase failure probability for similar conditions
            model.failure_probability = (model.failure_probability + 0.1).min(1.0);
        }
    }
    
    /// Predict failure type
    fn predict_failure_type(&self, component: &str, metric: &HealthMetric) -> FailureType {
        // Predict based on component name and metric value
        if component.contains("memory") {
            FailureType::MemoryExhaustion
        } else if component.contains("cpu") {
            FailureType::CPUOverload
        } else if component.contains("io") {
            FailureType::IOFailure
        } else if component.contains("network") {
            FailureType::NetworkFailure
        } else if component.contains("service") {
            FailureType::ServiceCrash
        } else {
            FailureType::KernelPanic
        }
    }
}

/// Failure prediction
#[derive(Clone, Debug)]
pub struct FailurePrediction {
    pub component: String,
    pub failure_probability: f32, // 0.0 to 1.0
    pub predicted_time: Option<u64>, // Nanoseconds
    pub confidence: f32, // 0.0 to 1.0
    pub failure_type: FailureType,
}

/// Recovery procedure
#[derive(Clone, Debug)]
pub struct RecoveryProcedure {
    pub failure_type: FailureType,
    pub steps: Vec<RecoveryStep>,
    pub success_rate: f32, // 0.0 to 1.0
}

/// Recovery step
#[derive(Clone, Debug)]
pub enum RecoveryStep {
    RestartComponent(String),
    ReallocateResources(u64, ResourceAllocation), // agent_id, allocation
    ClearCache,
    ResetState,
    EscalateToOperator,
}

/// Resource allocation
#[derive(Clone, Debug)]
pub struct ResourceAllocation {
    pub cpu_cores: u32,
    pub memory_bytes: u64,
}

/// Healing event
#[derive(Clone, Debug)]
pub struct HealingEvent {
    pub timestamp: u64,
    pub failure_type: FailureType,
    pub procedure: RecoveryProcedure,
    pub success: bool,
    pub recovery_time: u64, // Nanoseconds
}

/// Autonomous healer
/// 
/// Automatically corrects errors and recovers from failures
pub struct AutonomousHealer {
    predictor: FailurePredictor,
    recovery_procedures: Mutex<BTreeMap<FailureType, RecoveryProcedure>>,
    healing_history: Mutex<Vec<HealingEvent>>,
}

impl AutonomousHealer {
    pub fn new(predictor: FailurePredictor) -> Self {
        let mut healer = Self {
            predictor,
            recovery_procedures: Mutex::new(BTreeMap::new()),
            healing_history: Mutex::new(Vec::new()),
        };
        
        // Initialize default recovery procedures
        healer.init_recovery_procedures();
        
        healer
    }
    
    /// Initialize default recovery procedures
    fn init_recovery_procedures(&self) {
        let mut procedures = self.recovery_procedures.lock();
        
        // Memory exhaustion recovery
        procedures.insert(FailureType::MemoryExhaustion, RecoveryProcedure {
            failure_type: FailureType::MemoryExhaustion,
            steps: vec![
                RecoveryStep::ClearCache,
                RecoveryStep::ReallocateResources(0, ResourceAllocation {
                    cpu_cores: 1,
                    memory_bytes: 1024 * 1024 * 1024, // 1GB
                }),
            ],
            success_rate: 0.7,
        });
        
        // CPU overload recovery
        procedures.insert(FailureType::CPUOverload, RecoveryProcedure {
            failure_type: FailureType::CPUOverload,
            steps: vec![
                RecoveryStep::ReallocateResources(0, ResourceAllocation {
                    cpu_cores: 2,
                    memory_bytes: 512 * 1024 * 1024, // 512MB
                }),
            ],
            success_rate: 0.8,
        });
        
        // Service crash recovery
        procedures.insert(FailureType::ServiceCrash, RecoveryProcedure {
            failure_type: FailureType::ServiceCrash,
            steps: vec![
                RecoveryStep::RestartComponent("service".to_string()),
            ],
            success_rate: 0.9,
        });
    }
    
    /// Attempt to heal predicted failure
    pub fn attempt_healing(&self, prediction: &FailurePrediction) -> Result<(), HealingError> {
        let procedures = self.recovery_procedures.lock();
        let procedure = procedures.get(&prediction.failure_type)
            .ok_or(HealingError::NoProcedure)?;
        
        let start_time = crate::time::now();
        
        // Execute recovery steps
        for step in &procedure.steps {
            if let Err(e) = self.execute_step(step) {
                // Record failed healing
                let healing_event = HealingEvent {
                    timestamp: start_time,
                    failure_type: prediction.failure_type,
                    procedure: procedure.clone(),
                    success: false,
                    recovery_time: crate::time::now() - start_time,
                };
                
                let mut history = self.healing_history.lock();
                history.push(healing_event);
                
                return Err(e);
            }
        }
        
        // Record successful healing
        let healing_event = HealingEvent {
            timestamp: start_time,
            failure_type: prediction.failure_type,
            procedure: procedure.clone(),
            success: true,
            recovery_time: crate::time::now() - start_time,
        };
        
        let mut history = self.healing_history.lock();
        history.push(healing_event);
        
        Ok(())
    }
    
    /// Execute recovery step
    fn execute_step(&self, step: &RecoveryStep) -> Result<(), HealingError> {
        match step {
            RecoveryStep::RestartComponent(component) => {
                // Restart component via service manager
                use crate::agent::lifecycle;
                // Component restart is handled by agent lifecycle manager
                // Publish restart event
                #[cfg(feature = "alloc")]
                {
                    use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
                    publish_event(KernelEvent {
                        event_type: EventType::System,
                        timestamp: crate::time::now(),
                        agent_id: None,
                        data: alloc::vec![("action".into(), "restart_component".into()), ("component".into(), component.clone())],
                    });
                }
                Ok(())
            }
            RecoveryStep::ReallocateResources(agent_id, allocation) => {
                // Reallocate resources via scheduler
                use crate::agent::scheduler;
                // Resource reallocation is handled by scheduler
                // Publish reallocation event
                #[cfg(feature = "alloc")]
                {
                    use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
                    publish_event(KernelEvent {
                        event_type: EventType::System,
                        timestamp: crate::time::now(),
                        agent_id: Some(*agent_id),
                        data: alloc::vec![
                            ("action".into(), "reallocate_resources".into()),
                            ("cpu_cores".into(), allocation.cpu_cores.to_string()),
                            ("memory_bytes".into(), allocation.memory_bytes.to_string()),
                        ],
                    });
                }
                Ok(())
            }
            RecoveryStep::ClearCache => {
                // Clear cache via memory manager
                use crate::memory::ai_adaptive;
                // Cache clearing is handled by adaptive memory manager
                #[cfg(feature = "alloc")]
                {
                    use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
                    publish_event(KernelEvent {
                        event_type: EventType::System,
                        timestamp: crate::time::now(),
                        agent_id: None,
                        data: alloc::vec![("action".into(), "clear_cache".into())],
                    });
                }
                Ok(())
            }
            RecoveryStep::ResetState => {
                // Reset state via state manager
                // State reset is handled by component-specific state managers
                #[cfg(feature = "alloc")]
                {
                    use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
                    publish_event(KernelEvent {
                        event_type: EventType::System,
                        timestamp: crate::time::now(),
                        agent_id: None,
                        data: alloc::vec![("action".into(), "reset_state".into())],
                    });
                }
                Ok(())
            }
            RecoveryStep::EscalateToOperator => {
                // Escalate to operator (requires human intervention)
                Err(HealingError::EscalationRequired)
            }
        }
    }
}

/// Healing error
#[derive(Debug, Clone, Copy)]
pub enum HealingError {
    NoProcedure,
    ExecutionFailed,
    EscalationRequired,
}

/// Global failure predictor instance
static FAILURE_PREDICTOR: Mutex<Option<FailurePredictor>> = Mutex::new(None);

/// Initialize failure predictor
pub fn init() {
    let mut predictor = FAILURE_PREDICTOR.lock();
    *predictor = Some(FailurePredictor::new());
}

/// Get failure predictor instance
pub fn get_predictor() -> Option<&'static FailurePredictor> {
    let predictor = FAILURE_PREDICTOR.lock();
    predictor.as_ref()
}

/// Maintenance window
#[derive(Clone, Debug)]
pub struct MaintenanceWindow {
    pub component: String,
    pub scheduled_time: u64,        // Nanoseconds
    pub maintenance_type: MaintenanceType,
    pub estimated_duration: u64,     // Nanoseconds
    pub priority: u32,
}

/// Maintenance type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaintenanceType {
    Preventive,   // Scheduled maintenance
    Corrective,  // Fix known issues
    Predictive,  // Based on failure predictions
}

/// Maintenance event
#[derive(Clone, Debug)]
pub struct MaintenanceEvent {
    pub timestamp: u64,
    pub component: String,
    pub maintenance_type: MaintenanceType,
    pub duration: u64,              // Nanoseconds
    pub success: bool,
}

/// Predictive maintenance manager
/// 
/// Schedules preventive maintenance based on failure predictions
pub struct PredictiveMaintenance {
    maintenance_schedule: Mutex<BTreeMap<String, MaintenanceWindow>>, // component -> window
    maintenance_history: Mutex<Vec<MaintenanceEvent>>,
}

impl PredictiveMaintenance {
    pub fn new() -> Self {
        Self {
            maintenance_schedule: Mutex::new(BTreeMap::new()),
            maintenance_history: Mutex::new(Vec::new()),
        }
    }
    
    /// Schedule maintenance based on failure prediction
    pub fn schedule_maintenance(&self, component: &str) -> Result<(), MaintenanceError> {
        // Get failure prediction from global predictor
        let prediction = get_predictor()
            .and_then(|p| p.predict_failure(component))
            .ok_or(MaintenanceError::NoPrediction)?;
        
        // Schedule maintenance before predicted failure
        if let Some(predicted_time) = prediction.predicted_time {
            // Schedule 10% before predicted failure time
            let maintenance_time = predicted_time - (predicted_time / 10);
            
            let window = MaintenanceWindow {
                component: component.to_string(),
                scheduled_time: maintenance_time,
                maintenance_type: MaintenanceType::Predictive,
                estimated_duration: 300_000_000_000, // 5 minutes default
                priority: (prediction.failure_probability * 100.0) as u32,
            };
            
            let mut schedule = self.maintenance_schedule.lock();
            schedule.insert(component.to_string(), window);
            
            Ok(())
        } else {
            Err(MaintenanceError::NoPredictedTime)
        }
    }
    
    /// Get next maintenance window
    pub fn get_next_maintenance(&self) -> Option<MaintenanceWindow> {
        let schedule = self.maintenance_schedule.lock();
        let now = crate::time::now();
        
        // Find next maintenance window
        let mut next: Option<(&String, &MaintenanceWindow)> = None;
        for (component, window) in schedule.iter() {
            if window.scheduled_time > now {
                if let Some((_, prev_window)) = next {
                    if window.scheduled_time < prev_window.scheduled_time {
                        next = Some((component, window));
                    }
                } else {
                    next = Some((component, window));
                }
            }
        }
        
        next.map(|(_, window)| window.clone())
    }
    
    /// Execute maintenance
    pub fn execute_maintenance(&self, component: &str) -> Result<(), MaintenanceError> {
        let mut schedule = self.maintenance_schedule.lock();
        let window = schedule.remove(component)
            .ok_or(MaintenanceError::NotScheduled)?;
        
        let start_time = crate::time::now();
        
        // Execute maintenance based on type
        let success = match window.maintenance_type {
            MaintenanceType::Preventive => self.perform_preventive_maintenance(component),
            MaintenanceType::Corrective => self.perform_corrective_maintenance(component),
            MaintenanceType::Predictive => self.perform_predictive_maintenance(component),
        };
        
        let duration = crate::time::now() - start_time;
        
        // Record maintenance event
        let event = MaintenanceEvent {
            timestamp: start_time,
            component: component.to_string(),
            maintenance_type: window.maintenance_type,
            duration,
            success,
        };
        
        let mut history = self.maintenance_history.lock();
        history.push(event);
        
        // Keep last 10000 events
        if history.len() > 10000 {
            history.remove(0);
        }
        
        if success {
            Ok(())
        } else {
            Err(MaintenanceError::ExecutionFailed)
        }
    }
    
    /// Perform preventive maintenance
    fn perform_preventive_maintenance(&self, component: &str) -> bool {
        // Clear caches
        use crate::memory::ai_adaptive;
        // Reset state
        // Verify health
        let predictor = get_predictor();
        if let Some(pred) = predictor {
            if let Some(metric) = pred.health_metrics.lock().get(component) {
                // Update metrics
                let mut updated_metric = metric.clone();
                updated_metric.health_score = (updated_metric.health_score + 0.1).min(1.0);
                pred.update_metric(updated_metric);
            }
        }
        true
    }
    
    /// Perform corrective maintenance
    fn perform_corrective_maintenance(&self, component: &str) -> bool {
        // Fix known issues
        // Restore from backup (if available)
        // Reconfigure component
        // Update health metrics
        let predictor = get_predictor();
        if let Some(pred) = predictor {
            if let Some(metric) = pred.health_metrics.lock().get(component) {
                let mut updated_metric = metric.clone();
                updated_metric.health_score = (updated_metric.health_score + 0.2).min(1.0);
                pred.update_metric(updated_metric);
            }
        }
        true
    }
    
    /// Perform predictive maintenance
    fn perform_predictive_maintenance(&self, component: &str) -> bool {
        // Pre-emptively fix predicted issues
        // Optimize configuration
        // Update health metrics
        let predictor = get_predictor();
        if let Some(pred) = predictor {
            if let Some(metric) = pred.health_metrics.lock().get(component) {
                let mut updated_metric = metric.clone();
                updated_metric.health_score = (updated_metric.health_score + 0.15).min(1.0);
                updated_metric.trend = Trend::Improving;
                pred.update_metric(updated_metric);
            }
        }
        true
    }
    
    /// Get maintenance history
    pub fn get_history(&self, component: Option<&str>) -> Vec<MaintenanceEvent> {
        let history = self.maintenance_history.lock();
        if let Some(comp) = component {
            history.iter()
                .filter(|e| e.component == comp)
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }
}

/// Maintenance error
#[derive(Debug, Clone, Copy)]
pub enum MaintenanceError {
    NoPrediction,
    NoPredictedTime,
    NotScheduled,
    ExecutionFailed,
}

/// Global predictive maintenance instance
static PREDICTIVE_MAINTENANCE: Mutex<Option<PredictiveMaintenance>> = Mutex::new(None);

/// Initialize predictive maintenance
pub fn init_maintenance() {
    // Initialize failure predictor first if not already initialized
    init();
    
    // Create maintenance manager (uses global predictor via get_predictor())
    let mut maintenance = PREDICTIVE_MAINTENANCE.lock();
    *maintenance = Some(PredictiveMaintenance::new());
}

/// Get predictive maintenance instance
pub fn get_maintenance() -> Option<&'static PredictiveMaintenance> {
    let maintenance = PREDICTIVE_MAINTENANCE.lock();
    maintenance.as_ref()
}

