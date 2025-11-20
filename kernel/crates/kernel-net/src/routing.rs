//! Network routing
//! 
//! Routing tables, route management, and packet routing

use crate::ip::IpAddress;
use spin::Mutex;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Route entry
#[cfg(feature = "alloc")]
pub struct RouteEntry {
    pub destination: IpAddress,
    pub gateway: Option<IpAddress>,
    pub netmask: IpAddress,
    pub interface: u64, // Interface ID
    pub metric: u32,
    pub agent_id: Option<u64>, // Agent-specific route
}

/// Routing table
pub struct RoutingTable {
    #[cfg(feature = "alloc")]
    routes: Mutex<BTreeMap<u64, RouteEntry>>, // Route ID -> Route
    #[cfg(feature = "alloc")]
    default_gateway: Mutex<Option<IpAddress>>,
    #[cfg(feature = "alloc")]
    next_route_id: Mutex<u64>,
    #[cfg(not(feature = "alloc"))]
    routes: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    default_gateway: Mutex<()>,
    #[cfg(not(feature = "alloc"))]
    next_route_id: Mutex<()>,
}

static ROUTING_TABLE: Mutex<Option<RoutingTable>> = Mutex::new(None);

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            routes: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "alloc")]
            default_gateway: Mutex::new(None),
            #[cfg(feature = "alloc")]
            next_route_id: Mutex::new(1),
            #[cfg(not(feature = "alloc"))]
            routes: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            default_gateway: Mutex::new(()),
            #[cfg(not(feature = "alloc"))]
            next_route_id: Mutex::new(()),
        }
    }

    /// Initialize routing table
    pub fn init() {
        let mut table = ROUTING_TABLE.lock();
        *table = Some(RoutingTable::new());
    }

    /// Get routing table
    pub fn get() -> Option<&'static RoutingTable> {
        let table = ROUTING_TABLE.lock();
        table.as_ref()
    }

    /// Add route
    #[cfg(feature = "alloc")]
    pub fn add_route(&self, route: RouteEntry) -> u64 {
        let mut routes = self.routes.lock();
        let mut next_id = self.next_route_id.lock();
        let route_id = *next_id;
        *next_id += 1;
        routes.insert(route_id, route);
        route_id
    }

    #[cfg(not(feature = "alloc"))]
    pub fn add_route(&self, _route: RouteEntry) -> u64 {
        0
    }

    /// Remove route
    #[cfg(feature = "alloc")]
    pub fn remove_route(&self, route_id: u64) -> bool {
        let mut routes = self.routes.lock();
        routes.remove(&route_id).is_some()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn remove_route(&self, _route_id: u64) -> bool {
        false
    }

    /// Find route for destination
    #[cfg(feature = "alloc")]
    pub fn find_route(&self, dst: IpAddress, agent_id: Option<u64>) -> Option<(IpAddress, u64)> {
        let routes = self.routes.lock();
        let mut best_route: Option<(&RouteEntry, u64)> = None;
        let mut best_match_bits = 0;

        for (route_id, route) in routes.iter() {
            // Check if route is agent-specific
            if let Some(route_agent_id) = route.agent_id {
                if agent_id != Some(route_agent_id) {
                    continue;
                }
            }

            // Check if destination matches route
            let match_bits = self.match_route(dst, route.destination, route.netmask);
            if match_bits > best_match_bits || (match_bits == best_match_bits && route.metric < best_route.map(|(r, _)| r.metric).unwrap_or(u32::MAX)) {
                best_match_bits = match_bits;
                best_route = Some((route, *route_id));
            }
        }

        best_route.map(|(route, _)| {
            (route.gateway.unwrap_or(route.destination), route.interface)
        })
    }

    #[cfg(not(feature = "alloc"))]
    pub fn find_route(&self, _dst: IpAddress, _agent_id: Option<u64>) -> Option<(IpAddress, u64)> {
        None
    }

    /// Match route (returns number of matching bits)
    #[cfg(feature = "alloc")]
    fn match_route(&self, addr: IpAddress, dest: IpAddress, netmask: IpAddress) -> u32 {
        let mut match_bits = 0;
        for i in 0..4 {
            let masked = addr.octets[i] & netmask.octets[i];
            let dest_masked = dest.octets[i] & netmask.octets[i];
            if masked == dest_masked {
                match_bits += 8;
            } else {
                // Count matching bits in this octet
                let diff = masked ^ dest_masked;
                for bit in 0..8 {
                    if (diff >> (7 - bit)) & 1 == 0 {
                        match_bits += 1;
                    } else {
                        break;
                    }
                }
                break;
            }
        }
        match_bits
    }

    #[cfg(not(feature = "alloc"))]
    fn match_route(&self, _addr: IpAddress, _dest: IpAddress, _netmask: IpAddress) -> u32 {
        0
    }

    /// Set default gateway
    #[cfg(feature = "alloc")]
    pub fn set_default_gateway(&self, gateway: IpAddress) {
        let mut default = self.default_gateway.lock();
        *default = Some(gateway);
    }

    #[cfg(not(feature = "alloc"))]
    pub fn set_default_gateway(&self, _gateway: IpAddress) {
        // No-op
    }

    /// Get default gateway
    #[cfg(feature = "alloc")]
    pub fn get_default_gateway(&self) -> Option<IpAddress> {
        let default = self.default_gateway.lock();
        *default
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_default_gateway(&self) -> Option<IpAddress> {
        None
    }
}

