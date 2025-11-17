//! Distributed IPC + Message Routing
//! 
//! Semantic messages for agents; scalable beyond single node

use super::message::IPCMessage;
use alloc::vec::Vec;
use spin::Mutex;

/// Message namespace
pub struct MessageNamespace {
    pub namespace_id: u64,
    pub name: alloc::string::String,
    pub agents: Vec<u64>,
}

/// Trust-based routing rule
pub struct RoutingRule {
    pub from_agent: u64,
    pub to_agent: u64,
    pub trust_level: f64, // 0.0 to 1.0
    pub capabilities: Vec<u8>,
}

/// Message priority
#[repr(u32)]
pub enum MessagePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Retry policy
pub struct RetryPolicy {
    pub max_retries: u32,
    pub retry_delay: u64, // nanoseconds
    pub exponential_backoff: bool,
}

/// Distributed IPC router
pub struct DistributedIPCRouter {
    namespaces: Mutex<alloc::collections::BTreeMap<u64, MessageNamespace>>,
    routing_rules: Mutex<Vec<RoutingRule>>,
    message_queues: Mutex<alloc::collections::BTreeMap<(u64, MessagePriority), Vec<IPCMessage>>>,
}

impl DistributedIPCRouter {
    pub fn new() -> Self {
        Self {
            namespaces: Mutex::new(alloc::collections::BTreeMap::new()),
            routing_rules: Mutex::new(Vec::new()),
            message_queues: Mutex::new(alloc::collections::BTreeMap::new()),
        }
    }

    /// Route message
    pub fn route(&self, message: IPCMessage) -> Result<(), RoutingError> {
        // Check trust-based routing
        if !self.check_trust_routing(&message) {
            return Err(RoutingError::TrustViolation);
        }

        // Determine priority
        let priority = self.determine_priority(&message);

        // Add to priority queue
        let mut queues = self.message_queues.lock();
        let queue_key = (message.to, priority);
        let queue = queues.entry(queue_key).or_insert_with(Vec::new);
        queue.push(message);

        Ok(())
    }

    /// Check trust-based routing
    fn check_trust_routing(&self, message: &IPCMessage) -> bool {
        let rules = self.routing_rules.lock();
        for rule in rules.iter() {
            if rule.from_agent == message.from && rule.to_agent == message.to {
                // Check trust level
                if rule.trust_level < 0.5 {
                    return false;
                }
                return true;
            }
        }
        // Default: allow if no rule
        true
    }

    /// Determine message priority
    fn determine_priority(&self, _message: &IPCMessage) -> MessagePriority {
        // TODO: Determine from message metadata
        MessagePriority::Normal
    }

    /// Add routing rule
    pub fn add_routing_rule(&self, rule: RoutingRule) {
        let mut rules = self.routing_rules.lock();
        rules.push(rule);
    }

    /// Create namespace
    pub fn create_namespace(&self, name: alloc::string::String) -> u64 {
        let namespace_id = 1; // TODO: Generate unique ID
        let namespace = MessageNamespace {
            namespace_id,
            name,
            agents: Vec::new(),
        };

        let mut namespaces = self.namespaces.lock();
        namespaces.insert(namespace_id, namespace);

        namespace_id
    }

    /// Add agent to namespace
    pub fn add_agent_to_namespace(&self, namespace_id: u64, agent_id: u64) {
        let mut namespaces = self.namespaces.lock();
        if let Some(namespace) = namespaces.get_mut(&namespace_id) {
            namespace.agents.push(agent_id);
        }
    }
}

#[derive(Debug)]
pub enum RoutingError {
    TrustViolation,
    NamespaceNotFound,
    AgentNotFound,
    QueueFull,
}

