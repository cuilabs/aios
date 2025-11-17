# AIOS Boot & Attestation Sequence

**Production-grade boot flow with secure boot, TPM measured boot, and remote attestation**

## Boot Sequence Overview

```
UEFI → Bootloader → Kernel → Init → Services → Agents
  ↓        ↓          ↓        ↓        ↓         ↓
Secure   Verify    Measure  Verify  Attest   Register
Boot     Kernel    TPM      Attest  Remote   Agents
```

## Step-by-Step Boot Flow

### 1. UEFI Secure Boot

**Location:** Firmware (UEFI)

**Actions:**
- UEFI verifies bootloader signature (signed with vendor key)
- UEFI loads signed bootloader
- Measured boot entries written to TPM PCRs:
  - PCR[0]: Bootloader code
  - PCR[1]: Bootloader configuration
  - PCR[2]: UEFI configuration

**TPM PCRs Used:**
- `PCR[0]`: Bootloader code hash
- `PCR[1]`: Bootloader configuration hash
- `PCR[2]`: UEFI configuration hash

**Exit:** Bootloader loaded and measured

---

### 2. Bootloader (Signed, Minimal)

**Location:** `boot/bootloader/`

**Actions:**
1. Verify kernel image signature
2. Load signed initramfs
3. Load kernel and minimal userspace tools
4. Set up kernel command line
5. Pass device tree/handoff data
6. Measure kernel and initramfs → extend TPM PCRs:
   - `PCR[3]`: Kernel code hash
   - `PCR[4]`: Initramfs hash
   - `PCR[5]`: Kernel command line hash

**TPM PCRs Extended:**
- `PCR[3]`: Kernel code
- `PCR[4]`: Initramfs
- `PCR[5]`: Kernel command line

**Exit:** Kernel loaded and measured, ready for handoff

---

### 3. Kernel Entry (Rust Microkernel)

**Location:** `kernel/crates/kernel-core/src/boot.rs`

**Initialization Order:**

#### 3.1 Low-Level HAL
```rust
kernel_hal::init();
```
- Initialize APIC (interrupt controller)
- Initialize timers (HPET/TSC)
- Initialize PCIe enumeration
- Initialize IOMMU setup
- Parse ACPI tables

#### 3.2 Physical Memory Manager
```rust
memory::physical::init();
```
- Detect available memory
- Initialize frame allocator
- Set up memory map

#### 3.3 Virtual Memory
```rust
memory::virtual_mem::init();
```
- Initialize page tables
- Identity map kernel region
- Set up virtual memory structures

#### 3.4 Minimal Device Drivers
```rust
drivers::init();
```
- Disk drivers (SATA/NVMe) for initramfs access
- Network stack initialization
- TPM driver (for attestation)
- RNG (hardware random number generator)

#### 3.5 Secure Enclave Manager
```rust
enclave::init();
```
- Detect TEEs (SGX/SEV-like or external HSM)
- Create enclave runtime
- Initialize secure enclave interface

**TPM PCRs Extended:**
- `PCR[6]`: Kernel initialization hash
- `PCR[7]`: Device driver hashes

**Exit:** Kernel fully initialized, ready for init

---

### 4. Kernel Measurements & Attestation

**Location:** `kernel/crates/kernel-core/src/attestation.rs`

**Actions:**
1. Kernel measures critical modules:
   - Kernel core hash
   - HAL hash
   - Driver hashes
   - Enclave manager hash
2. Extend TPM PCRs:
   - `PCR[8]`: Kernel modules hash
3. Generate attestation evidence:
   - Quote from TPM (if available)
   - Enclave attestation (if available)
4. Provide evidence to bootloader/remote verifier

**Attestation Evidence:**
```rust
pub struct AttestationEvidence {
    pub tpm_quote: Option<TpmQuote>,
    pub enclave_attestation: Option<EnclaveAttestation>,
    pub pcr_values: [u8; 24 * 20], // 24 PCRs × 20 bytes
}
```

**Exit:** Attestation evidence ready

---

### 5. Init Process (PID 1)

**Location:** `services/initd/`

**Actions:**

#### 5.1 Launch Init
- Launch `initd` from initramfs (signed)
- Verify initd signature
- Measure initd → extend TPM PCR[9]

#### 5.2 Verify Kernel Attestation
- Verify kernel attestation evidence
- Optional: Remote attestation to management server
- Record attestation result

#### 5.3 Initialize Runtime Interfaces
- Mount pseudo-filesystems:
  - `/sys/aios` - AIOS system interface
  - `/proc/aios` - Agent process information
  - `/dev/aios` - AIOS device nodes
- Create runtime directories

#### 5.4 Start Privileged Userland Services

**Service Startup Order:**

1. **identityd** (`/svc/identityd`)
   - Agent identity provisioning
   - Talks to secure enclave/HSM
   - Provides identity management API

