# Kernel API Documentation

## Overview

The AIOS kernel provides a comprehensive API for agent-first computing with security, performance, and observability built-in.

## Core APIs

### Memory Management

#### `allocate_agent_memory(size: usize) -> Option<PhysAddr>`
Allocates physical memory for an agent.

**Parameters:**
- `size`: Size in bytes to allocate

**Returns:**
- `Some(PhysAddr)`: Physical address of allocated memory
- `None`: Allocation failed

**Example:**
```rust
let memory = allocate_agent_memory(1024 * 1024); // 1MB
```

#### `deallocate_agent_memory(addr: PhysAddr, size: usize)`
Deallocates agent memory.

**Parameters:**
- `addr`: Physical address to deallocate
- `size`: Size in bytes

### Memory Fabric

#### `MemoryFabricManager::create_region(agent_id, region_type, size) -> Result<u64, MemoryFabricError>`
Creates a memory region in the fabric.

**Parameters:**
- `agent_id`: Agent ID
- `region_type`: `Ephemeral` or `Persistent`
- `size`: Size in bytes

**Returns:**
- `Ok(region_id)`: Region ID
- `Err(MemoryFabricError)`: Creation failed

### Agent Scheduling

#### `AgentScheduler::add_agent(agent_id, weight, priority)`
Adds an agent to the scheduler.

**Parameters:**
- `agent_id`: Agent ID
- `weight`: Scheduling weight
- `priority`: Priority level

#### `AgentScheduler::next() -> Option<u64>`
Gets the next agent to run.

**Returns:**
- `Some(agent_id)`: Next agent ID
- `None`: No agents available

### Policy Engine

#### `PolicyEngine::evaluate(context: &PolicyContext) -> PolicyDecision`
Evaluates policies for an operation.

**Parameters:**
- `context`: Policy context with agent_id, operation, resource

**Returns:**
- `PolicyDecision`: Allow/deny decision with constraints

### IPC

#### `BinaryIPC::send(message: IPCMessage) -> Result<(), IPCError>`
Sends a binary IPC message.

**Parameters:**
- `message`: IPC message structure

**Returns:**
- `Ok(())`: Message sent
- `Err(IPCError)`: Send failed

## Event Bus

#### `publish_event(event: KernelEvent)`
Publishes a kernel event.

**Parameters:**
- `event`: Kernel event structure

#### `subscribe(event_types, agent_id_filter, callback) -> u64`
Subscribes to kernel events.

**Parameters:**
- `event_types`: Vector of event types
- `agent_id_filter`: Optional agent ID filter
- `callback`: Callback function

**Returns:**
- Subscription ID

## Observability

#### `ObservabilitySystem::collect_metrics() -> SystemMetrics`
Collects system-wide metrics.

**Returns:**
- `SystemMetrics`: CPU, memory, network, I/O metrics

#### `ObservabilitySystem::get_agent_profile(agent_id) -> Option<AgentProfile>`
Gets agent behavior profile.

**Parameters:**
- `agent_id`: Agent ID

**Returns:**
- `Some(AgentProfile)`: Agent profile
- `None`: Agent not found

## Audit

#### `AuditManager::record(event: AuditEvent)`
Records an audit event.

**Parameters:**
- `event`: Audit event structure

#### `AuditLog::verify() -> bool`
Verifies audit log integrity.

**Returns:**
- `true`: Log is valid
- `false`: Log integrity compromised

## Version

**Last Updated:** November 2025  
**API Version:** 0.1.0

