//! Global Error Taxonomy
//! 
//! Unified error model for kernel and userland

use alloc::vec::Vec;

/// Error category
#[repr(u32)]
pub enum ErrorCategory {
    IO = 1,
    Memory = 2,
    Network = 3,
    Security = 4,
    System = 5,
    Agent = 6,
    Capability = 7,
    Resource = 8,
}

/// Error severity
#[repr(u32)]
pub enum ErrorSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
    Fatal = 5,
}

/// Global error
#[repr(C)]
pub struct GlobalError {
    pub category: ErrorCategory,
    pub severity: ErrorSeverity,
    pub code: u32,
    pub agent_id: Option<u64>,
    pub resource: Option<ResourceId>,
    pub operation: Option<OperationId>,
    pub context: Vec<u8>,
    pub timestamp: u64,
}

/// Resource identifier
#[repr(C)]
pub struct ResourceId {
    pub resource_type: u32,
    pub resource_id: u64,
}

/// Operation identifier
#[repr(C)]
pub struct OperationId {
    pub operation_type: u32,
    pub operation_id: u64,
}

/// Error propagation rules
pub struct ErrorPropagation {
    pub retry: bool,
    pub retry_count: u32,
    pub escalate: bool,
    pub escalate_to: Option<u64>, // Agent/service ID
}

/// Error handler
pub struct ErrorHandler;

impl ErrorHandler {
    /// Create error
    pub fn create(
        category: ErrorCategory,
        severity: ErrorSeverity,
        code: u32,
        agent_id: Option<u64>,
    ) -> GlobalError {
        GlobalError {
            category,
            severity,
            code,
            agent_id,
            resource: None,
            operation: None,
            context: Vec::new(),
            timestamp: crate::time::now(),
        }
    }

    /// Get propagation rules for error
    pub fn get_propagation_rules(error: &GlobalError) -> ErrorPropagation {
        match error.severity {
            ErrorSeverity::Info | ErrorSeverity::Warning => ErrorPropagation {
                retry: false,
                retry_count: 0,
                escalate: false,
                escalate_to: None,
            },
            ErrorSeverity::Error => ErrorPropagation {
                retry: true,
                retry_count: 3,
                escalate: false,
                escalate_to: None,
            },
            ErrorSeverity::Critical => ErrorPropagation {
                retry: true,
                retry_count: 1,
                escalate: true,
                escalate_to: Some(0), // Escalate to kernel error handler (agent ID 0 = kernel)
            },
            ErrorSeverity::Fatal => ErrorPropagation {
                retry: false,
                retry_count: 0,
                escalate: true,
                escalate_to: Some(0), // Escalate to kernel error handler (agent ID 0 = kernel)
            },
        }
    }

    /// Should retry error
    pub fn should_retry(error: &GlobalError, attempt: u32) -> bool {
        let rules = Self::get_propagation_rules(error);
        rules.retry && attempt < rules.retry_count
    }

    /// Should escalate error
    pub fn should_escalate(error: &GlobalError) -> bool {
        let rules = Self::get_propagation_rules(error);
        rules.escalate
    }
}

