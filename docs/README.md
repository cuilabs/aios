# AIOS Documentation

Welcome to the AIOS documentation directory. This directory contains all technical documentation for the AIOS project.

## 📖 Documentation Index

### Core Architecture
- **[Architecture](./architecture/ARCHITECTURE.md)** - Complete architecture documentation
- **[Technical Whitepaper](./architecture/AIOS_TECHNICAL_WHITEPAPER.md)** - Comprehensive technical whitepaper
- **[Boot Sequence](./architecture/BOOT_SEQUENCE.md)** - Detailed boot & attestation flow
- **[Repository Layout](./architecture/REPO_LAYOUT.md)** - Repository structure
- **[Syscall API](./architecture/SYSCALL_API.md)** - Complete syscall API with examples

### Component Documentation
- **[Missing Components](./components/MISSING_COMPONENTS.md)** - Critical missing OS components
- **[Agent-First Components](./components/AGENT_FIRST_COMPONENTS.md)** - 🔥 MANDATORY agent-first architecture components
- **[Foundational Subsystems](./components/FOUNDATIONAL_SUBSYSTEMS.md)** - 13 foundational subsystems status
- **[Subsystem Inventory](./components/SUBSYSTEM_INVENTORY.md)** - Complete inventory of all 30 subsystems
- **[Complete System Inventory](./components/COMPLETE_SYSTEM_INVENTORY.md)** - Complete inventory of 39 unique components (no overlaps)

### Setup & Getting Started
- **[GitHub Setup](./setup/GITHUB_SETUP.md)** - GitHub repository setup guide
- **[Quick Start](./setup/QUICK_START.md)** - Quick start guide for pushing to GitHub

## 📚 Documentation Structure

```
docs/
├── README.md                    # This file
├── architecture/                # Core architecture docs
│   ├── ARCHITECTURE.md
│   ├── BOOT_SEQUENCE.md
│   ├── REPO_LAYOUT.md
│   └── SYSCALL_API.md
├── components/                  # Component documentation
│   ├── MISSING_COMPONENTS.md
│   ├── AGENT_FIRST_COMPONENTS.md
│   ├── FOUNDATIONAL_SUBSYSTEMS.md
│   ├── SUBSYSTEM_INVENTORY.md
│   └── COMPLETE_SYSTEM_INVENTORY.md
├── setup/                      # Setup guides
│   ├── GITHUB_SETUP.md
│   └── QUICK_START.md
└── business/                   # Business docs (PRIVATE - not in public repo)
```

## 🔍 Quick Links

### For Developers
- Start with [Architecture](./architecture/ARCHITECTURE.md) to understand the system
- Read [Syscall API](./architecture/SYSCALL_API.md) for kernel API details
- Check [Repository Layout](./architecture/REPO_LAYOUT.md) for code organization

### For Contributors
- See [../CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines
- Review [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) for community standards
- Check [../SECURITY.md](../SECURITY.md) for security reporting

### For Understanding Components
- Review [Missing Components](./components/MISSING_COMPONENTS.md) to see what's needed
- Check [Agent-First Components](./components/AGENT_FIRST_COMPONENTS.md) for mandatory features
- See [Foundational Subsystems](./components/FOUNDATIONAL_SUBSYSTEMS.md) for implementation status

---

**Copyright (c) 2025 CUI Labs (Pte.) Ltd., Singapore**
