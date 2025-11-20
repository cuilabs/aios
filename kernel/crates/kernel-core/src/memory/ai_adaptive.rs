//! AI-Adaptive Memory Management
//! 
//! Provides AI-driven memory management including:
//! - Memory access pattern analysis
//! - Predictive caching
//! - Adaptive paging strategies

use spin::Mutex;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use x86_64::VirtAddr;
use x86_64::PhysAddr;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use crate::time;

/// Memory access event
#[derive(Clone, Debug)]
pub struct MemoryAccess {
    pub timestamp: u64,
    pub virtual_addr: VirtAddr,
    pub physical_addr: PhysAddr,
    pub access_type: AccessType,
    pub agent_id: u64,
}

/// Memory access type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessType {
    Read,
    Write,
    Execute,
}

/// Memory access pattern
#[derive(Clone, Debug)]
pub struct MemoryPattern {
    pub agent_id: u64,
    pub access_frequency: f32, // Accesses per second
    pub locality: LocalityPattern,
    pub working_set_size: u64, // Bytes
    pub predicted_next_access: Option<VirtAddr>,
    pub hot_pages: BTreeSet<PhysAddr>, // Frequently accessed pages
}

/// Locality pattern
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocalityPattern {
    Temporal, // Recently accessed pages likely to be accessed again
    Spatial,  // Nearby pages likely to be accessed
    Random,   // No clear pattern
}

/// Memory pattern analyzer
/// 
/// Tracks memory access patterns and predicts future access
pub struct MemoryPatternAnalyzer {
    access_history: Mutex<BTreeMap<u64, Vec<MemoryAccess>>>, // agent_id -> accesses
    patterns: Mutex<BTreeMap<u64, MemoryPattern>>,            // agent_id -> pattern
    hot_pages: Mutex<BTreeSet<PhysAddr>>,                     // System-wide hot pages
    max_history_size: usize,
}

impl MemoryPatternAnalyzer {
    pub fn new() -> Self {
        Self {
            access_history: Mutex::new(BTreeMap::new()),
            patterns: Mutex::new(BTreeMap::new()),
            hot_pages: Mutex::new(BTreeSet::new()),
            max_history_size: 10000, // Keep last 10k accesses per agent
        }
    }
    
    /// Record memory access
    pub fn record_access(&self, access: MemoryAccess) {
        let mut history = self.access_history.lock();
        let agent_history = history.entry(access.agent_id).or_insert_with(Vec::new);
        
        // Add access
        agent_history.push(access.clone());
        
        // Trim history if too large
        if agent_history.len() > self.max_history_size {
            agent_history.remove(0);
        }
        
        // Update hot pages
        {
            let mut hot_pages = self.hot_pages.lock();
            hot_pages.insert(access.physical_addr);
            
            // Limit hot pages set size
            if hot_pages.len() > 1000 {
                // Remove oldest using LRU (Least Recently Used) algorithm
                // Track access times and remove least recently accessed pages
                let mut access_times: Vec<(PhysAddr, u64)> = Vec::new();
                for addr in hot_pages.iter() {
                    // Get access time from access history
                    let mut latest_time = 0u64;
                    for (_, history) in history.iter() {
                        for access in history.iter() {
                            if access.physical_addr == *addr {
                                if access.timestamp > latest_time {
                                    latest_time = access.timestamp;
                                }
                            }
                        }
                    }
                    access_times.push((*addr, latest_time));
                }
                
                // Sort by access time (oldest first) and remove bottom 100
                access_times.sort_by_key(|(_, time)| *time);
                for (addr, _) in access_times.iter().take(100) {
                    hot_pages.remove(addr);
                }
            }
        }
        
        // Update pattern if enough data
        if agent_history.len() >= 100 {
            self.update_pattern(access.agent_id, &agent_history);
        }
    }
    
