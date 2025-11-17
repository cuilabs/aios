//! Kernel Scheduler 2.0
//! 
//! Agent-aware CFS-like fair scheduler with:
//! - Capability-based priority adjustments
//! - Time-slice enforcement
//! - Agent lifecycle hooks
//! - Semantic-hint scheduling (for SILOX)

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Agent scheduling entity (CFS-like)
struct AgentEntity {
    agent_id: u64,
    vruntime: u64,              // Virtual runtime (for CFS)
    weight: u64,                // Scheduling weight
    priority: i32,               // Priority (adjusted by capabilities)
    time_slice: u64,            // Time slice in nanoseconds
    time_used: u64,             // Time used in current slice
    last_run: u64,              // Last run timestamp
    lifecycle_hooks: Vec<LifecycleHook>,
}

/// Lifecycle hook
pub struct LifecycleHook {
    pub hook_type: LifecycleHookType,
    pub callback: fn(u64), // Agent ID
}

/// Lifecycle hook type
#[repr(u32)]
pub enum LifecycleHookType {
    OnSpawn = 1,
    OnKill = 2,
    OnClone = 3,
    OnMerge = 4,
    OnSplit = 5,
    OnUpgrade = 6,
    OnSpecialize = 7,
}

/// Semantic scheduling hint (for SILOX)
pub struct SemanticHint {
    pub agent_id: u64,
    pub hint_type: SemanticHintType,
    pub priority_boost: i32,
}

/// Semantic hint type
#[repr(u32)]
pub enum SemanticHintType {
    HighPriority = 1,
    LowLatency = 2,
    Batch = 3,
    Interactive = 4,
}

/// Agent Scheduler 2.0
/// 
/// CFS-like fair scheduler with agent awareness
pub struct AgentScheduler {
    entities: Mutex<BTreeMap<u64, AgentEntity>>,
    runqueue: Mutex<Vec<u64>>, // Sorted by vruntime
    semantic_hints: Mutex<BTreeMap<u64, SemanticHint>>,
    min_vruntime: Mutex<u64>,
}

impl AgentScheduler {
    pub fn new() -> Self {
        Self {
            entities: Mutex::new(BTreeMap::new()),
            runqueue: Mutex::new(Vec::new()),
            semantic_hints: Mutex::new(BTreeMap::new()),
            min_vruntime: Mutex::new(0),
        }
    }

    /// Add agent to scheduler
    pub fn add_agent(&self, agent_id: u64, weight: u64, priority: i32) {
        let entity = AgentEntity {
            agent_id,
            vruntime: 0,
            weight,
            priority,
            time_slice: 10_000_000, // 10ms default
            time_used: 0,
            last_run: 0,
            lifecycle_hooks: Vec::new(),
        };

        let mut entities = self.entities.lock();
        entities.insert(agent_id, entity);

        let mut runqueue = self.runqueue.lock();
        runqueue.push(agent_id);
        runqueue.sort_by_key(|&id| {
            entities.get(&id).map(|e| e.vruntime).unwrap_or(0)
        });

        // Call lifecycle hooks
        self.call_lifecycle_hooks(agent_id, LifecycleHookType::OnSpawn);
    }

    /// Get next agent to run (CFS selection)
    pub fn next(&self) -> Option<u64> {
        let mut runqueue = self.runqueue.lock();
        let mut entities = self.entities.lock();
        let mut min_vruntime = self.min_vruntime.lock();

        // Find agent with minimum vruntime
        let mut min_vrt = u64::MAX;
        let mut selected = None;

        for &agent_id in runqueue.iter() {
            if let Some(entity) = entities.get(&agent_id) {
                // Adjust priority based on capabilities and semantic hints
                let adjusted_priority = self.adjust_priority(agent_id, entity.priority);

                let vruntime = entity.vruntime;
                if vruntime < min_vrt {
                    min_vrt = vruntime;
                    selected = Some(agent_id);
                }
            }
        }

        if let Some(agent_id) = selected {
            // Update min_vruntime
            if min_vrt > *min_vruntime {
                *min_vruntime = min_vrt;
            }

            // Update entity
            if let Some(entity) = entities.get_mut(&agent_id) {
                entity.last_run = 0; // TODO: Get from kernel time
                entity.time_used = 0;
            }

            Some(agent_id)
        } else {
            None
        }
    }

