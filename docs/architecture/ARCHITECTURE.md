# AIOS Architecture

**AIOS — AI-Native Operating System**

This document describes the architecture of AIOS, an operating system where native processes are AI agents.

## Core Philosophy

**AIOS = Linux + Kubernetes + LangChain + post-quantum crypto + cognitive runtime**

AIOS treats AI agents as first-class citizens. Instead of traditional OS primitives (files, processes, sockets), AIOS provides:

- **Agent identity management** — Cryptographic identities for agents
- **Memory graphs** — Semantic memory accessible by agents
- **Context buses** — Shared execution contexts
- **Reasoning modules** — Planning and execution engines (userland services)
- **Semantic IPC** — Inter-agent messaging with structured intent (userland daemon)
- **Cognitive security layers** — Capability-based security with behavioral analysis
- **Self-optimization kernels** — Meta-learning and performance optimization

## Why This Matters

1. **First OS where the primary citizens are AI agents, not human processes**
2. **First OS where IPC supports semantic interpretation (via userland daemon)**
3. **First OS with built-in PQ crypto syscalls (full implementation in userland)**
4. **First OS with cognition as a primitive (planning, context, memory)**
5. **First OS designed to eventually support SILOX (self-evolving compute)**

## Architecture Layers

### Kernel Layer (Rust)

**Location:** `kernel/`

A real operating system kernel written in Rust, running on bare metal. The kernel maintains OS purity: deterministic, byte-level operations only.

#### kernel-core
- Boot and initialization
- Physical and virtual memory management
- Interrupt handling
- System call interface (with capability validation)
- Agent memory pools

#### kernel-hal
- **Hardware Abstraction Layer** — Portable hardware interfaces
- APIC (Advanced Programmable Interrupt Controller)
- Timer (HPET/TSC)
- PCIe enumeration
- IOMMU setup
- ACPI (Advanced Configuration and Power Interface)
- SATA/NVMe drivers
- Basic NIC drivers

#### kernel-agent
- Agent-first process management
- Agent scheduler (deterministic epoch + weighted fair share + EDF)
- Agent lifecycle (spawn, clone, merge, split, upgrade, specialize)
- Resource isolation
- Agent sandbox runtime (Wasm/microVM)

#### kernel-ipc
- **Binary IPC kernel primitives** — Deterministic, byte-level message routing
- Routes messages by agent ID only
- No semantic interpretation (maintains determinism and security boundaries)
- Semantic interpretation handled in userland via `semantic-ipcd` daemon
- Size limits: 64KB max message size

#### kernel-crypto
- **PQC syscalls only** — Kernel exposes syscall interface
- Full PQC libraries (CRYSTALS-Kyber, CRYSTALS-Dilithium) implemented in userland
- Keys stored in secure enclaves or hardware modules
- Minimal stubs for hardware acceleration (if available)
- Keeps kernel small and future-proof

### Userland Services

**Location:** `services/`

Privileged system services running in userland:

#### semantic-ipcd
- **Semantic IPC Daemon** — Interprets intent from binary IPC messages
- Receives binary messages from kernel
- Extracts semantic intent from metadata
- Routes based on intent (not just agent ID)
- Provides semantic layer on top of kernel binary IPC

#### planner (/svc/planner)
- **Planning Service** — Privileged system service for agent planning
- Creates and executes planning tasks
- Handles reasoning and execution graphs
- **Not a kernel syscall** — Maintains kernel determinism and auditability
- Called via userland API

#### memoryd (/svc/memoryd)
- **Memory Fabric Service** — Privileged userland service for semantic memory
- Implements semantic memory fabric (vector stores, embeddings, semantic indexes)
- Uses kernel primitives (FrameAlloc, PageMap, AgentPoolAlloc) for low-level operations
- Provides high-level semantic memory operations via IPC
- **Not a kernel syscall** — MemoryStore/MemoryQuery moved from kernel to IPC calls

### Runtime Layer (TypeScript)

**Location:** `packages/` and `apps/`

High-level runtime and userland for development and deployment:

- Agent orchestration
- Workflow management
- Application layer
- Development tools

## System Calls

AIOS provides system calls for low-level agent operations. All syscalls:
- **Validate capability tokens** — Every syscall requires valid capability
- **Enforce size/time limits** — Prevents resource exhaustion
- **Use async/non-blocking** — Long operations return async handles

### Kernel Syscalls

#### Agent Operations
- `AgentSpawn` - Create new agent (async, returns instance ID)
  - Validates capability token
  - Returns instance ID immediately (spawn happens asynchronously)
  - Failure modes: Invalid capability, resource exhaustion, security violation

#### IPC Operations
- `IPCSend` - Send binary IPC message (kernel routes by agent ID)
  - Size limit: 64KB max
  - Validates capability token
  - Returns message ID