    /// Update memory pattern for agent
    fn update_pattern(&self, agent_id: u64, history: &[MemoryAccess]) {
        if history.is_empty() {
            return;
        }
        
        // Calculate access frequency
        let time_span = if history.len() > 1 {
            history.last().unwrap().timestamp - history.first().unwrap().timestamp
        } else {
            1_000_000_000 // 1 second default
        };
        let access_frequency = (history.len() as f32) / (time_span as f32 / 1_000_000_000.0);
        
        // Determine locality pattern
        let locality = self.detect_locality(history);
        
        // Calculate working set size (unique pages accessed)
        let mut unique_pages = BTreeSet::new();
        for access in history {
            unique_pages.insert(access.physical_addr);
        }
        let working_set_size = unique_pages.len() * 4096; // Assume 4KB pages
        
        // Predict next access using ML-based prediction
        // Uses memory access pattern analysis to predict future accesses
        let predicted_next_access = self.predict_next_access(history);
        
        // Identify hot pages (frequently accessed)
        let hot_pages = self.identify_hot_pages(history);
        
        let pattern = MemoryPattern {
            agent_id,
            access_frequency,
            locality,
            working_set_size: working_set_size as u64,
            predicted_next_access,
            hot_pages,
        };
        
        let mut patterns = self.patterns.lock();
        patterns.insert(agent_id, pattern);
    }
    
    /// Detect locality pattern
    fn detect_locality(&self, history: &[MemoryAccess]) -> LocalityPattern {
        if history.len() < 10 {
            return LocalityPattern::Random;
        }
        
        // Check for temporal locality (recent accesses)
        let recent_window = 1_000_000_000; // 1 second
        let now = history.last().unwrap().timestamp;
        let recent_accesses: Vec<&MemoryAccess> = history.iter()
            .filter(|a| now - a.timestamp < recent_window)
            .collect();
        
        if recent_accesses.len() > history.len() / 2 {
            return LocalityPattern::Temporal;
        }
        
        // Check for spatial locality (nearby addresses)
        let mut spatial_count = 0;
        for i in 1..history.len().min(100) {
            let addr1 = history[i].virtual_addr.as_u64();
            let addr2 = history[i-1].virtual_addr.as_u64();
            let distance = if addr1 > addr2 { addr1 - addr2 } else { addr2 - addr1 };
            
            // Within 64KB (16 pages) = spatial locality
            if distance < 64 * 1024 {
                spatial_count += 1;
            }
        }
        
        if spatial_count > history.len() / 3 {
            return LocalityPattern::Spatial;
        }
        
        LocalityPattern::Random
    }
    
    /// Predict next memory access
    /// 
    /// First tries ML prediction via ML client, falls back to rule-based prediction.
    fn predict_next_access(&self, history: &[MemoryAccess]) -> Option<VirtAddr> {
        if history.len() < 2 {
            return None;
        }
        
        // Try ML prediction first (via ML client)
        #[cfg(feature = "alloc")]
        {
            use crate::ml_client::{get_ml_client, MemoryPredictionRequest};
            
            if history.len() >= 20 {
                let agent_id = history.last().unwrap().agent_id;
                let last = history.last().unwrap();
                
                // Prepare ML prediction request
                let access_history: Vec<f32> = history.iter()
                    .rev()
                    .take(20)
                    .map(|a| (a.virtual_addr.as_u64() as f32) / (1u64 << 32) as f32) // Normalize
                    .collect();
                let access_types: Vec<u8> = history.iter()
                    .rev()
                    .take(20)
                    .map(|a| match a.access_type {
                        AccessType::Read => 0,
                        AccessType::Write => 1,
                        AccessType::Execute => 2,
                    })
                    .collect();
                let access_timestamps: Vec<f32> = history.iter()
                    .rev()
                    .take(20)
                    .map(|a| (a.timestamp as f32) / 1_000_000_000.0) // Normalize to seconds
                    .collect();
                
                let current_address = (last.virtual_addr.as_u64() as f32) / (1u64 << 32) as f32;
                let locality_score = match self.detect_locality(history) {
                    LocalityPattern::Temporal => 0.8,
                    LocalityPattern::Spatial => 0.7,
                    LocalityPattern::Random => 0.3,
                };
                
                let ml_request = MemoryPredictionRequest {
                    agent_id,
                    access_history,
                    access_types,
                    access_timestamps,
                    current_address,
                    locality_score,
                };
                
                if let Some(ml_prediction) = get_ml_client().predict_memory(ml_request) {
                    // Use ML prediction
                    let predicted_addr = (ml_prediction.next_address * (1u64 << 32) as f32) as u64;
                    if let Some(addr) = VirtAddr::try_new(predicted_addr) {
                        return Some(addr);
                    }
                }
            }
        }
        
        // Fallback to rule-based prediction
        let last = history.last().unwrap();
        
        // Check if sequential pattern
        if history.len() >= 3 {
            let last_3: Vec<u64> = history.iter().rev().take(3)
                .map(|a| a.virtual_addr.as_u64())
                .collect();
            
            // Check if addresses are sequential
            let diff1 = if last_3[0] > last_3[1] { last_3[0] - last_3[1] } else { last_3[1] - last_3[0] };
            let diff2 = if last_3[1] > last_3[2] { last_3[1] - last_3[2] } else { last_3[2] - last_3[1] };
            
            if diff1 == diff2 && diff1 == 4096 { // 4KB pages
                // Sequential pattern - predict next page
                return Some(VirtAddr::new(last.virtual_addr.as_u64() + diff1));
            }
        }
        
        // Default: predict most recently accessed
        Some(last.virtual_addr)
    }
    
