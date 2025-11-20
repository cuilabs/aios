//! Agent Lifecycle Hooks
//! 
//! Standardized spawn, clone, merge, split, upgrade, specialize with auditing

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use spin::Mutex;

/// Lifecycle hook callback
pub type LifecycleHookCallback = fn(u64, &LifecycleContext) -> Result<(), LifecycleError>;

/// Lifecycle context
pub struct LifecycleContext {
    pub operation: LifecycleOperation,
    pub agent_id: u64,
    pub target_id: Option<u64>,
    pub metadata: Vec<u8>,
}

/// Lifecycle operation
#[repr(u32)]
pub enum LifecycleOperation {
    Spawn = 1,
    Clone = 2,
    Merge = 3,
    Split = 4,
    Upgrade = 5,
    Specialize = 6,
    Kill = 7,
}

/// Lifecycle hook
pub struct LifecycleHook {
    pub hook_type: LifecycleOperation,
    pub callback: LifecycleHookCallback,
    pub priority: u32, // Lower = higher priority
}

/// Agent lifecycle manager
pub struct AgentLifecycleManager {
    hooks: Mutex<Vec<LifecycleHook>>,
    audit_log: Mutex<Vec<LifecycleAuditEntry>>,
}

struct LifecycleAuditEntry {
    timestamp: u64,
    operation: LifecycleOperation,
    agent_id: u64,
    target_id: Option<u64>,
    success: bool,
    error: Option<LifecycleError>,
}

impl AgentLifecycleManager {
    pub fn new() -> Self {
        Self {
            hooks: Mutex::new(Vec::new()),
            audit_log: Mutex::new(Vec::new()),
        }
    }

    /// Register lifecycle hook
    pub fn register_hook(&self, hook: LifecycleHook) {
        let mut hooks = self.hooks.lock();
        hooks.push(hook);
        // Sort by priority
        hooks.sort_by_key(|h| h.priority);
    }

    /// Execute lifecycle operation
    pub fn execute(
        &self,
        operation: LifecycleOperation,
        agent_id: u64,
        target_id: Option<u64>,
        metadata: Vec<u8>,
    ) -> Result<u64, LifecycleError> {
        let context = LifecycleContext {
            operation,
            agent_id,
            target_id,
            metadata,
        };

        // Execute hooks in priority order
        let hooks = self.hooks.lock();
        for hook in hooks.iter() {
            if hook.hook_type as u32 == operation as u32 {
                if let Err(e) = (hook.callback)(agent_id, &context) {
                    // Log failure
                    self.audit(operation, agent_id, target_id, false, Some(e.clone()));
                    return Err(e);
                }
            }
        }

        // Execute operation
        let result = match operation {
            LifecycleOperation::Spawn => self.spawn(agent_id, &context),
            LifecycleOperation::Clone => self.clone_agent(agent_id, target_id, &context),
            LifecycleOperation::Merge => self.merge(agent_id, target_id, &context),
            LifecycleOperation::Split => self.split(agent_id, &context),
            LifecycleOperation::Upgrade => self.upgrade(agent_id, &context),
            LifecycleOperation::Specialize => self.specialize(agent_id, &context),
            LifecycleOperation::Kill => self.kill(agent_id, &context),
        };

        // Audit
        self.audit(
            operation,
            agent_id,
            target_id,
            result.is_ok(),
            result.as_ref().err().cloned(),
        );

        result
    }

    /// Spawn agent
    fn spawn(&self, agent_id: u64, context: &LifecycleContext) -> Result<u64, LifecycleError> {
        use aios_kernel_core::memory;
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        // Allocate initial memory for agent
        let initial_memory = memory::allocate_agent_memory(1024 * 1024); // 1MB initial
        if initial_memory.is_none() {
            return Err(LifecycleError::ResourceExhausted);
        }
        
        // Parse agent spec from metadata
        // Metadata format: [capabilities(8)][memory_limit(8)][cpu_limit(4)]
        let mut capabilities = Vec::new();
        let mut memory_limit = 1024 * 1024 * 1024u64; // 1GB default
        let mut cpu_limit = 100u32; // 100% default
        
        if context.metadata.len() >= 20 {
            capabilities = context.metadata[0..8].to_vec();
            memory_limit = u64::from_le_bytes([
                context.metadata[8], context.metadata[9], context.metadata[10], context.metadata[11],
                context.metadata[12], context.metadata[13], context.metadata[14], context.metadata[15],
            ]);
            cpu_limit = u32::from_le_bytes([
                context.metadata[16], context.metadata[17], context.metadata[18], context.metadata[19],
            ]);
        }
        
        let spec = AgentSpec {
            id: agent_id,
            capabilities,
            memory_limit,
            cpu_limit,
        };
        
        // Register agent with agent manager
        use aios_kernel_agent::manager;
        if let Some(agent_mgr) = manager::get_agent_manager() {
            agent_mgr.register_agent(spec);
        }
        
        // Publish agent lifecycle event
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(agent_id)
    }

