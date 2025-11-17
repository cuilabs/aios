# AIOS Repository Layout

**Production-grade repository structure for kernel, runtime, and services**

## Directory Structure

```
AIOS/
├── boot/                          # Boot components
│   ├── bootloader/               # Signed bootloader
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── README.md
│   └── initramfs/                # Initial ram filesystem
│       ├── initd/                 # Init daemon
│       ├── tools/                 # Minimal userspace tools
│       └── config/
│
├── kernel/                        # Rust kernel
│   ├── Cargo.toml                 # Workspace root
│   ├── x86_64-aios.json          # Custom target
│   ├── .cargo/
│   │   └── config.toml
│   ├── crates/
│   │   ├── kernel-core/           # Core kernel
│   │   │   ├── src/
│   │   │   │   ├── boot.rs       # Boot sequence
│   │   │   │   ├── memory/       # Memory management
│   │   │   │   ├── interrupts.rs # Interrupt handling
│   │   │   │   ├── syscall.rs    # Syscall interface
│   │   │   │   ├── attestation.rs # Attestation
│   │   │   │   └── lib.rs
│   │   │   └── Cargo.toml
│   │   ├── kernel-hal/           # Hardware abstraction
│   │   │   ├── src/
│   │   │   │   ├── apic.rs
│   │   │   │   ├── timer.rs
│   │   │   │   ├── pcie.rs
│   │   │   │   ├── iommu.rs
│   │   │   │   ├── acpi.rs
│   │   │   │   ├── storage.rs
│   │   │   │   └── network.rs
│   │   │   └── Cargo.toml
│   │   ├── kernel-agent/         # Agent management
│   │   │   ├── src/
│   │   │   │   ├── agent.rs
│   │   │   │   ├── scheduler.rs
│   │   │   │   └── lifecycle.rs
│   │   │   └── Cargo.toml
│   │   ├── kernel-ipc/            # Binary IPC
│   │   │   ├── src/
│   │   │   │   ├── message.rs
│   │   │   │   └── bus.rs
│   │   │   └── Cargo.toml
│   │   ├── kernel-crypto/        # PQC syscalls
│   │   │   ├── src/
│   │   │   │   └── lib.rs
│   │   │   └── Cargo.toml
│   │   └── kernel-drivers/        # Device drivers
│   │       ├── src/
│   │       │   ├── tpm.rs
│   │       │   ├── storage.rs
│   │       │   └── network.rs
│   │       └── Cargo.toml
│   ├── build.sh                   # Build script
│   └── README.md
│
├── services/                      # Userland services
│   ├── initd/                     # Init daemon (PID 1)
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── identityd/                 # Identity service
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   ├── memoryd/                   # Memory fabric service
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   ├── semantic-ipcd/             # Semantic IPC daemon
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   ├── planner/                   # Planning service
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   ├── agentsupervisor/           # Agent supervisor
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   └── networkd/                  # Network service
│       ├── src/
│       ├── package.json
│       └── tsconfig.json
│
├── packages/                      # TypeScript packages
│   ├── kernel/                    # Kernel API bindings
│   ├── cognitive/                 # Cognitive runtime
│   ├── orchestration/             # Agent orchestration
│   ├── memory/                    # Memory fabric
│   ├── ipc/                       # Semantic IPC
│   ├── security/                  # Security model
│   └── application/               # Application layer
│
├── apps/                          # Applications
│   ├── runtime/                   # Main runtime
│   └── devvm/                     # Development VM
│
├── sdk/                           # SDKs
│   ├── rust/                      # Rust SDK
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── typescript/                # TypeScript SDK
│   │   ├── src/
│   │   └── package.json
│   └── python/                    # Python SDK
│       ├── src/
│       └── pyproject.toml
│
├── tests/                         # Tests
│   ├── kernel/                    # Kernel tests
│   │   ├── unit/
│   │   ├── integration/
│   │   └── fuzz/
│   ├── services/                  # Service tests
│   └── e2e/                       # End-to-end tests
│
├── docs/                          # Documentation
│   ├── architecture/
│   ├── api/
│   ├── guides/
│   └── examples/
│
├── tools/                         # Development tools
│   ├── build/                     # Build tools
│   ├── test/                      # Testing tools
│   └── fuzz/                      # Fuzzing tools
│
├── .github/                       # GitHub workflows
│   └── workflows/
│
├── ARCHITECTURE.md                # Architecture doc
├── BOOT_SEQUENCE.md              # Boot sequence doc
├── REPO_LAYOUT.md                # This file
├── SYSCALL_API.md                # Syscall API doc
├── LICENSE                       # License
└── README.md                     # Main README
```

## Key Directories

### `boot/`
- **bootloader/**: Signed bootloader that verifies kernel
- **initramfs/**: Initial ram filesystem with initd and tools

### `kernel/`
- **crates/kernel-core/**: Core kernel functionality
- **crates/kernel-hal/**: Hardware abstraction layer
- **crates/kernel-agent/**: Agent management
- **crates/kernel-ipc/**: Binary IPC
- **crates/kernel-crypto/**: PQC syscalls
- **crates/kernel-drivers/**: Device drivers

### `services/`
- **initd/**: Init daemon (PID 1)
- **identityd/**: Identity service
- **memoryd/**: Memory fabric service
- **semantic-ipcd/**: Semantic IPC daemon
- **planner/**: Planning service
- **agentsupervisor/**: Agent supervisor
- **networkd/**: Network service

### `sdk/`
- **rust/**: Rust SDK for kernel development
- **typescript/**: TypeScript SDK for userland
- **python/**: Python SDK for agents

### `tests/`
- **kernel/**: Kernel unit/integration/fuzz tests
- **services/**: Service tests
- **e2e/**: End-to-end tests

---

**Version:** 0.1.0  
**Last Updated:** August 2025

