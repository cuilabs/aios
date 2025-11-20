//! Error Recovery and Resilience
//! 
//! Provides retry mechanisms, circuit breakers, and graceful degradation

use crate::error::{ErrorHandler, ErrorSeverity, ErrorCategory, GlobalError};
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Circuit breaker state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing - reject requests
    HalfOpen,  // Testing - allow limited requests
}

/// Circuit breaker
#[cfg(feature = "alloc")]
pub struct CircuitBreaker {
    state: Mutex<CircuitState>,
    failure_count: Mutex<u32>,
    failure_threshold: u32,
    success_count: Mutex<u32>,
    success_threshold: u32,
    last_failure_time: Mutex<u64>,
    timeout: u64, // Timeout in nanoseconds
}

#[cfg(feature = "alloc")]
impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: u64) -> Self {
        Self {
            state: Mutex::new(CircuitState::Closed),
            failure_count: Mutex::new(0),
            failure_threshold,
            success_count: Mutex::new(0),
            success_threshold: 1,
            last_failure_time: Mutex::new(0),
            timeout,
        }
    }

    /// Check if operation is allowed
    pub fn is_allowed(&self) -> bool {
        let state = *self.state.lock();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                let last_failure = *self.last_failure_time.lock();
                let now = crate::time::now();
                if now - last_failure >= self.timeout {
                    // Transition to half-open
                    *self.state.lock() = CircuitState::HalfOpen;
                    *self.success_count.lock() = 0;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record success
    pub fn record_success(&self) {
        let mut state = self.state.lock();
        match *state {
            CircuitState::Closed => {
                // Reset failure count
                *self.failure_count.lock() = 0;
            }
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.lock();
                *success_count += 1;
                if *success_count >= self.success_threshold {
                    // Transition to closed
                    *state = CircuitState::Closed;
                    *self.failure_count.lock() = 0;
                }
            }
            CircuitState::Open => {
                // Should not happen, but handle gracefully
            }
        }
    }

    /// Record failure
    pub fn record_failure(&self) {
        let mut state = self.state.lock();
        let mut failure_count = self.failure_count.lock();
        *failure_count += 1;
        *self.last_failure_time.lock() = crate::time::now();

        match *state {
            CircuitState::Closed | CircuitState::HalfOpen => {
                if *failure_count >= self.failure_threshold {
                    // Transition to open
                    *state = CircuitState::Open;
                }
            }
            CircuitState::Open => {
                // Already open
            }
        }
    }
}

/// Retry policy
#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: u64, // Nanoseconds
    pub max_delay: u64,
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: 1_000_000, // 1ms
            max_delay: 100_000_000,   // 100ms
            backoff_multiplier: 2.0,
        }
    }

    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let delay = (self.initial_delay as f64) * (self.backoff_multiplier.powi(attempt as i32));
        core::cmp::min(delay as u64, self.max_delay)
    }
}

/// Error recovery manager
pub struct ErrorRecoveryManager {
    #[cfg(feature = "alloc")]
    circuit_breakers: Mutex<BTreeMap<u64, CircuitBreaker>>, // Resource ID -> Circuit breaker
    #[cfg(feature = "alloc")]
    retry_policies: Mutex<BTreeMap<u32, RetryPolicy>>, // Error code -> Retry policy
    #[cfg(not(feature = "alloc"))]
    circuit_breakers: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    retry_policies: Mutex<()>,
}

