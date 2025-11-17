# AIOS — AI-Native Operating System

**The OS layer for agent-first computing.**

This is what Windows, macOS, Linux, and Android cannot become.

---

**Copyright (c) 2025 CUI Labs (Pte.) Ltd., Singapore**

AIOS is developed and maintained by [CUI Labs (Pte.) Ltd.](https://cuilabs.io), Singapore.

**"AIOS" and the AIOS logo are trademarks of CUI Labs (Pte.) Ltd., Singapore.**

## 🧠 Core Definition

AIOS is an operating system where native processes are AI agents, not traditional applications.

**AIOS = Linux + Kubernetes + LangChain + post-quantum crypto + cognitive runtime**

Instead of:
- File managers
- Processes
- Sockets
- System interrupts
- Drivers

AIOS exposes:
- Agent identity management
- Memory graphs
- Context buses
- Reasoning modules
- Semantic IPC (inter-agent messaging)
- Cognitive security layers
- Self-optimization kernels

**Think:** Linux kernel + multi-agent brain + self-evolving compute fabric.

## 📚 Documentation

All documentation is organized in the [`docs/`](./docs/) directory.

### Core Architecture
- **[Architecture](./docs/architecture/ARCHITECTURE.md)** - Complete architecture documentation
- **[Technical Whitepaper](./docs/architecture/AIOS_TECHNICAL_WHITEPAPER.md)** - Comprehensive technical whitepaper
- **[Boot Sequence](./docs/architecture/BOOT_SEQUENCE.md)** - Detailed boot & attestation flow
- **[Repository Layout](./docs/architecture/REPO_LAYOUT.md)** - Repository structure
- **[Syscall API](./docs/architecture/SYSCALL_API.md)** - Complete syscall API with examples

### Component Documentation
- **[Missing Components](./docs/components/MISSING_COMPONENTS.md)** - Critical missing OS components
- **[Agent-First Components](./docs/components/AGENT_FIRST_COMPONENTS.md)** - 🔥 MANDATORY agent-first architecture components
- **[Foundational Subsystems](./docs/components/FOUNDATIONAL_SUBSYSTEMS.md)** - 13 foundational subsystems status
- **[Subsystem Inventory](./docs/components/SUBSYSTEM_INVENTORY.md)** - Complete inventory of all 30 subsystems
- **[Complete System Inventory](./docs/components/COMPLETE_SYSTEM_INVENTORY.md)** - Complete inventory of 39 unique components (no overlaps)

### Setup & Getting Started
- **[GitHub Setup](./docs/setup/GITHUB_SETUP.md)** - GitHub repository setup guide
- **[Quick Start](./docs/setup/QUICK_START.md)** - Quick start guide for pushing to GitHub

See [docs/README.md](./docs/README.md) for the complete documentation index.

## 🏗 Architecture

### Kernel (Rust)
**Location:** `kernel/`

A real operating system kernel written in Rust:
- Agent-first process management
- Binary IPC at kernel level
- Post-quantum cryptography syscalls
- Hardware abstraction layer (HAL)
- Secure boot & attestation

### Services (TypeScript/Rust)
**Location:** `services/`

Privileged system services:
- `initd` - Init daemon (PID 1)
- `identityd` - Identity service
- `memoryd` - Memory fabric service
- `semantic-ipcd` - Semantic IPC daemon
- `planner` - Planning service
- `agentsupervisor` - Agent supervisor
- `networkd` - Network service

### Runtime (TypeScript)
**Location:** `packages/` and `apps/`

High-level runtime and userland:
- Agent orchestration
- Workflow management
- Application layer
- Development tools

## 🚀 Getting Started

### Kernel Development

```bash
cd kernel
./build.sh
```

### Runtime Development

```bash
# Install dependencies
pnpm install

# Build all packages
pnpm build

# Run development mode
pnpm dev
```

## 📦 Project Structure

```
AIOS/
├── boot/              # Bootloader and initramfs
├── kernel/            # Rust kernel
├── services/          # Userland services
├── packages/          # TypeScript packages
├── apps/              # Applications
├── sdk/               # SDKs (Rust, TypeScript, Python)
├── tests/             # Tests
└── docs/              # Documentation
```

## 🔐 Security

AIOS uses:
- Capability tokens (required for all syscalls)
- Behavioral anomaly detection
- Cryptographic identity
- Quantum-safe signing (CRYSTALS-Dilithium)
- Strict agent-to-agent trust graphs
- Secure boot & TPM measured boot
- Remote attestation

## 🎯 Why This Matters

1. **First OS where the primary citizens are AI agents, not human processes**
2. **First OS where IPC supports semantic interpretation (via userland daemon)**
3. **First OS with built-in PQ crypto syscalls (full implementation in userland)**
4. **First OS with cognition as a primitive (planning, context, memory)**
5. **First OS designed to eventually support SILOX (self-evolving compute)**

## 📄 License

**Open Source (MIT/Apache 2.0):**
- Kernel
- Basic runtime
- SDKs
- Documentation

**Commercial (Proprietary):**
- memoryd (advanced features)
- Behavioral analyzer
- SILOX meta-learning
- Enterprise features

---

**Built with Rust (kernel) and TypeScript (runtime).**  
**Enterprise-grade, production-ready.**  
**Version:** 0.1.0
