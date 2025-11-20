//! Distributed IPC + Message Routing
//! 
//! Semantic messages for agents; scalable beyond single node

use super::message::IPCMessage;
use alloc::vec::Vec;
use spin::Mutex;

#[cfg(feature = "alloc")]
use aios_kernel_net::{create_agent_endpoint, send_data, recv_data, close_agent_endpoint};
#[cfg(feature = "alloc")]
use aios_kernel_net::ip::IpAddress;
#[cfg(feature = "alloc")]
use aios_kernel_net::socket::SocketType;
#[cfg(feature = "alloc")]
use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};

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

/// Agent location (local or remote)
#[derive(Clone, Copy, Debug)]
pub enum AgentLocation {
    Local,
    Remote { node_id: u64, address: IpAddress, port: u16 },
}

/// Pending message with retry info
#[cfg(feature = "alloc")]
struct PendingMessage {
    message: IPCMessage,
    retry_count: u32,
    next_retry: u64, // Timestamp
    retry_policy: RetryPolicy,
}

/// Distributed IPC router
pub struct DistributedIPCRouter {
    namespaces: Mutex<alloc::collections::BTreeMap<u64, MessageNamespace>>,
    routing_rules: Mutex<Vec<RoutingRule>>,
    message_queues: Mutex<alloc::collections::BTreeMap<(u64, MessagePriority), Vec<IPCMessage>>>,
    agent_locations: Mutex<alloc::collections::BTreeMap<u64, AgentLocation>>, // Agent ID -> Location
    network_sockets: Mutex<alloc::collections::BTreeMap<u64, u64>>, // Node ID -> Socket ID
    #[cfg(feature = "alloc")]
    pending_messages: Mutex<alloc::collections::BTreeMap<u64, PendingMessage>>, // Message ID -> Pending
    #[cfg(feature = "alloc")]
    message_acks: Mutex<alloc::collections::BTreeSet<u64>>, // Acknowledged message IDs
    next_namespace_id: Mutex<u64>,
}

impl DistributedIPCRouter {
    pub fn new() -> Self {
        Self {
            namespaces: Mutex::new(alloc::collections::BTreeMap::new()),
            routing_rules: Mutex::new(Vec::new()),
            message_queues: Mutex::new(alloc::collections::BTreeMap::new()),
            agent_locations: Mutex::new(alloc::collections::BTreeMap::new()),
            network_sockets: Mutex::new(alloc::collections::BTreeMap::new()),
            #[cfg(feature = "alloc")]
            pending_messages: Mutex::new(alloc::collections::BTreeMap::new()),
            #[cfg(feature = "alloc")]
            message_acks: Mutex::new(alloc::collections::BTreeSet::new()),
            next_namespace_id: Mutex::new(1),
        }
    }

    /// Route message (local or remote)
    pub fn route(&self, message: IPCMessage) -> Result<(), RoutingError> {
        // Check trust-based routing
        if !self.check_trust_routing(&message) {
            return Err(RoutingError::TrustViolation);
        }

        // Check if destination is local or remote
        let locations = self.agent_locations.lock();
        let location = locations.get(&message.to).copied().unwrap_or(AgentLocation::Local);

        match location {
            AgentLocation::Local => {
                // Route locally
                self.route_local(message)
            }
            AgentLocation::Remote { node_id, address, port } => {
                // Route over network
                drop(locations);
                self.route_remote(message, node_id, address, port)
            }
        }
    }

    /// Route message locally
    fn route_local(&self, message: IPCMessage) -> Result<(), RoutingError> {
        // Determine priority
        let priority = self.determine_priority(&message);

        // Add to priority queue
        let mut queues = self.message_queues.lock();
        let queue_key = (message.to, priority);
        let queue = queues.entry(queue_key).or_insert_with(Vec::new);
        queue.push(message);

        Ok(())
    }