static RECOVERY_MANAGER: Mutex<Option<ErrorRecoveryManager>> = Mutex::new(None);

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            circuit_breakers: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            retry_policies: Mutex::new(BTreeMap::new()),
            #[cfg(not(feature = "alloc"))]
            circuit_breakers: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            retry_policies: Mutex::new(()),
        }
    }

    /// Initialize recovery manager
    pub fn init() {
        let mut manager = RECOVERY_MANAGER.lock();
        *manager = Some(ErrorRecoveryManager::new());
    }

    /// Get recovery manager
    pub fn get() -> Option<&'static ErrorRecoveryManager> {
        let manager = RECOVERY_MANAGER.lock();
        manager.as_ref()
    }

    /// Check if circuit breaker allows operation
    #[cfg(feature = "alloc")]
    pub fn check_circuit_breaker(&self, resource_id: u64) -> bool {
        let mut breakers = self.circuit_breakers.lock();
        let breaker = breakers.entry(resource_id)
            .or_insert_with(|| CircuitBreaker::new(5, 1_000_000_000)); // 5 failures, 1s timeout
        breaker.is_allowed()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn check_circuit_breaker(&self, _resource_id: u64) -> bool {
        true
    }

    /// Record circuit breaker success
    #[cfg(feature = "alloc")]
    pub fn record_circuit_breaker_success(&self, resource_id: u64) {
        let mut breakers = self.circuit_breakers.lock();
        if let Some(breaker) = breakers.get_mut(&resource_id) {
            breaker.record_success();
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn record_circuit_breaker_success(&self, _resource_id: u64) {
        // No-op
    }

    /// Record circuit breaker failure
    #[cfg(feature = "alloc")]
    pub fn record_circuit_breaker_failure(&self, resource_id: u64) {
        let mut breakers = self.circuit_breakers.lock();
        let breaker = breakers.entry(resource_id)
            .or_insert_with(|| CircuitBreaker::new(5, 1_000_000_000));
        breaker.record_failure();
    }

    #[cfg(not(feature = "alloc"))]
    pub fn record_circuit_breaker_failure(&self, _resource_id: u64) {
        // No-op
    }

    /// Get retry policy for error code
    #[cfg(feature = "alloc")]
    pub fn get_retry_policy(&self, error_code: u32) -> RetryPolicy {
        let policies = self.retry_policies.lock();
        policies.get(&error_code)
            .copied()
            .unwrap_or_else(RetryPolicy::default)
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_retry_policy(&self, _error_code: u32) -> RetryPolicy {
        RetryPolicy::default()
    }

    /// Execute operation with retry
    #[cfg(feature = "alloc")]
    pub fn execute_with_retry<F, T, E>(&self, operation: F, error_code: u32) -> Result<T, E>
    where
        F: Fn() -> Result<T, E>,
        E: Clone,
    {
        let policy = self.get_retry_policy(error_code);
        let mut last_error: Option<E> = None;

        for attempt in 0..policy.max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());
                    if attempt < policy.max_attempts - 1 {
                        let delay = policy.calculate_delay(attempt);
                        // Busy-wait delay (proper sleep would use timer interrupts)
                        let start = crate::time::now();
                        while crate::time::now() - start < delay {
                            core::hint::spin_loop();
                        }
                    }
                }
            }
        }

        Err(last_error.expect("Retry loop should have at least one error"))
    }

    #[cfg(not(feature = "alloc"))]
    pub fn execute_with_retry<F, T, E>(&self, operation: F, _error_code: u32) -> Result<T, E>
    where
        F: Fn() -> Result<T, E>,
    {
        operation()
    }
}

/// Graceful degradation level
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DegradationLevel {
    Normal,
    Reduced,      // Reduced functionality
    Minimal,      // Minimal functionality
    Emergency,    // Emergency mode only
}

/// Graceful degradation manager
pub struct DegradationManager {
    #[cfg(feature = "alloc")]
    current_level: Mutex<DegradationLevel>,
    #[cfg(feature = "alloc")]
    resource_states: Mutex<BTreeMap<u64, bool>>, // Resource ID -> Available
    #[cfg(not(feature = "alloc"))]
    current_level: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    resource_states: Mutex<()>,
}

static DEGRADATION_MANAGER: Mutex<Option<DegradationManager>> = Mutex::new(None);

impl DegradationManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            current_level: Mutex::new(DegradationLevel::Normal),
            #[cfg(feature = "alloc")]
            resource_states: Mutex::new(BTreeMap::new()),
            #[cfg(not(feature = "alloc"))]
            current_level: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            resource_states: Mutex::new(()),
        }
    }

    /// Initialize degradation manager
    pub fn init() {
        let mut manager = DEGRADATION_MANAGER.lock();
        *manager = Some(DegradationManager::new());
    }

    /// Get degradation manager
    pub fn get() -> Option<&'static DegradationManager> {
        let manager = DEGRADATION_MANAGER.lock();
        manager.as_ref()
    }

    /// Get current degradation level
    #[cfg(feature = "alloc")]
    pub fn get_level(&self) -> DegradationLevel {
        *self.current_level.lock()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_level(&self) -> DegradationLevel {
        DegradationLevel::Normal
    }

    /// Set resource availability
    #[cfg(feature = "alloc")]
    pub fn set_resource_available(&self, resource_id: u64, available: bool) {
        let mut states = self.resource_states.lock();
        states.insert(resource_id, available);
        
        // Recalculate degradation level
        self.recalculate_level();
    }

    #[cfg(not(feature = "alloc"))]
    pub fn set_resource_available(&self, _resource_id: u64, _available: bool) {
        // No-op
    }

    /// Recalculate degradation level
    #[cfg(feature = "alloc")]
    fn recalculate_level(&self) {
        let states = self.resource_states.lock();
        let available_count = states.values().filter(|&&v| v).count();
        let total_count = states.len();
        
        let level = if total_count == 0 {
            DegradationLevel::Normal
        } else {
            let availability_ratio = available_count as f64 / total_count as f64;
            if availability_ratio >= 0.9 {
                DegradationLevel::Normal
            } else if availability_ratio >= 0.7 {
                DegradationLevel::Reduced
            } else if availability_ratio >= 0.5 {
                DegradationLevel::Minimal
            } else {
                DegradationLevel::Emergency
            }
        };
        
        *self.current_level.lock() = level;
    }

    #[cfg(not(feature = "alloc"))]
    fn recalculate_level(&self) {
        // No-op
    }
}

/// Initialize error recovery system
pub fn init() {
    ErrorRecoveryManager::init();
    DegradationManager::init();
}