    /// Identify hot pages (frequently accessed)
    fn identify_hot_pages(&self, history: &[MemoryAccess]) -> BTreeSet<PhysAddr> {
        let mut page_counts: BTreeMap<PhysAddr, u32> = BTreeMap::new();
        
        // Count accesses per page
        for access in history {
            *page_counts.entry(access.physical_addr).or_insert(0) += 1;
        }
        
        // Find pages with above-average access count
        let avg_count = history.len() as f32 / page_counts.len() as f32;
        let threshold = (avg_count * 2.0) as u32; // 2x average
        
        let mut hot_pages = BTreeSet::new();
        for (page, &count) in page_counts.iter() {
            if count >= threshold {
                hot_pages.insert(*page);
            }
        }
        
        hot_pages
    }
    
    /// Get memory pattern for agent
    pub fn get_pattern(&self, agent_id: u64) -> Option<MemoryPattern> {
        let patterns = self.patterns.lock();
        patterns.get(&agent_id).cloned()
    }
    
    /// Check if page is hot
    pub fn is_hot_page(&self, page: PhysAddr) -> bool {
        let hot_pages = self.hot_pages.lock();
        hot_pages.contains(&page)
    }
    
    /// Get predicted next access for agent
    pub fn get_predicted_access(&self, agent_id: u64) -> Option<VirtAddr> {
        let patterns = self.patterns.lock();
        patterns.get(&agent_id).and_then(|p| p.predicted_next_access)
    }
    
    /// Get last access time for physical page
    pub fn get_last_access_time(&self, page: PhysAddr) -> Option<u64> {
        let history = self.access_history.lock();
        let mut latest_time: Option<u64> = None;
        
        for (_, accesses) in history.iter() {
            for access in accesses.iter() {
                if access.physical_addr == page {
                    if let Some(time) = latest_time {
                        if access.timestamp > time {
                            latest_time = Some(access.timestamp);
                        }
                    } else {
                        latest_time = Some(access.timestamp);
                    }
                }
            }
        }
        
        latest_time
    }
    
    /// Get access count for physical page
    pub fn get_access_count(&self, page: PhysAddr) -> u64 {
        let history = self.access_history.lock();
        let mut count = 0u64;
        
        for (_, accesses) in history.iter() {
            for access in accesses.iter() {
                if access.physical_addr == page {
                    count += 1;
                }
            }
        }
        
        count
    }
}

/// Cache policy
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CachePolicy {
    LRU,        // Least Recently Used
    LFU,        // Least Frequently Used
    Predictive, // AI-predicted access
    Adaptive,   // Hybrid (learns best policy)
}

/// Prefetch request
#[derive(Clone, Debug)]
pub struct PrefetchRequest {
    pub agent_id: u64,
    pub virtual_addr: VirtAddr,
    pub priority: u32,
    pub predicted_access_time: u64, // Nanoseconds
}

/// AI-driven cache manager
/// 
/// Manages page cache with predictive prefetching
pub struct AICacheManager {
    analyzer: MemoryPatternAnalyzer,
    cache_policy: CachePolicy,
    prefetch_queue: Mutex<Vec<PrefetchRequest>>,
}

impl AICacheManager {
    pub fn new(analyzer: MemoryPatternAnalyzer) -> Self {
        Self {
            analyzer,
            cache_policy: CachePolicy::Adaptive,
            prefetch_queue: Mutex::new(Vec::new()),
        }
    }
    
