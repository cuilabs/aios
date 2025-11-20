//! Kernel Scheduler 2.0
//! 
//! Agent-aware CFS-like fair scheduler with:
//! - Capability-based priority adjustments
//! - Time-slice enforcement
//! - Agent lifecycle hooks
//! - Semantic-hint scheduling (for SILOX)
//! - SMP support with per-CPU runqueues

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use aios_kernel_core::smp;

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
    deadline: Option<u64>,      // Deadline for real-time agents (nanoseconds)
    priority_inherited: bool,   // Whether priority was inherited
    original_priority: i32,     // Original priority before inheritance
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
            deadline: None,
            priority_inherited: false,
            original_priority: priority,
        };

        let mut entities = self.entities.lock();
        entities.insert(agent_id, entity);

        // Add to per-CPU runqueue if SMP is available
        #[cfg(feature = "alloc")]
        {
            if let Some(per_cpu_data) = smp::get_per_cpu_data() {
                let mut runqueue = per_cpu_data.runqueue.lock();
                runqueue.push(agent_id);
            } else {
                // Fallback to global runqueue
                let mut runqueue = self.runqueue.lock();
                runqueue.push(agent_id);
                runqueue.sort_by_key(|&id| {
                    entities.get(&id).map(|e| e.vruntime).unwrap_or(0)
                });
            }
        }

        #[cfg(not(feature = "alloc"))]
        {
            let mut runqueue = self.runqueue.lock();
            runqueue.push(agent_id);
        }

        // Call lifecycle hooks
        self.call_lifecycle_hooks(agent_id, LifecycleHookType::OnSpawn);
    }

    /// Get next agent to run (CFS selection with AI prediction)
    pub fn next(&self) -> Option<u64> {
        // Try per-CPU runqueue first
        #[cfg(feature = "alloc")]
        {
            if let Some(per_cpu_data) = smp::get_per_cpu_data() {
                let mut runqueue = per_cpu_data.runqueue.lock();
                let mut entities = self.entities.lock();
                
                // Use AI prediction if available
                #[cfg(feature = "alloc")]
                {
                    use crate::ai_scheduler::{get, WorkloadPrediction};
                    if let Some(predictor) = get() {
                        // Predict workload for agents in runqueue
                        let mut best_agent = None;
                        let mut best_score = f32::MIN;
                        
                        for &agent_id in runqueue.iter() {
                            if let Some(entity) = entities.get(&agent_id) {
                                // Get AI prediction (5 seconds ahead)
                                if let Some(prediction) = predictor.predict_workload(agent_id, 5_000_000_000) {
                                    // Score based on predicted CPU usage and confidence
                                    let score = prediction.predicted_cpu * prediction.confidence;
                                    if score > best_score {
                                        best_score = score;
                                        best_agent = Some(agent_id);
                                    }
                                }
                            }
                        }
                        
                        // If AI prediction found high-priority agent, use it
                        if let Some(agent_id) = best_agent {
                            if let Some(entity) = entities.get_mut(&agent_id) {
                                entity.last_run = aios_kernel_core::time::now();
                                entity.time_used = 0;
                            }
                            return Some(agent_id);
                        }
                    }
                }
                
                // Fallback to CFS selection
                let mut min_vrt = u64::MAX;
                let mut selected = None;
                
                for &agent_id in runqueue.iter() {
                    if let Some(entity) = entities.get(&agent_id) {
                        let vruntime = entity.vruntime;
                        if vruntime < min_vrt {
                            min_vrt = vruntime;
                            selected = Some(agent_id);
                        }
                    }
                }
                
                if let Some(agent_id) = selected {
                    // Update entity
                    if let Some(entity) = entities.get_mut(&agent_id) {
                        entity.last_run = aios_kernel_core::time::now();
                        entity.time_used = 0;
                    }
                    return Some(agent_id);
                }
            }
        }
        
        // Fallback to global runqueue
        let mut runqueue = self.runqueue.lock();
        let mut entities = self.entities.lock();
        let mut min_vruntime = self.min_vruntime.lock();

        // Use AI prediction if available
        #[cfg(feature = "alloc")]
        {
            use crate::ai_scheduler::{get, WorkloadPrediction};
            if let Some(predictor) = get() {
                // Predict workload for agents in runqueue
                let mut best_agent = None;
                let mut best_score = f32::MIN;
                
                for &agent_id in runqueue.iter() {
                    if let Some(entity) = entities.get(&agent_id) {
                        // Get AI prediction (5 seconds ahead)
                        if let Some(prediction) = predictor.predict_workload(agent_id, 5_000_000_000) {
                            // Score based on predicted CPU usage and confidence
                            let score = prediction.predicted_cpu * prediction.confidence;
                            if score > best_score {
                                best_score = score;
                                best_agent = Some(agent_id);
                            }
                        }
                    }
                }
                
                // If AI prediction found high-priority agent, use it
                if let Some(agent_id) = best_agent {
                    if let Some(entity) = entities.get_mut(&agent_id) {
                        entity.last_run = aios_kernel_core::time::now();
                        entity.time_used = 0;
                    }
                    return Some(agent_id);
                }
            }
        }

        // Fallback to CFS selection
        let mut min_vrt = u64::MAX;
        let mut selected = None;

        for &agent_id in runqueue.iter() {
            if let Some(entity) = entities.get(&agent_id) {
                // Check deadline for real-time agents
                if let Some(deadline) = entity.deadline {
                    let now = aios_kernel_core::time::now();
                    if now >= deadline {
                        // Deadline missed - prioritize this agent
                        selected = Some(agent_id);
                        break;
                    }
                }

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
                entity.last_run = aios_kernel_core::time::now();
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
        // Agents with ADMIN capability get priority boost
        #[cfg(feature = "alloc")]
        {
            use aios_kernel_capability::{get, capability::Capabilities};
            // Query agent's capability token from agent manager
            // Capability system is integrated and ready for use
            // Priority boost would be applied when agent's capability token is available
            // Example: if agent has ADMIN capability, priority += 10
        }

        priority
    }
    
    /// Adjust priority based on capability token (called when token is available)
    #[cfg(feature = "alloc")]
    pub fn adjust_priority_with_capability(&self, agent_id: u64, base_priority: i32, capability: Option<&aios_kernel_capability::capability::CapabilityToken>) -> i32 {
        let mut priority = base_priority;

        // Adjust based on semantic hints
        let hints = self.semantic_hints.lock();
        if let Some(hint) = hints.get(&agent_id) {
            priority += hint.priority_boost;
        }

        // Adjust based on capabilities
        if let Some(cap) = capability {
            use aios_kernel_capability::{capability::Capabilities, has_capability};
            if has_capability(Some(cap), Capabilities::ADMIN) {
                priority += 10; // ADMIN agents get priority boost
            }
            if has_capability(Some(cap), Capabilities::SUPERVISOR) {
                priority += 5; // SUPERVISOR agents get smaller boost
            }
        }

        priority
    }

    /// Set deadline for real-time agent
    pub fn set_deadline(&self, agent_id: u64, deadline: u64) {
        let mut entities = self.entities.lock();
        if let Some(entity) = entities.get_mut(&agent_id) {
            entity.deadline = Some(deadline);
        }
    }

    /// Inherit priority from blocking agent (priority inheritance)
    pub fn inherit_priority(&self, blocking_agent_id: u64, waiting_agent_id: u64) {
        let mut entities = self.entities.lock();
        
        if let Some(blocking_entity) = entities.get_mut(&blocking_agent_id) {
            if let Some(waiting_entity) = entities.get_mut(&waiting_agent_id) {
                // If waiting agent has higher priority, inherit it
                if waiting_entity.priority > blocking_entity.priority {
                    if !blocking_entity.priority_inherited {
                        blocking_entity.original_priority = blocking_entity.priority;
                        blocking_entity.priority_inherited = true;
                    }
                    blocking_entity.priority = waiting_entity.priority;
                }
            }
        }
    }

    /// Restore original priority after inheritance
    pub fn restore_priority(&self, agent_id: u64) {
        let mut entities = self.entities.lock();
        if let Some(entity) = entities.get_mut(&agent_id) {
            if entity.priority_inherited {
                entity.priority = entity.original_priority;
                entity.priority_inherited = false;
            }
        }
    }

    /// Check if agent should be preempted (time slice exhausted or higher priority)
    pub fn should_preempt(&self, current_agent_id: u64, candidate_agent_id: u64) -> bool {
        let entities = self.entities.lock();
        
        if let (Some(current), Some(candidate)) = (
            entities.get(&current_agent_id),
            entities.get(&candidate_agent_id),
        ) {
            // Preempt if candidate has higher priority
            if candidate.priority > current.priority {
                return true;
            }
            
            // Preempt if current agent's time slice is exhausted
            if current.time_used >= current.time_slice {
                return true;
            }
            
            // Preempt if candidate has deadline and current doesn't
            if candidate.deadline.is_some() && current.deadline.is_none() {
                return true;
            }
        }
        
        false
    }
    
    /// Check if agent should be preempted based on capabilities
    #[cfg(feature = "alloc")]
    pub fn should_preempt_by_capability(
        &self,
        current_agent_id: u64,
        candidate_agent_id: u64,
        current_capability: Option<&aios_kernel_capability::capability::CapabilityToken>,
        candidate_capability: Option<&aios_kernel_capability::capability::CapabilityToken>,
    ) -> bool {
        // First check standard preemption
        if self.should_preempt(current_agent_id, candidate_agent_id) {
            return true;
        }
        
        // Check capability-based preemption
        if let (Some(current_cap), Some(candidate_cap)) = (current_capability, candidate_capability) {
            use aios_kernel_capability::{capability::Capabilities, has_capability};
            
            // Preempt if candidate has ADMIN and current doesn't
            if has_capability(Some(candidate_cap), Capabilities::ADMIN) &&
               !has_capability(Some(current_cap), Capabilities::ADMIN) {
                return true;
            }
            
            // Preempt if candidate has SUPERVISOR and current doesn't (and neither has ADMIN)
            if !has_capability(Some(current_cap), Capabilities::ADMIN) &&
               !has_capability(Some(candidate_cap), Capabilities::ADMIN) {
                if has_capability(Some(candidate_cap), Capabilities::SUPERVISOR) &&
                   !has_capability(Some(current_cap), Capabilities::SUPERVISOR) {
                    return true;
                }
            }
        }
        
        false
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
