//! Network namespaces
//! 
//! Agent-isolated network namespaces

use crate::ip::IpAddress;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Network namespace
#[cfg(feature = "alloc")]
pub struct NetworkNamespace {
    pub namespace_id: u64,
    pub agent_id: u64,
    pub local_address: Option<IpAddress>,
    pub routes: Vec<u64>, // Route IDs
    pub firewall_rules: Vec<u64>, // Firewall rule IDs
}

/// Network namespace manager
pub struct NamespaceManager {
    #[cfg(feature = "alloc")]
    namespaces: Mutex<BTreeMap<u64, NetworkNamespace>>, // Agent ID -> Namespace
    #[cfg(feature = "alloc")]
    next_namespace_id: Mutex<u64>,
    #[cfg(not(feature = "alloc"))]
    namespaces: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_namespace_id: Mutex<()>,
}

static NAMESPACE_MANAGER: Mutex<Option<NamespaceManager>> = Mutex::new(None);

impl NamespaceManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            namespaces: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            next_namespace_id: Mutex::new(1),
            #[cfg(not(feature = "alloc"))]
            namespaces: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            next_namespace_id: Mutex::new(()),
        }
    }

    /// Initialize namespace manager
    pub fn init() {
        let mut manager = NAMESPACE_MANAGER.lock();
        *manager = Some(NamespaceManager::new());
    }

    /// Get namespace manager
    pub fn get() -> Option<&'static NamespaceManager> {
        let manager = NAMESPACE_MANAGER.lock();
        manager.as_ref()
    }

    /// Create namespace for agent
    #[cfg(feature = "alloc")]
    pub fn create_namespace(&self, agent_id: u64) -> u64 {
        let mut namespaces = self.namespaces.lock();
        let mut next_id = self.next_namespace_id.lock();
        let namespace_id = *next_id;
        *next_id += 1;

        let namespace = NetworkNamespace {
            namespace_id,
            agent_id,
            local_address: None,
            routes: Vec::new(),
            firewall_rules: Vec::new(),
        };

        namespaces.insert(agent_id, namespace);
        namespace_id
    }

    #[cfg(not(feature = "alloc"))]
    pub fn create_namespace(&self, _agent_id: u64) -> u64 {
        0
    }

    /// Get namespace for agent
    #[cfg(feature = "alloc")]
    pub fn get_namespace(&self, agent_id: u64) -> Option<NetworkNamespace> {
        let namespaces = self.namespaces.lock();
        namespaces.get(&agent_id).cloned()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_namespace(&self, _agent_id: u64) -> Option<NetworkNamespace> {
        None
    }

    /// Set local address for namespace
    #[cfg(feature = "alloc")]
    pub fn set_local_address(&self, agent_id: u64, addr: IpAddress) {
        let mut namespaces = self.namespaces.lock();
        if let Some(namespace) = namespaces.get_mut(&agent_id) {
            namespace.local_address = Some(addr);
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn set_local_address(&self, _agent_id: u64, _addr: IpAddress) {
        // No-op
    }

    /// Add route to namespace
    #[cfg(feature = "alloc")]
    pub fn add_route(&self, agent_id: u64, route_id: u64) {
        let mut namespaces = self.namespaces.lock();
        if let Some(namespace) = namespaces.get_mut(&agent_id) {
            if !namespace.routes.contains(&route_id) {
                namespace.routes.push(route_id);
            }
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn add_route(&self, _agent_id: u64, _route_id: u64) {
        // No-op
    }

    /// Add firewall rule to namespace
    #[cfg(feature = "alloc")]
    pub fn add_firewall_rule(&self, agent_id: u64, rule_id: u64) {
        let mut namespaces = self.namespaces.lock();
        if let Some(namespace) = namespaces.get_mut(&agent_id) {
            if !namespace.firewall_rules.contains(&rule_id) {
                namespace.firewall_rules.push(rule_id);
            }
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn add_firewall_rule(&self, _agent_id: u64, _rule_id: u64) {
        // No-op
    }

    /// Delete namespace
    #[cfg(feature = "alloc")]
    pub fn delete_namespace(&self, agent_id: u64) {
        let mut namespaces = self.namespaces.lock();
        namespaces.remove(&agent_id);
    }

    #[cfg(not(feature = "alloc"))]
    pub fn delete_namespace(&self, _agent_id: u64) {
        // No-op
    }
}

#[cfg(feature = "alloc")]
impl Clone for NetworkNamespace {
    fn clone(&self) -> Self {
        Self {
            namespace_id: self.namespace_id,
            agent_id: self.agent_id,
            local_address: self.local_address,
            routes: self.routes.clone(),
            firewall_rules: self.firewall_rules.clone(),
        }
    }
}

