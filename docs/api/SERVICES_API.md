# AIOS Services REST API Documentation

**Complete REST API reference for all AIOS userland services**

**Last Updated**: November 19, 2025

---

## Overview

AIOS services expose HTTP REST APIs for agent management, memory operations, IPC, metrics, and boot logs. All services run on localhost and use JSON for request/response bodies.

**Base URLs:**
- Agent Supervisor: `http://localhost:9001`
- Memory Fabric: `http://localhost:9002`
- Semantic IPC: `http://localhost:9003`
- Metrics: `http://localhost:9004`
- Boot Logs: `http://localhost:9005`

---

## 1. Agent Supervisor Service (Port 9001)

### Agent Management

#### Spawn Agent
```http
POST /api/agents/spawn
Content-Type: application/json

{
  "name": "my-agent",
  "type": "worker"
}
```

**Response:**
```json
{
  "agent_id": 562519064,
  "instance_id": 562519065
}
```

#### List Agents
```http
GET /api/agents?name=my-agent
```

**Response:**
```json
[
  {
    "id": 562519064,
    "name": "562519064",
    "state": "running",
    "cpu_id": 0
  }
]
```

#### Get Agent Info
```http
GET /api/agents/562519064
```

**Response:**
```json
{
  "id": 562519064,
  "state": "running",
  "cpu_id": 0
}
```

#### Kill Agent
```http
DELETE /api/agents/562519064
```

**Response:**
```json
{
  "success": true
}
```

**Errors:**
- `403 Forbidden` - Insufficient capabilities (KILL_AGENT required)
- `404 Not Found` - Agent not found

### Agent Lifecycle

#### Checkpoint Agent
```http
POST /api/agents/562519064/checkpoint
```

**Response:**
```json
{
  "checkpoint_id": "checkpoint_562519064_1763562519070"
}
```

**Checkpoint Location:**
`tests/artifacts/integration/checkpoints/checkpoint_562519064_1763562519070`

#### Migrate Agent
```http
POST /api/agents/562519064/migrate
Content-Type: application/json

{
  "target_cpu": 1
}
```

**Response:**
```json
{
  "success": true
}
```

#### Restore Agent
```http
POST /api/agents/restore
Content-Type: application/json

{
  "checkpoint_id": "checkpoint_562519064_1763562519070"
}
```

**Response:**
```json
{
  "agent_id": 562519064,
  "instance_id": 562519065
}
```

### Agent Actions

#### Perform Audited Action
```http
POST /api/agents/562519064/action
Content-Type: application/json

{
  "action": "access_fs"
}
```

**Response:**
```json
{
  "success": true,
  "operation_id": "op_1763562519070_abc123"
}
```

**Errors:**
- `403 Forbidden` - Insufficient capabilities for action

### GPU Management

#### Claim GPU Device
```http
POST /api/gpu/claim
Content-Type: application/json

{
  "agent_id": 562519064,
  "device_id": 0,
  "priority": 10
}
```

**Response:**
```json
{
  "success": true,
  "device_handle": 1
}
```

**Errors:**
- `403 Forbidden` - Insufficient capabilities (ACCESS_GPU required)
- `409 Conflict` - Device already claimed by another agent (unless preemption possible)

**Preemption:**
If request priority > current owner priority, device is preempted and assigned to new agent.

#### Release GPU Device
```http
DELETE /api/gpu/release/1
```

**Response:**
```json
{
  "success": true
}
```

#### Get GPU Device Status
```http
GET /api/gpu/status/0
```

**Response:**
```json
{
  "device_id": 0,
  "owner_agent_id": 562519064,
  "state": "claimed"
}
```

**Available State:**
```json
{
  "device_id": 0,
  "owner_agent_id": null,
  "state": "available"
}
```

#### Get GPU Utilization
```http
GET /api/gpu/utilization
```

**Response:**
```json
{
  "gpu_percent": 45.2
}
```

### Capability & Audit

#### Get Capability Snapshot
```http
GET /api/capabilities/snapshot
```

**Response:**
```json
{
  "capabilities": {
    "562519064": ["SPAWN_AGENT", "ALLOC_MEMORY"],
    "562519069": ["SPAWN_AGENT", "ALLOC_MEMORY", "ACCESS_GPU"]
  }
}
```

