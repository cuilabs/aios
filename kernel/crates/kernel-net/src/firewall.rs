//! Network firewall
//! 
//! Firewall rules and packet filtering

use crate::ip::IpAddress;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Firewall rule action
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FirewallAction {
    Allow,
    Deny,
    Reject,
}

/// Firewall rule direction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FirewallDirection {
    Inbound,
    Outbound,
    Both,
}

/// Firewall rule
#[cfg(feature = "alloc")]
pub struct FirewallRule {
    pub rule_id: u64,
    pub agent_id: Option<u64>, // Agent-specific rule
    pub direction: FirewallDirection,
    pub src_addr: Option<IpAddress>,
    pub src_mask: Option<IpAddress>,
    pub dst_addr: Option<IpAddress>,
    pub dst_mask: Option<IpAddress>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: Option<u8>, // TCP=6, UDP=17, etc.
    pub action: FirewallAction,
    pub priority: u32, // Lower = higher priority
}

/// Firewall manager
pub struct FirewallManager {
    #[cfg(feature = "alloc")]
    rules: Mutex<BTreeMap<u64, FirewallRule>>, // Rule ID -> Rule
    #[cfg(feature = "alloc")]
    next_rule_id: Mutex<u64>,
    #[cfg(not(feature = "alloc"))]
    rules: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_rule_id: Mutex<()>,
}

static FIREWALL_MANAGER: Mutex<Option<FirewallManager>> = Mutex::new(None);

impl FirewallManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            rules: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            next_rule_id: Mutex::new(1),
            #[cfg(not(feature = "alloc"))]
            rules: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            next_rule_id: Mutex::new(()),
        }
    }

    /// Initialize firewall manager
    pub fn init() {
        let mut manager = FIREWALL_MANAGER.lock();
        *manager = Some(FirewallManager::new());
    }

    /// Get firewall manager
    pub fn get() -> Option<&'static FirewallManager> {
        let manager = FIREWALL_MANAGER.lock();
        manager.as_ref()
    }

    /// Add firewall rule
    #[cfg(feature = "alloc")]
    pub fn add_rule(&self, mut rule: FirewallRule) -> u64 {
        let mut rules = self.rules.lock();
        let mut next_id = self.next_rule_id.lock();
        let rule_id = *next_id;
        *next_id += 1;
        rule.rule_id = rule_id;
        rules.insert(rule_id, rule);
        rule_id
    }

    #[cfg(not(feature = "alloc"))]
    pub fn add_rule(&self, _rule: FirewallRule) -> u64 {
        0
    }

    /// Remove firewall rule
    #[cfg(feature = "alloc")]
    pub fn remove_rule(&self, rule_id: u64) -> bool {
        let mut rules = self.rules.lock();
        rules.remove(&rule_id).is_some()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn remove_rule(&self, _rule_id: u64) -> bool {
        false
    }

    /// Check if packet is allowed
    #[cfg(feature = "alloc")]
    pub fn check_packet(
        &self,
        direction: FirewallDirection,
        src_addr: IpAddress,
        dst_addr: IpAddress,
        src_port: Option<u16>,
        dst_port: Option<u16>,
        protocol: Option<u8>,
        agent_id: Option<u64>,
    ) -> bool {
        let rules = self.rules.lock();
        
        // Sort rules by priority (lower priority number = higher priority)
        let mut sorted_rules: Vec<(&FirewallRule, u64)> = rules.iter()
            .map(|(id, rule)| (rule, *id))
            .collect();
        sorted_rules.sort_by_key(|(rule, _)| rule.priority);

        // Check rules in priority order
        for (rule, _) in sorted_rules {
            // Check if rule applies to this agent
            if let Some(rule_agent_id) = rule.agent_id {
                if agent_id != Some(rule_agent_id) {
                    continue;
                }
            }

            // Check direction
            if !matches!(rule.direction, FirewallDirection::Both) && rule.direction != direction {
                continue;
            }

            // Check source address
            if let Some(rule_src) = rule.src_addr {
                if let Some(rule_mask) = rule.src_mask {
                    if !self.match_address(src_addr, rule_src, rule_mask) {
                        continue;
                    }
                } else if src_addr != rule_src {
                    continue;
                }
            }

            // Check destination address
            if let Some(rule_dst) = rule.dst_addr {
                if let Some(rule_mask) = rule.dst_mask {
                    if !self.match_address(dst_addr, rule_dst, rule_mask) {
                        continue;
                    }
                } else if dst_addr != rule_dst {
                    continue;
                }
            }

            // Check ports
            if let Some(rule_src_port) = rule.src_port {
                if src_port != Some(rule_src_port) {
                    continue;
                }
            }
            if let Some(rule_dst_port) = rule.dst_port {
                if dst_port != Some(rule_dst_port) {
                    continue;
                }
            }

            // Check protocol
            if let Some(rule_protocol) = rule.protocol {
                if protocol != Some(rule_protocol) {
                    continue;
                }
            }

            // Rule matches - apply action
            return matches!(rule.action, FirewallAction::Allow);
        }

        // Default: deny if no rule matches
        false
    }

    #[cfg(not(feature = "alloc"))]
    pub fn check_packet(
        &self,
        _direction: FirewallDirection,
        _src_addr: IpAddress,
        _dst_addr: IpAddress,
        _src_port: Option<u16>,
        _dst_port: Option<u16>,
        _protocol: Option<u8>,
        _agent_id: Option<u64>,
    ) -> bool {
        false
    }

    /// Match address against rule
    fn match_address(&self, addr: IpAddress, rule_addr: IpAddress, mask: IpAddress) -> bool {
        for i in 0..4 {
            if (addr.octets[i] & mask.octets[i]) != (rule_addr.octets[i] & mask.octets[i]) {
                return false;
            }
        }
        true
    }
}

