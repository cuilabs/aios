//! Agent management API

use crate::kernel::KernelClient;

/// Agent client
pub struct AgentClient {
    kernel: KernelClient,
}

impl AgentClient {
    /// Spawn agent
    pub fn spawn(&self, config: AgentConfig) -> Result<u64, AgentError> {
        // TODO: Spawn agent via kernel
        Ok(0)
    }
    
    /// Get agent status
    pub fn status(&self, agent_id: u64) -> Result<AgentStatus, AgentError> {
        // TODO: Get agent status
        Err(AgentError::NotFound)
    }
}

/// Agent configuration
pub struct AgentConfig {
    pub memory_size: usize,
    pub priority: i32,
}

/// Agent status
pub struct AgentStatus {
    pub agent_id: u64,
    pub state: AgentState,
}

/// Agent state
pub enum AgentState {
    Running,
    Stopped,
    Failed,
}

/// Agent error
#[derive(Debug)]
pub enum AgentError {
    NotFound,
    PermissionDenied,
    ResourceExhausted,
}