- `IPCRecv` - Receive binary IPC message (returns raw binary data)
  - Non-blocking
  - Returns message ID

#### Memory Operations (Low-Level Primitives)
- `FrameAlloc` - Allocate physical frame
- `PageMap` - Map virtual to physical page
- `AgentPoolAlloc` - Allocate from agent memory pool
- `AgentMemAlloc` - Allocate agent memory (with size limits: 1GB max per agent)
- `AgentMemFree` - Free agent memory

#### Crypto Operations
- `PQCOperation` - Post-quantum crypto operation (async, delegates to userland)
  - Async operation returns handle
  - Full implementation in userland

### Userland APIs (Not Syscalls)
- **Planning** - `/svc/planner` service (called via IPC)
- **Semantic IPC** - `semantic-ipcd` daemon (interprets binary IPC)
- **Memory Fabric** - `/svc/memoryd` service (called via IPC, not syscall)
- **Full PQC** - Userland libraries (kernel only provides syscall interface)

## Boot & Attestation

### Secure Boot Chain
1. **UEFI Secure Boot** — Validates bootloader signature
2. **Bootloader** — Validates kernel signature
3. **Kernel** — Validates init and system services
4. **Init** — Authenticates system services
5. **System Services** — Attest to remote verifiers

### TPM Measured Boot
- Each stage measures next stage
- Measurements stored in TPM PCRs
- Remote attestation verifies chain

### Remote Attestation
- Agent identities attested via secure enclaves
- Enclave → sign key flow
- Remote verifiers validate attestation

### Enclave Lifecycle
1. Enclave creation with attestation
2. Key provisioning (attested keys)
3. Enclave operations
4. Enclave termination

## Agent Sandbox Runtime

### Runtime Models
- **WebAssembly (Wasmtime)** — Strong isolation, portable
- **microVMs (Firecracker)** — Lightweight, fast startup
- **Native code** — With capability restrictions
- **Python runtimes** — Sandboxed execution
- **Model binaries** — Safe execution with quotas

### GPU Access
- **VFIO (Virtual Function I/O)** — Direct GPU passthrough
- **Mediated devices** — Shared GPU access
- **Capability checks** — GPU access requires capability token
- **Quota accounting** — GPU usage tracked and limited

### Isolation
- Memory isolation per agent
- CPU isolation (cgroups-like)
- Network isolation
- Storage isolation

## Deterministic Scheduler

### Scheduler Design
- **Deterministic epoch window** — Predictable scheduling
- **Weighted fair share** — Fair resource allocation
- **Real-time class (EDF)** — Earliest Deadline First for low-latency agents
- **Preemption** — High-priority agents can preempt
- **Priority inversion handling** — Prevents starvation
- **Starvation controls** — Ensures all agents make progress

### Scheduling Algorithm
1. Agents assigned to scheduling classes
2. Epoch-based scheduling windows
3. Weighted fair share within epochs
4. EDF for real-time agents
5. Preemption based on priority

## Memory Model

### Kernel Primitives (Low-Level)
- **FrameAlloc** — Physical frame allocation
- **PageMap** — Virtual to physical mapping
- **AgentPoolAlloc** — Agent memory pool allocation

### Userland Service (High-Level)
- **memoryd** — Semantic memory fabric service
- Vector stores
- Embeddings
- Semantic indexes
- Memory versioning
- Memory graphs

### Memory Flow
1. Agent requests memory operation
2. IPC call to `/svc/memoryd`
3. memoryd uses kernel primitives (FrameAlloc, PageMap, AgentPoolAlloc)
4. memoryd implements semantic layer
5. Returns result via IPC

## IPC Model

### Kernel Level (Binary)
- **Binary message packets** — Deterministic, byte-level
- Routes by agent ID only
- No semantic interpretation
- IPC-safe and low-latency
- Size limit: 64KB

### Userland Level (Semantic)
- **semantic-ipcd daemon** — Interprets intent from binary messages
- Extracts semantic intent from metadata
- Routes based on meaning
- Provides semantic layer

### Message Flow
1. Agent sends message with semantic metadata
2. Kernel receives binary packet, routes by agent ID
3. `semantic-ipcd` receives binary message
4. Daemon interprets semantic intent
5. Daemon routes based on intent
6. Kernel delivers to destination agent

## Security Model

### Capability Tokens
- Every syscall requires valid capability token
- Tokens contain:
  - Token ID
  - Agent ID
  - Capabilities (bitmask)
  - Expiration time
  - Cryptographic signature
- Kernel validates tokens before executing syscalls

### Identity
- Cryptographic identities for agents
- Quantum-safe key pairs (stored in secure enclaves)
- Attestation

### Capabilities
- Fine-grained permissions
- Capability tokens
- Expiration and revocation

### Trust
- Agent-to-agent trust graphs
- Trust levels and paths
- Capability delegation

