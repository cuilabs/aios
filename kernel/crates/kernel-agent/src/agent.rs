//! Agent structure and management

use crate::AgentSpec;
use crate::AgentError;

/// Agent state
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AgentState {
    Created,
    Running,
    Waiting,
    Terminated,
}

/// An AI agent in AIOS
/// 
/// Agents are first-class citizens in AIOS, replacing traditional processes
pub struct Agent {
    id: u64,
    state: AgentState,
    spec: AgentSpec,
    context: AgentContext,
}

struct AgentContext {
    memory_pool: *mut u8,
    memory_size: usize,
}

impl Agent {
    pub fn new(spec: AgentSpec) -> Result<Self, AgentError> {
        // Validate spec
        if spec.memory_limit == 0 {
            return Err(AgentError::InvalidSpec);
        }

        Ok(Self {
            id: spec.id,
            state: AgentState::Created,
            spec,
            context: AgentContext {
                memory_pool: core::ptr::null_mut(),
                memory_size: 0,
            },
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> AgentState {
        self.state
    }

    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
    }
}