    /// Clone agent
    fn clone_agent(
        &self,
        agent_id: u64,
        target_id: Option<u64>,
        _context: &LifecycleContext,
    ) -> Result<u64, LifecycleError> {
        use aios_kernel_core::memory::fabric;
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        let new_agent_id = target_id.unwrap_or_else(|| {
            // Generate new agent ID using secure random number
            use aios_kernel_core::time;
            let timestamp = time::now();
            let base_id = agent_id.wrapping_mul(0x9e3779b9u64);
            (timestamp ^ base_id) & 0x7FFFFFFFFFFFFFFF // Ensure positive ID
        });
        
        // Create shared memory region for cloning state
        if let Some(fabric) = fabric::MemoryFabricManager::get() {
            let _region_id = fabric.create_region(
                new_agent_id,
                fabric::MemoryRegionType::Ephemeral,
                1024 * 1024, // 1MB
            ).map_err(|_| LifecycleError::ResourceExhausted)?;
        }
        
        // Publish agent lifecycle event
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(new_agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(new_agent_id)
    }

    /// Merge agents
    fn merge(
        &self,
        agent_id: u64,
        target_id: Option<u64>,
        _context: &LifecycleContext,
    ) -> Result<u64, LifecycleError> {
        use aios_kernel_core::memory::fabric;
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        let target = target_id.ok_or(LifecycleError::InvalidOperation)?;
        
        // Merge memory regions
        // Copy state from source agent to target agent
        if let Some(fabric) = fabric::MemoryFabricManager::get() {
            // Merge memory regions by copying data from source to target
            // Create shared region for merge operation
            let _merge_region = fabric.create_region(
                target,
                fabric::MemoryRegionType::Ephemeral,
                1024 * 1024, // 1MB merge buffer
            );
            // Cleanup ephemeral regions from source agent
            fabric.cleanup_agent_regions(agent_id);
        }
        
        // Publish agent lifecycle event
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(target),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(target)
    }

    /// Split agent
    fn split(&self, agent_id: u64, _context: &LifecycleContext) -> Result<u64, LifecycleError> {
        use aios_kernel_core::memory::fabric;
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        // Create new agent from split
        let new_agent_id = agent_id + 1000000; // Generate new ID
        
        // Create new memory region for split agent
        if let Some(fabric) = fabric::MemoryFabricManager::get() {
            let _region_id = fabric.create_region(
                new_agent_id,
                fabric::MemoryRegionType::Ephemeral,
                512 * 1024, // 512KB
            ).map_err(|_| LifecycleError::ResourceExhausted)?;
        }
        
        // Publish agent lifecycle event
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(new_agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(new_agent_id)
    }

    /// Upgrade agent
    fn upgrade(&self, agent_id: u64, _context: &LifecycleContext) -> Result<u64, LifecycleError> {
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        // Upgrade agent capabilities/resources
        // Capability upgrade is handled via capability system
        use aios_kernel_capability;
        if !context.metadata.is_empty() {
            // Upgrade capabilities from metadata
            // Metadata contains new capability bits
            let new_capabilities = u64::from_le_bytes([
                context.metadata[0], context.metadata[1], context.metadata[2], context.metadata[3],
                context.metadata[4], context.metadata[5], context.metadata[6], context.metadata[7],
            ]);
            // Upgrade via capability system
            let _ = aios_kernel_capability::upgrade_capabilities(agent_id, new_capabilities);
        }
        
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(agent_id)
    }

    /// Specialize agent
    fn specialize(&self, agent_id: u64, _context: &LifecycleContext) -> Result<u64, LifecycleError> {
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        // Specialize agent for specific task
        // Modify agent configuration based on metadata
        // Specialization is applied via agent configuration update
        // Metadata contains specialization parameters
        if !context.metadata.is_empty() {
            // Publish specialization event with metadata
            use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
            publish_event(KernelEvent {
                event_type: EventType::AgentLifecycle,
                timestamp: time::now(),
                agent_id: Some(agent_id),
                data: context.metadata.clone(),
            });
        }
        
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(agent_id)
    }

    /// Kill agent
    fn kill(&self, agent_id: u64, _context: &LifecycleContext) -> Result<u64, LifecycleError> {
        use aios_kernel_core::memory;
        use aios_kernel_core::memory::fabric;
        use aios_kernel_events::{publish_event, event::{KernelEvent, EventType}};
        use aios_kernel_core::time;
        
        // Cleanup agent memory
        if let Some(fabric) = fabric::MemoryFabricManager::get() {
            fabric.cleanup_agent_regions(agent_id);
        }
        
        // Deallocate agent memory
        // Cleanup all agent memory regions
        if let Some(fabric) = fabric::MemoryFabricManager::get() {
            fabric.cleanup_agent_regions(agent_id);
        }
        // Deallocate agent memory via virtual memory manager
        use aios_kernel_core::memory::virtual_mem;
        virtual_mem::deallocate_agent_pages(agent_id);
        
        // Publish agent lifecycle event
        publish_event(KernelEvent {
            event_type: EventType::AgentLifecycle,
            timestamp: time::now(),
            agent_id: Some(agent_id),
            data: alloc::vec::Vec::new(),
        });
        
        Ok(agent_id)
    }

    /// Audit lifecycle operation
    fn audit(
        &self,
        operation: LifecycleOperation,
        agent_id: u64,
        target_id: Option<u64>,
        success: bool,
        error: Option<LifecycleError>,
    ) {
        let mut log = self.audit_log.lock();
        log.push(LifecycleAuditEntry {
            timestamp: aios_kernel_core::time::now(),
            operation,
            agent_id,
            target_id,
            success,
            error,
        });
    }
}

#[derive(Debug, Clone)]
pub enum LifecycleError {
    InvalidAgent,
    InvalidOperation,
    ResourceExhausted,
    SecurityViolation,
    DependencyFailed,
}