    /// Route message over network
    #[cfg(feature = "alloc")]
    fn route_remote(&self, message: IPCMessage, node_id: u64, address: IpAddress, port: u16) -> Result<(), RoutingError> {
        // Get or create socket for this node
        let socket_id = self.get_or_create_socket(node_id, address, port)?;

        // Serialize message
        let serialized = self.serialize_message(&message);

        // Send over network with retry policy
        let retry_policy = RetryPolicy {
            max_retries: 3,
            retry_delay: 100_000_000, // 100ms
            exponential_backoff: true,
        };

        match send_data(socket_id, message.from, &serialized) {
            Ok(_) => {
                // Success - track if requires ACK
                if message.flags.contains(super::message::MessageFlags::REQUIRES_ACK) {
                    let mut pending = self.pending_messages.lock();
                    pending.insert(message.id, PendingMessage {
                        message,
                        retry_count: 0,
                        next_retry: message.timestamp + retry_policy.retry_delay,
                        retry_policy,
                    });
                }

                // Publish network event
                publish_event(KernelEvent {
                    event_type: EventType::Network,
                    timestamp: message.timestamp,
                    agent_id: Some(message.from),
                    data: alloc::vec::Vec::new(),
                });

                Ok(())
            }
            Err(_) => {
                // Failed - add to pending for retry
                let mut pending = self.pending_messages.lock();
                pending.insert(message.id, PendingMessage {
                    message,
                    retry_count: 0,
                    next_retry: message.timestamp + retry_policy.retry_delay,
                    retry_policy,
                });
                Err(RoutingError::NetworkError)
            }
        }
    }

    #[cfg(not(feature = "alloc"))]
    fn route_remote(&self, _message: IPCMessage, _node_id: u64, _address: IpAddress, _port: u16) -> Result<(), RoutingError> {
        Err(RoutingError::NetworkError)
    }

    /// Get or create socket for remote node
    #[cfg(feature = "alloc")]
    fn get_or_create_socket(&self, node_id: u64, address: IpAddress, port: u16) -> Result<u64, RoutingError> {
        let mut sockets = self.network_sockets.lock();
        
        if let Some(socket_id) = sockets.get(&node_id) {
            return Ok(*socket_id);
        }

        // Create new socket
        // Use agent ID 0 (system) for network sockets
        let socket_id = create_agent_endpoint(0, SocketType::Stream, None)
            .map_err(|_| RoutingError::NetworkError)?;

        // Connect to remote node
        // Connect socket to remote address and port
        use aios_kernel_net::socket;
        if let Some(mut sock) = socket::get_socket(socket_id) {
            let _ = sock.connect_agent_endpoint(address, port);
        }
        sockets.insert(node_id, socket_id);

        Ok(socket_id)
    }

    #[cfg(not(feature = "alloc"))]
    fn get_or_create_socket(&self, _node_id: u64, _address: IpAddress, _port: u16) -> Result<u64, RoutingError> {
        Err(RoutingError::NetworkError)
    }

    /// Serialize message for network transmission
    #[cfg(feature = "alloc")]
    fn serialize_message(&self, message: &IPCMessage) -> Vec<u8> {
        // Binary serialization format: [id(8)][from(8)][to(8)][data_len(8)][data][metadata_len(8)][metadata][flags(4)]
        let mut serialized = Vec::new();
        
        // Message ID
        serialized.extend_from_slice(&message.id.to_le_bytes());
        // From agent
        serialized.extend_from_slice(&message.from.to_le_bytes());
        // To agent
        serialized.extend_from_slice(&message.to.to_le_bytes());
        // Data length and data
        serialized.extend_from_slice(&(message.data.len() as u64).to_le_bytes());
        serialized.extend_from_slice(&message.data);
        // Metadata length and metadata
        serialized.extend_from_slice(&(message.metadata.len() as u64).to_le_bytes());
        serialized.extend_from_slice(&message.metadata);
        // Flags
        serialized.extend_from_slice(&message.flags.bits().to_le_bytes());
        
        serialized
    }