#### Get Audit Entry
```http
GET /api/audit/562519064?operation_id=op_1763562519070_abc123
```

**Response:**
```json
{
  "agent_id": 562519064,
  "operation_type": "access_fs",
  "timestamp": 1763562519070
}
```

**Errors:**
- `404 Not Found` - Audit entry not found

### Healing & Metrics

#### Get Healing Events
```http
GET /api/healing/events
```

**Response:**
```json
[
  {
    "timestamp": 1763562519070,
    "event_type": "agent_recovery",
    "confidence_score": 0.95,
    "details": "Agent recovered from memory leak",
    "recovery_time_ms": 150
  }
]
```

#### Get Healing Metrics
```http
GET /api/healing/metrics
```

**Response:**
```json
{
  "heal_ops_per_minute": 5
}
```

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "agentsupervisor"
}
```

---

## 2. Memory Fabric Service (Port 9002)

### Memory Operations

#### Write to Memory Fabric
```http
POST /api/memory/write
Content-Type: application/json

{
  "key": "my:memory:key",
  "data": "SGVsbG8gV29ybGQ=",  // base64 encoded
  "lease_duration_ms": 60000
}
```

**Response:**
```json
{
  "success": true,
  "version_id": 1,
  "lease_id": "lease_my:memory:key_1763562519070"
}
```

#### Read from Memory Fabric
```http
GET /api/memory/read/my:memory:key
```

**Response:**
```json
{
  "data": "SGVsbG8gV29ybGQ=",  // base64 encoded
  "version_id": 1
}
```

**Errors:**
- `404 Not Found` - Memory entry not found or expired

#### Read Specific Version
```http
GET /api/memory/read/my:memory:key/1
```

**Response:**
```json
{
  "data": "SGVsbG8gV29ybGQ=",
  "version_id": 1
}
```

#### Get Memory Fabric Snapshot
```http
GET /api/memory/snapshot
```

**Response:**
```json
{
  "snapshot": {
    "my:memory:key": {
      "versions": 3,
      "latest_version": 3
    }
  }
}
```

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "memoryd"
}
```

---

## 3. Semantic IPC Daemon (Port 9003)

### IPC Operations

#### Send IPC Message
```http
POST /api/ipc/send
Content-Type: application/json

{
  "from": 562519064,
  "to": 562519069,
  "message": {
    "intent": "process_data",
    "data": "Hello from test",
    "type": "text"
  }
}
```

**Alternative Format:**
```json
{
  "from_agent_id": 562519064,
  "to_agent_id": 562519069,
  "data": "SGVsbG8gV29ybGQ=",  // base64
  "metadata": {
    "type": "text"
  }
}
```

**Response:**
```json
{
  "success": true,
  "message_id": 1763562519070
}
```

**Errors:**
- `400 Bad Request` - Type mismatch (declared type doesn't match actual type)

**Type Safety:**
If message declares `"type": "text"` but data is a number, request is rejected with:
```json
{
  "success": false,
  "error": "Type mismatch: declared type 'text' but data is 'number'"
}
```

#### Receive IPC Message
```http
GET /api/ipc/receive/562519069
```

**Response:**
```json
{
  "message": {
    "message_id": 1763562519070,
    "from_agent_id": 562519064,
    "intent": "process_data",
    "data": "Hello from test",
    "type": "text"
  }
}
```

**Errors:**
- `404 Not Found` - No messages available

**Message Queue:**
- Messages are queued per agent (FIFO)
- Each agent has independent message queue
- Messages are dequeued on receive

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "semantic-ipcd"
}
```

---

## 4. Metrics Daemon (Port 9004)

### System Metrics

#### Get CPU Utilization
```http
GET /api/metrics/cpu
```

**Response:**
```json
{
  "cpu_percent": 45.2
}
```

**Implementation:**
- Linux: Reads from `/proc/loadavg`
- macOS: Uses `sysctl vm.loadavg`
- Fallback: Queries service metrics if available

#### Get Memory Utilization
```http
GET /api/metrics/memory
```

**Response:**
```json
{
  "ram_mb": 8192
}
```

#### Get I/O Utilization
```http
GET /api/metrics/io
```

**Response:**
```json
{
  "io_ops_per_sec": 1250.5
}
```

#### Get Swap Activity
```http
GET /api/metrics/swap
```

**Response:**
```json
{
  "swap_in_per_minute": 0,
  "swap_out_per_minute": 0
}
```

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "metricsd"
}
```

