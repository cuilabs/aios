#![no_std]

//! Agent management kernel module
//! 
//! Provides agent-first process management for AIOS

extern crate alloc;

pub mod agent;
pub mod scheduler;
pub mod lifecycle;
#[cfg(feature = "alloc")]
pub mod ai_scheduler;

use agent::Agent;
use scheduler::AgentScheduler;

/// Agent management system
pub struct AgentManager {
    scheduler: AgentScheduler,
    agents: spin::Mutex<alloc::vec::Vec<Agent>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            scheduler: AgentScheduler::new(),
            agents: spin::Mutex::new(alloc::vec::Vec::new()),
        }
    }

    /// Spawn a new agent
    pub fn spawn_agent(&self, spec: AgentSpec) -> Result<u64, AgentError> {
        let agent = Agent::new(spec)?;
        let agent_id = agent.id();
        
        let mut agents = self.agents.lock();
        agents.push(agent);
        
        self.scheduler.add_agent(agent_id, 1024, 0); // Default weight and priority
        
        // Initialize AI scheduler if not already initialized
        #[cfg(feature = "alloc")]
        {
            use crate::ai_scheduler;
            if ai_scheduler::get().is_none() {
                ai_scheduler::init();
            }
        }
        
        Ok(agent_id)
    }

    /// Get agent by ID
    pub fn get_agent(&self, agent_id: u64) -> Option<Agent> {
        let agents = self.agents.lock();
        agents.iter().find(|a| a.id() == agent_id).cloned()
    }
}

pub struct AgentSpec {
    pub id: u64,
    pub capabilities: alloc::vec::Vec<u8>,
    pub memory_limit: usize,
    pub cpu_limit: u8,
}

#[derive(Debug)]
pub enum AgentError {
    InvalidSpec,
    ResourceExhausted,
    SecurityViolation,
}