### Behavioral Analysis
- Anomaly detection
- Pattern analysis
- Security monitoring

## Post-Quantum Cryptography

### Kernel
- Syscall interface only (`PQCOperation`)
- Secure enclave integration
- Hardware acceleration stubs

### Userland
- Full CRYSTALS-Kyber implementation
- Full CRYSTALS-Dilithium implementation
- Key management
- Cryptographic operations

### Key Storage
- Keys stored in secure enclaves
- Hardware security modules
- Kernel provides storage interface

## Planning and Reasoning

### Kernel
- **No planning syscalls** — Maintains kernel determinism
- Planning is a userland service

### Userland
- `/svc/planner` — Privileged planning service
- Creates and executes planning tasks
- Handles reasoning and execution graphs
- Called via userland API (IPC)

## Device & Accelerator Driver Model

### Driver Interface
- **Crypto accelerators** — Hardware PQC acceleration
- **GPUs** — VFIO/mediated device access
- **TPUs** — Tensor processing units
- **NPUs** — Neural processing units

### Agent Access
- **Capability checks** — Device access requires capability
- **Quota accounting** — Device usage tracked
- **Safe APIs** — Isolated device access

## Policy & Fail-Safe Features

### Circuit Breaker
- System-wide circuit breaker
- Prevents cascading failures
- Automatic recovery

### Per-Agent Kill Switch
- Immediate agent termination
- Emergency stop capability
- Audit logging

### Canary Modes
- Gradual rollout
- A/B testing
- Safe deployment

### Automatic Rollback
- Rollback to last known-good snapshot
- Per-agent versioned snapshots
- Snapshot verification

### Quorum Requirements
- Global shutdown requires quorum
- Prevents accidental shutdowns
- Consensus-based decisions

## Immutable Audit & Governance

### Append-Only Event Log
- **Local signed journal** — All events signed
- **Optional ledger anchoring** — Hash anchors to external ledger
- **Export APIs** — Export logs for audit
- **Verification APIs** — Verify log integrity
- **Legal audit support** — Compliance-ready

### Per-Agent Snapshots
- Versioned snapshot store
- Rollback capability
- Snapshot verification

## Developer & Operator Surface

### SDKs
- **Rust SDK** — Native kernel development
- **TypeScript SDK** — Userland development
- **Python SDK** — Agent development

### OpenAPI
- Userland services expose OpenAPI
- Standard REST interfaces
- API documentation

### Reproducible Builds
- Build manifests
- Deterministic builds
- Reproducible artifacts

### DevVM
- Local testing environment
- Agent development
- Integration testing

### Agent Lifecycle Tooling
- **Image format** — Agent container format
- **Signing** — Cryptographic signing
- **Provenance** — Supply chain tracking

## Tests, Fuzzing, and Verification

### Unit Tests
- Kernel primitive tests
- Syscall harness tests
- Property tests for determinism

### Fuzzing
- Syscall fuzzers
- TCB (Trusted Computing Base) fuzzing
- Continuous fuzzing pipelines

### Verification
- Formal verification (where applicable)
- Security audits
- Compliance verification

## Licensing & Open-Core Split

### Open Source (MIT/Apache 2.0)
- **Kernel** — Full kernel source code
- **Basic runtime** — Core runtime components
- **SDKs** — Development SDKs
- **Documentation** — Architecture and API docs

### Commercial (Proprietary)
- **memoryd** — Advanced memory fabric features
- **Behavioral analyzer** — Advanced behavioral analysis
- **SILOX meta-learning** — Self-evolving compute
- **Enterprise features** — Advanced governance, compliance

### Clear Boundaries
- Open source: Core OS functionality
- Commercial: Advanced AI/ML features
- No feature gating in open source
- Commercial features are additive

## Design Principles

### Kernel Purity
- Deterministic operations only
- Byte-level IPC (no semantic interpretation)
- Minimal crypto (syscalls only, not full libraries)
- No planning/reasoning (userland services)
- No memory fabric (userland service)

### Userland Services
- Semantic interpretation (`semantic-ipcd`)
- Planning and reasoning (`/svc/planner`)
- Memory fabric (`/svc/memoryd`)
- Full PQC libraries
- High-level operations

### Separation of Concerns
- Kernel: Low-level, deterministic, fast
- Userland: High-level, semantic, flexible

## Future Enhancements

- **Distributed Memory** - Distributed memory fabric
- **Meta-Learning** - Self-improving agents
- **Quantum Computing** - Quantum algorithm integration
- **Hardware Acceleration** - GPU/TPU support
- **Real Post-Quantum Crypto** - Actual CRYSTALS implementations in userland
- **SILOX Integration** - Self-evolving compute

---

**Built:** August 2025  
**Version:** 0.1.0  
**Status:** Production-grade foundation - early development  
**Target:** x86_64 bare metal