---

## 5. Boot Log Server (Port 9005)

### Boot Log Operations

#### Get Boot Log
```http
GET /api/boot/log?boot_id=1763562519070
```

**Response:**
```json
{
  "log": "[2025-11-19T14:28:39.050Z] Kernel initialized\n[2025-11-19T14:28:39.051Z] Memory manager initialized\n..."
}
```

**Without boot_id:**
Returns most recent boot log.

#### Write Boot Log Entry
```http
POST /api/boot/entry
Content-Type: application/json

{
  "entry": "Kernel initialized"
}
```

**Response:**
```json
{
  "success": true
}
```

**Storage:**
Boot logs stored in `tests/artifacts/integration/boot_logs/boot_<timestamp>.log`

---

## Error Responses

All endpoints return consistent error format:

```json
{
  "success": false,
  "error": "Error message description"
}
```

**HTTP Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid request (missing fields, type mismatch)
- `403 Forbidden` - Insufficient capabilities
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict (e.g., GPU already claimed)
- `500 Internal Server Error` - Server error

---

## Authentication & Authorization

Currently, services run on localhost without authentication. In production:

1. **Capability-based authorization** - All operations check agent capabilities
2. **Audit logging** - All privileged operations are logged
3. **Service-to-service authentication** - Services authenticate via capability tokens

---

## Rate Limiting

Currently no rate limiting implemented. In production, consider:
- Per-agent rate limits
- Per-endpoint rate limits
- Burst protection

---

## CORS Configuration

All services have CORS enabled for:
- `http://localhost:*`
- `http://127.0.0.1:*`

In production, restrict to specific origins.

---

## Versioning

Current API version: `v1` (implicit)

Future versions will use URL versioning:
- `/api/v1/agents/spawn`
- `/api/v2/agents/spawn`

---

## Testing

All APIs are tested via integration test suite:

```bash
# Run integration tests
./target/release/aios-test-runner --output tests/artifacts/integration -v integration
```

Test coverage includes:
- All endpoints
- Error cases
- Type validation
- Capability enforcement
- State persistence

---

## Examples

### Complete Agent Lifecycle
```bash
# 1. Spawn agent
curl -X POST http://localhost:9001/api/agents/spawn \
  -H "Content-Type: application/json" \
  -d '{"name": "worker", "type": "worker"}'

# 2. Checkpoint agent
curl -X POST http://localhost:9001/api/agents/562519064/checkpoint

# 3. Migrate agent
curl -X POST http://localhost:9001/api/agents/562519064/migrate \
  -H "Content-Type: application/json" \
  -d '{"target_cpu": 1}'

# 4. Claim GPU
curl -X POST http://localhost:9001/api/gpu/claim \
  -H "Content-Type: application/json" \
  -d '{"agent_id": 562519064, "device_id": 0, "priority": 10}'

# 5. Kill agent
curl -X DELETE http://localhost:9001/api/agents/562519064
```

### Memory Fabric Operations
```bash
# Write data
curl -X POST http://localhost:9002/api/memory/write \
  -H "Content-Type: application/json" \
  -d '{"key": "test:key", "data": "SGVsbG8=", "lease_duration_ms": 60000}'

# Read data
curl http://localhost:9002/api/memory/read/test:key

# Read specific version
curl http://localhost:9002/api/memory/read/test:key/1
```

### IPC Communication
```bash
# Send message
curl -X POST http://localhost:9003/api/ipc/send \
  -H "Content-Type: application/json" \
  -d '{
    "from": 562519064,
    "to": 562519069,
    "message": {
      "intent": "process_data",
      "data": "Hello",
      "type": "text"
    }
  }'

# Receive message
curl http://localhost:9003/api/ipc/receive/562519069
```

---

## Production Considerations

1. **HTTPS**: Use TLS in production
2. **Authentication**: Implement token-based authentication
3. **Rate Limiting**: Add rate limiting per agent/endpoint
4. **Monitoring**: Add metrics and observability
5. **Logging**: Structured logging for all operations
6. **Validation**: Enhanced input validation
7. **Documentation**: OpenAPI/Swagger specification

