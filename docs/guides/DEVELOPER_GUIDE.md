# Developer Guide

## Getting Started

### Prerequisites

- Rust 1.70+ (for kernel development)
- TypeScript 5.0+ (for userland services)
- Bun or Node.js 20+ (for runtime)
- QEMU (for testing)

### Building the Kernel

```bash
cd kernel
cargo build --features alloc
```

### Running Tests

```bash
cd tests/kernel
cargo test --features alloc
```

## Kernel Development

### Adding a New Subsystem

1. Create a new crate in `kernel/crates/`:

```bash
mkdir kernel/crates/kernel-<name>
cd kernel/crates/kernel-<name>
cargo init --name aios-kernel-<name>
```

2. Add to `kernel/Cargo.toml` workspace:

```toml
[workspace]
members = [
    "crates/kernel-<name>",
    # ...
]
```

3. Implement the subsystem following the pattern:
   - `lib.rs`: Public API
   - `mod.rs`: Internal modules
   - Use `spin::Mutex` for synchronization
   - Use `spin::Once` for singletons
   - Add `#[cfg(feature = "alloc")]` for dynamic allocation

### Code Style

- Follow Rust standard formatting
- Use `cargo fmt` before committing
- Use `cargo clippy` for linting
- Document all public APIs
- Add unit tests for new code

### Testing

- Unit tests: `tests/kernel/unit/`
- Integration tests: `tests/kernel/integration/`
- Use the test framework from `test_framework.rs`

## Userland Service Development

### Creating a New Service

1. Create service directory:

```bash
mkdir services/<service-name>
cd services/<service-name>
```

2. Create `package.json`:

```json
{
  "name": "@aios/services/<service-name>",
  "version": "0.1.0",
  "type": "module",
  "main": "src/index.ts"
}
```

3. Implement service with IPC handlers:

```typescript
import { SemanticMessageBus } from "@aios/ipc";

export class ServiceName {
  private readonly messageBus: SemanticMessageBus;
  
  async start(): Promise<void> {
    await this.messageBus.subscribe("operation", async (message) => {
      // Handle operation
    });
  }
}
```

## Debugging

### Kernel Debugging

- Use `log::debug!()` for debug output
- Enable logging in boot sequence
- Use QEMU monitor for low-level debugging

### Service Debugging

- Use `console.log()` for service debugging
- Enable verbose logging in service configuration
- Use observability system for metrics

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes following code style
4. Add tests
5. Update documentation
6. Submit pull request

## Resources

- [Architecture Documentation](../architecture/)
- [API Documentation](../api/)
- [Testing Guide](../testing/)