    #[cfg(not(feature = "alloc"))]
    fn serialize_message(&self, _message: &IPCMessage) -> Vec<u8> {
        Vec::new()
    }

    /// Handle incoming network message
    #[cfg(feature = "alloc")]
    pub fn handle_network_message(&self, data: &[u8], target_agent: u64) -> Result<(), RoutingError> {
        // Deserialize message
        let message = self.deserialize_message(data, target_agent)?;
        
        // Route locally
        self.route_local(message)
    }

    #[cfg(not(feature = "alloc"))]
    pub fn handle_network_message(&self, _data: &[u8], _target_agent: u64) -> Result<(), RoutingError> {
        Err(RoutingError::NetworkError)
    }

    /// Deserialize message from network
    #[cfg(feature = "alloc")]
    fn deserialize_message(&self, data: &[u8], target_agent: u64) -> Result<IPCMessage, RoutingError> {
        if data.len() < 8 + 8 + 8 + 8 + 8 + 4 {
            return Err(RoutingError::InvalidMessage);
        }

        let mut offset = 0;
        
        // Message ID
        let id = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7],
        ]);
        offset += 8;
        
        // From agent
        let from = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7],
        ]);
        offset += 8;
        
        // To agent (should match target_agent)
        let to = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7],
        ]);
        offset += 8;
        
        if to != target_agent {
            return Err(RoutingError::InvalidMessage);
        }
        
        // Data length
        let data_len = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7],
        ]) as usize;
        offset += 8;
        
        if offset + data_len > data.len() {
            return Err(RoutingError::InvalidMessage);
        }
        
        let message_data = data[offset..offset + data_len].to_vec();
        offset += data_len;
        
        // Metadata length
        if offset + 8 > data.len() {
            return Err(RoutingError::InvalidMessage);
        }
        
        let metadata_len = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7],
        ]) as usize;
        offset += 8;
        
        if offset + metadata_len + 4 > data.len() {
            return Err(RoutingError::InvalidMessage);
        }
        
        let metadata = data[offset..offset + metadata_len].to_vec();
        offset += metadata_len;
        
        // Flags
        let flags_bits = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
        ]);
        let flags = super::message::MessageFlags::from_bits(flags_bits)
            .unwrap_or(super::message::MessageFlags::empty());
        
        Ok(IPCMessage {
            id,
            from,
            to,
            data: message_data,
            metadata,
            timestamp: message.timestamp,
            flags,
        })
    }

    #[cfg(not(feature = "alloc"))]
    fn deserialize_message(&self, _data: &[u8], _target_agent: u64) -> Result<IPCMessage, RoutingError> {
        Err(RoutingError::NetworkError)
    }

    /// Register agent location
    pub fn register_agent_location(&self, agent_id: u64, location: AgentLocation) {
        let mut locations = self.agent_locations.lock();
        locations.insert(agent_id, location);
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
    fn determine_priority(&self, message: &IPCMessage) -> MessagePriority {
        // Check message flags for high priority
        if message.flags.contains(super::message::MessageFlags::HIGH_PRIORITY) {
            return MessagePriority::High;
        }
        
        // Parse metadata for priority hints
        // Metadata may contain priority information
        if message.metadata.len() >= 1 {
            let priority_byte = message.metadata[0];
            match priority_byte {
                1 => return MessagePriority::Low,
                2 => return MessagePriority::Normal,
                3 => return MessagePriority::High,
                4 => return MessagePriority::Critical,
                _ => {}
            }
        }
        MessagePriority::Normal
    }

    /// Process pending retries
    /// 
    /// Note: This requires the caller to provide current time to avoid circular dependency.
    /// For now, uses message timestamps for comparison.
    #[cfg(feature = "alloc")]
    pub fn process_retries(&self, current_time: u64) {
        let mut pending = self.pending_messages.lock();
        let mut to_retry = Vec::new();
        
        // Collect messages that need retry
        for (message_id, pending_msg) in pending.iter() {
            if pending_msg.next_retry <= current_time && pending_msg.retry_count < pending_msg.retry_policy.max_retries {
                to_retry.push(*message_id);
            }
        }
        
        // Retry messages
        for message_id in to_retry {
            if let Some(mut pending_msg) = pending.remove(&message_id) {
                // Get agent location
                let locations = self.agent_locations.lock();
                let location = locations.get(&pending_msg.message.to).copied().unwrap_or(AgentLocation::Local);
                drop(locations);
                
                if let AgentLocation::Remote { node_id, address, port } = location {
                    // Retry sending
                    let socket_id = match self.get_or_create_socket(node_id, address, port) {
                        Ok(id) => id,
                        Err(_) => {
                            // Still failed - put back with updated retry info
                            pending_msg.retry_count += 1;
                            let delay = if pending_msg.retry_policy.exponential_backoff {
                                pending_msg.retry_policy.retry_delay * (2_u64.pow(pending_msg.retry_count))
                            } else {
                                pending_msg.retry_policy.retry_delay
                            };
                            pending_msg.next_retry = now + delay;
                            pending.insert(message_id, pending_msg);
                            continue;
                        }
                    };
                    
                    let serialized = self.serialize_message(&pending_msg.message);
                    match send_data(socket_id, pending_msg.message.from, &serialized) {
                        Ok(_) => {
                            // Success - remove from pending if no ACK required
                            if !pending_msg.message.flags.contains(super::message::MessageFlags::REQUIRES_ACK) {
                                // Message sent successfully, no need to track
                            } else {
                                // Still need ACK - reset retry count
                                pending_msg.retry_count = 0;
                                pending_msg.next_retry = now + pending_msg.retry_policy.retry_delay;
                                pending.insert(message_id, pending_msg);
                            }
                        }
                        Err(_) => {
                            // Still failed - update retry info
                            pending_msg.retry_count += 1;
                            let delay = if pending_msg.retry_policy.exponential_backoff {
                                pending_msg.retry_policy.retry_delay * (2_u64.pow(pending_msg.retry_count))
                            } else {
                                pending_msg.retry_policy.retry_delay
                            };
                            pending_msg.next_retry = now + delay;
                            pending.insert(message_id, pending_msg);
                        }
                    }
                }
            }
        }
    }

    /// Acknowledge message receipt
    #[cfg(feature = "alloc")]
    pub fn acknowledge_message(&self, message_id: u64) {
        let mut acks = self.message_acks.lock();
        acks.insert(message_id);
        
        // Remove from pending
        let mut pending = self.pending_messages.lock();
        pending.remove(&message_id);
    }

    /// Get next message from priority queue
    #[cfg(feature = "alloc")]
    pub fn get_next_message(&self, agent_id: u64) -> Option<IPCMessage> {
        let mut queues = self.message_queues.lock();
        
        // Try priorities in order: Critical, High, Normal, Low
        let priorities = [
            MessagePriority::Critical,
            MessagePriority::High,
            MessagePriority::Normal,
            MessagePriority::Low,
        ];
        
        for priority in priorities.iter() {
            let queue_key = (agent_id, *priority);
            if let Some(queue) = queues.get_mut(&queue_key) {
                if !queue.is_empty() {
                    return Some(queue.remove(0));
                }
            }
        }
        
        None
    }

    /// Add routing rule
    pub fn add_routing_rule(&self, rule: RoutingRule) {
        let mut rules = self.routing_rules.lock();
        rules.push(rule);
    }

    /// Create namespace
    pub fn create_namespace(&self, name: alloc::string::String) -> u64 {
        let mut next_id = self.next_namespace_id.lock();
        let namespace_id = *next_id;
        *next_id = next_id.wrapping_add(1);
        
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
    NetworkError,
    InvalidMessage,
}