2. **memoryd** (`/svc/memoryd`)
   - Semantic memory service (privileged)
   - Uses kernel memory primitives
   - Provides memory fabric API

3. **semantic-ipcd** (`/svc/semantic-ipcd`)
   - Semantic IPC daemon
   - Interprets binary IPC messages
   - Routes by semantic intent

4. **planner** (`/svc/planner`)
   - Privileged planning service
   - Agent planning and execution
   - Called via IPC

5. **agentsupervisor** (`/svc/agentsupervisor`)
   - Binds to kernel agent syscalls
   - Manages agent images
   - Coordinates agent lifecycle

#### 5.5 Register with Remote Attestation
- Register with remote attestation server
- Publish system state
- Receive attestation verification

**TPM PCRs Extended:**
- `PCR[9]`: Initd hash
- `PCR[10]`: Service hashes

**Exit:** All services running, system ready

---

### 6. Secure Key Provisioning

**Location:** `services/identityd/`

**Actions:**
1. `identityd` requests key material from secure enclave/HSM
2. Keys provisioned via kernel KMS (Key Management Service) interface
3. Keys never exposed in clear to initd
4. Attestations performed for key provisioning
5. Keys stored in secure enclave

**Key Provisioning Flow:**
```
identityd → Kernel KMS → Secure Enclave/HSM
    ↓            ↓              ↓
  Request    Validate      Provision
    ↓            ↓              ↓
  Attest    Attestation    Key Material
    ↓            ↓              ↓
  Verify    Verify         Store (encrypted)
```

**Exit:** Keys provisioned and secured

---

### 7. Agent Supervisor Launch & Agent Registration

**Location:** `services/agentsupervisor/`

**Actions:**

#### 7.1 Supervisor Registration
1. `agentsupervisor` registers with kernel via `AgentSupervisorRegister` syscall
2. Obtains privileged capability tokens to manage agents
3. Receives supervisor ID from kernel

#### 7.2 Agent Image Loading
1. `agentsupervisor` coordinates loading of agent images
2. Verifies signatures:
   - Agent image signature
   - Manifest signature
3. Verifies provenance:
   - Signed manifests
   - Supply chain attestation
4. Loads agent into memory

#### 7.3 Agent Registration
1. Register agent with kernel via `AgentRegister` syscall
2. Agent receives capability tokens
3. Agent assigned agent ID

**Exit:** Agents registered and ready

---

### 8. Service Discovery & Network

**Location:** `services/networkd/`

**Actions:**
1. Kernel network stack initialized
2. `networkd` starts
3. Network interfaces configured
4. Service discovery enabled
5. Optional: Operator performs remote attestation
6. Optional: Register node with control plane

**Exit:** Network operational, services discoverable

---

### 9. Operational Mode

**System State:** Normal operational mode

**Agent Operations:**
- Agents spawn via `agentsupervisor` or authenticated users
- All actions via `AgentSpawn` syscall (via supervisor)
- Agents communicate via semantic IPC
- Memory operations via `/svc/memoryd`
- Planning via `/svc/planner`

**Audit & Governance:**
- All actions appended to signed audit journal (local)
- Optional: Anchored to remote ledger
- Immutable audit trail maintained

**Exit:** System fully operational

---

## TPM PCR Usage Summary

| PCR | Content | Extended By |
|-----|---------|------------|
| PCR[0] | Bootloader code | UEFI |
| PCR[1] | Bootloader config | UEFI |
| PCR[2] | UEFI config | UEFI |
| PCR[3] | Kernel code | Bootloader |
| PCR[4] | Initramfs | Bootloader |
| PCR[5] | Kernel cmdline | Bootloader |
| PCR[6] | Kernel init | Kernel |
| PCR[7] | Device drivers | Kernel |
| PCR[8] | Kernel modules | Kernel |
| PCR[9] | Initd | Init |
| PCR[10] | Services | Init |
| PCR[11-23] | Reserved for future use | - |

---

## Attestation Flow

```
┌─────────┐
│  UEFI   │ → Secure Boot → TPM PCR[0-2]
└─────────┘
     ↓
┌─────────┐
│Bootloader│ → Verify Kernel → TPM PCR[3-5]
└─────────┘
     ↓
┌─────────┐
│ Kernel  │ → Measure & Attest → TPM PCR[6-8] → Evidence
└─────────┘
     ↓
┌─────────┐
│  Init   │ → Verify Attestation → Start Services → TPM PCR[9-10]
└─────────┘
     ↓
┌─────────┐
│Services │ → Remote Attestation → Control Plane
└─────────┘
```

---

## Security Properties

1. **Chain of Trust:** UEFI → Bootloader → Kernel → Init → Services
2. **Measured Boot:** Every stage measured and extended to TPM
3. **Remote Attestation:** System state verifiable remotely
4. **Secure Key Provisioning:** Keys never exposed in clear
5. **Immutable Audit:** All actions logged and signed

---

**Version:** 0.1.0  
**Last Updated:** August 2025