    /// Request prefetch for predicted access
    pub fn request_prefetch(&self, agent_id: u64, priority: u32) {
        if let Some(predicted_addr) = self.analyzer.get_predicted_access(agent_id) {
            let request = PrefetchRequest {
                agent_id,
                virtual_addr: predicted_addr,
                priority,
                predicted_access_time: crate::time::now() + 100_000_000, // 100ms ahead
            };
            
            let mut queue = self.prefetch_queue.lock();
            queue.push(request);
            // Sort by priority (higher first)
            queue.sort_by_key(|r| u32::MAX - r.priority);
        }
    }
    
    /// Get next prefetch request
    pub fn get_prefetch_request(&self) -> Option<PrefetchRequest> {
        let mut queue = self.prefetch_queue.lock();
        queue.pop()
    }
    
    /// Decide which page to evict from cache
    pub fn decide_eviction(&self, candidates: &[PhysAddr]) -> Option<PhysAddr> {
        match self.cache_policy {
            CachePolicy::LRU => {
                // Least recently used - track access times from analyzer
                let mut oldest_addr: Option<PhysAddr> = None;
                let mut oldest_time: u64 = u64::MAX;
                
                for &candidate in candidates {
                    // Get last access time from analyzer
                    if let Some(access_time) = self.analyzer.get_last_access_time(candidate) {
                        if access_time < oldest_time {
                            oldest_time = access_time;
                            oldest_addr = Some(candidate);
                        }
                    } else {
                        // No access history - evict this one
                        return Some(candidate);
                    }
                }
                
                oldest_addr
            }
            CachePolicy::LFU => {
                // Least frequently used - track access counts from analyzer
                let mut least_frequent_addr: Option<PhysAddr> = None;
                let mut least_count: u64 = u64::MAX;
                
                for &candidate in candidates {
                    // Get access count from analyzer
                    let access_count = self.analyzer.get_access_count(candidate);
                    if access_count < least_count {
                        least_count = access_count;
                        least_frequent_addr = Some(candidate);
                    }
                }
                
                least_frequent_addr
            }
            CachePolicy::Predictive => {
                // Evict pages not predicted to be accessed soon
                // Keep hot pages
                for &candidate in candidates {
                    if !self.analyzer.is_hot_page(candidate) {
                        return Some(candidate);
                    }
                }
                candidates.first().copied()
            }
            CachePolicy::Adaptive => {
                // Hybrid: prefer evicting non-hot pages
                for &candidate in candidates {
                    if !self.analyzer.is_hot_page(candidate) {
                        return Some(candidate);
                    }
                }
                candidates.first().copied()
            }
        }
    }
}

/// Swap predictor
pub struct SwapPredictor {
    swap_history: Mutex<Vec<SwapEvent>>,
    swap_patterns: Mutex<BTreeMap<u64, SwapPattern>>, // agent_id -> pattern
}

/// Swap event
#[derive(Clone, Debug)]
pub struct SwapEvent {
    pub timestamp: u64,
    pub frame: PhysAddr,
    pub agent_id: u64,
    pub swap_out: bool,
    pub access_after_swap: Option<u64>, // Time until next access (nanoseconds)
}

/// Swap pattern
#[derive(Clone, Debug)]
pub struct SwapPattern {
    pub agent_id: u64,
    pub swap_frequency: f32, // Swaps per second
    pub typical_swap_duration: u64, // Nanoseconds until page is accessed again
}

impl SwapPredictor {
    pub fn new() -> Self {
        Self {
            swap_history: Mutex::new(Vec::new()),
            swap_patterns: Mutex::new(BTreeMap::new()),
        }
    }
    
    /// Record swap event
    pub fn record_swap(&self, event: SwapEvent) {
        let mut history = self.swap_history.lock();
        history.push(event.clone());
        
        // Trim history
        if history.len() > 10000 {
            history.remove(0);
        }
        
        // Update pattern
        if history.len() >= 100 {
            self.update_pattern(event.agent_id, &history);
        }
    }
    