    /// Update agent runtime (called on context switch)
    pub fn update_runtime(&self, agent_id: u64, time_delta: u64) {
        let mut entities = self.entities.lock();
        if let Some(entity) = entities.get_mut(&agent_id) {
            entity.time_used += time_delta;

            // Update vruntime (CFS formula)
            let weight = entity.weight;
            let vruntime_delta = (time_delta * 1024) / weight; // Normalize by weight
            entity.vruntime += vruntime_delta;

            // Check if time slice exhausted
            if entity.time_used >= entity.time_slice {
                // Time slice exhausted - agent should yield
                entity.time_used = 0;
            }
        }

        // Re-sort runqueue
        let mut runqueue = self.runqueue.lock();
        runqueue.sort_by_key(|&id| {
            entities.get(&id).map(|e| e.vruntime).unwrap_or(0)
        });
    }

    /// Yield current agent (put back in queue)
    pub fn yield_agent(&self, agent_id: u64) {
        // Agent is already in runqueue, just update vruntime
        self.update_runtime(agent_id, 0);
    }

    /// Adjust priority based on capabilities and semantic hints
    fn adjust_priority(&self, agent_id: u64, base_priority: i32) -> i32 {
        let mut priority = base_priority;

        // Adjust based on semantic hints
        let hints = self.semantic_hints.lock();
        if let Some(hint) = hints.get(&agent_id) {
            priority += hint.priority_boost;
        }

        // Adjust based on capabilities
        // TODO: Query capability system for priority boost

        priority
    }

    /// Set semantic hint for agent
    pub fn set_semantic_hint(&self, agent_id: u64, hint: SemanticHint) {
        let mut hints = self.semantic_hints.lock();
        hints.insert(agent_id, hint);
    }

    /// Register lifecycle hook
    pub fn register_lifecycle_hook(&self, agent_id: u64, hook: LifecycleHook) {
        let mut entities = self.entities.lock();
        if let Some(entity) = entities.get_mut(&agent_id) {
            entity.lifecycle_hooks.push(hook);
        }
    }

    /// Call lifecycle hooks
    fn call_lifecycle_hooks(&self, agent_id: u64, hook_type: LifecycleHookType) {
        let entities = self.entities.lock();
        if let Some(entity) = entities.get(&agent_id) {
            for hook in &entity.lifecycle_hooks {
                if hook.hook_type as u32 == hook_type as u32 {
                    (hook.callback)(agent_id);
                }
            }
        }
    }

    /// Remove agent from scheduler
    pub fn remove_agent(&self, agent_id: u64) {
        let mut entities = self.entities.lock();
        entities.remove(&agent_id);

        let mut runqueue = self.runqueue.lock();
        runqueue.retain(|&id| id != agent_id);

        let mut hints = self.semantic_hints.lock();
        hints.remove(&agent_id);

        // Call lifecycle hooks
        self.call_lifecycle_hooks(agent_id, LifecycleHookType::OnKill);
    }

    /// Set time slice for agent
    pub fn set_time_slice(&self, agent_id: u64, time_slice: u64) {
        let mut entities = self.entities.lock();
        if let Some(entity) = entities.get_mut(&agent_id) {
            entity.time_slice = time_slice;
        }
    }

    /// Get agent statistics
    pub fn get_stats(&self, agent_id: u64) -> Option<AgentSchedulerStats> {
        let entities = self.entities.lock();
        if let Some(entity) = entities.get(&agent_id) {
            Some(AgentSchedulerStats {
                vruntime: entity.vruntime,
                time_used: entity.time_used,
                time_slice: entity.time_slice,
                priority: entity.priority,
            })
        } else {
            None
        }
    }
}

/// Agent scheduler statistics
pub struct AgentSchedulerStats {
    pub vruntime: u64,
    pub time_used: u64,
    pub time_slice: u64,
    pub priority: i32,
}