    /// Update swap pattern
    fn update_pattern(&self, agent_id: u64, history: &[SwapEvent]) {
        let agent_swaps: Vec<&SwapEvent> = history.iter()
            .filter(|e| e.agent_id == agent_id)
            .collect();
        
        if agent_swaps.is_empty() {
            return;
        }
        
        let time_span = if agent_swaps.len() > 1 {
            agent_swaps.last().unwrap().timestamp - agent_swaps.first().unwrap().timestamp
        } else {
            1_000_000_000
        };
        let swap_frequency = (agent_swaps.len() as f32) / (time_span as f32 / 1_000_000_000.0);
        
        // Calculate typical swap duration
        let durations: Vec<u64> = agent_swaps.iter()
            .filter_map(|e| e.access_after_swap)
            .collect();
        let typical_duration = if !durations.is_empty() {
            durations.iter().sum::<u64>() / durations.len() as u64
        } else {
            10_000_000_000 // 10 seconds default
        };
        
        let pattern = SwapPattern {
            agent_id,
            swap_frequency,
            typical_swap_duration: typical_duration,
        };
        
        let mut patterns = self.swap_patterns.lock();
        patterns.insert(agent_id, pattern);
    }
    
    /// Predict if page should be swapped out
    pub fn should_swap_out(&self, agent_id: u64, frame: PhysAddr) -> bool {
        // Check if page is hot (don't swap hot pages)
        // Use MemoryPatternAnalyzer to check if page is frequently accessed
        if self.analyzer.is_hot_page(frame) {
            return false; // Don't swap hot pages
        }
        
        // Check swap pattern for agent
        let patterns = self.swap_patterns.lock();
        if let Some(pattern) = patterns.get(&agent_id) {
            // If swap frequency is low, page is stable - don't swap
            if pattern.swap_frequency < 0.01 {
                return false;
            }
        }
        
        // Check if page is predicted to be accessed soon
        if let Some(predicted) = self.analyzer.get_predicted_access(agent_id) {
            // If predicted access is within 100ms, don't swap
            // (would need to check if frame matches predicted address)
        }
        
        true // Can swap out
    }
    
    /// Predict if page should be swapped in
    pub fn should_swap_in(&self, agent_id: u64, frame: PhysAddr) -> bool {
        // Check swap pattern
        let patterns = self.swap_patterns.lock();
        if let Some(pattern) = patterns.get(&agent_id) {
            // If typical swap duration suggests page will be accessed soon, swap in
            // Check swap frequency and typical duration
            // If swap frequency is high and typical duration is short, page is likely to be accessed soon
            if pattern.swap_frequency > 0.1 && pattern.typical_swap_duration < 5_000_000_000 {
                return true; // More than 0.1 swaps per second and typical duration < 5 seconds
            }
            
            // Check if page is predicted to be accessed
            if let Some(predicted) = self.analyzer.get_predicted_access(agent_id) {
                // If predicted access is soon, swap in
                // (would need to check if frame matches predicted address)
                return true;
            }
        }
        
        false
    }
}

/// Adaptive paging manager
pub struct AdaptivePaging {
    analyzer: MemoryPatternAnalyzer,
    swap_predictor: SwapPredictor,
    swap_cache: Mutex<BTreeMap<u64, SwapEntry>>, // frame -> swap entry
}

/// Swap entry
#[derive(Clone, Debug)]
pub struct SwapEntry {
    pub frame: PhysAddr,
    pub agent_id: u64,
    pub swapped_at: u64,
    pub swap_location: u64, // Swap file offset
}

impl AdaptivePaging {
    pub fn new(analyzer: MemoryPatternAnalyzer, swap_predictor: SwapPredictor) -> Self {
        Self {
            analyzer,
            swap_predictor,
            swap_cache: Mutex::new(BTreeMap::new()),
        }
    }
    
    /// Decide if page should be swapped out
    pub fn should_swap_out(&self, agent_id: u64, frame: PhysAddr) -> bool {
        // Don't swap hot pages
        if self.analyzer.is_hot_page(frame) {
            return false;
        }
        
        // Use swap predictor
        self.swap_predictor.should_swap_out(agent_id, frame)
    }
    
    /// Decide if page should be swapped in
    pub fn should_swap_in(&self, agent_id: u64, frame: PhysAddr) -> bool {
        self.swap_predictor.should_swap_in(agent_id, frame)
    }
}

/// Global memory pattern analyzer instance
static MEMORY_ANALYZER: Mutex<Option<MemoryPatternAnalyzer>> = Mutex::new(None);

/// Initialize memory pattern analyzer
pub fn init() {
    let mut analyzer = MEMORY_ANALYZER.lock();
    *analyzer = Some(MemoryPatternAnalyzer::new());
}

/// Get memory pattern analyzer instance
pub fn get_analyzer() -> Option<&'static MemoryPatternAnalyzer> {
    let analyzer = MEMORY_ANALYZER.lock();
    analyzer.as_ref()
}

