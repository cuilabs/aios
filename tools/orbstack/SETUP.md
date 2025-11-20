# AIOS Setup for macOS with OrbStack

## Prerequisites

1. **Install OrbStack** from https://orbstack.dev
   - OrbStack provides Docker-compatible containers with better macOS integration
   - Alternative: Docker Desktop (slower but works)

2. **Install QEMU** (for running the kernel):
   ```bash
   brew install qemu
   ```

## Quick Start

### Option 1: Automated Setup (Recommended)

```bash
cd tools/orbstack
./quick_start.sh
```

This will:
- Build the OrbStack development container
- Start the container
- Optionally enter the container for you

### Option 2: Manual Setup

1. **Start the development container:**
   ```bash
   cd tools/orbstack
   docker-compose up -d --build
   ```

2. **Enter the container:**
   ```bash
   docker-compose exec aios-dev bash
   ```

3. **Build AIOS:**
   ```bash
   cd /workspace
   ./tools/build/build_all.sh
   ```

4. **Run AIOS in QEMU:**
   ```bash
   ./tools/qemu/run_aios.sh
   ```

## What Gets Built

- **Kernel**: Built with Rust, targeting `x86_64-aios`
- **Initramfs**: Contains initd and basic system files
- **Disk Image**: Bootable disk image (optional)

## Running the Kernel

The QEMU script will:
- Load the kernel
- Load the initramfs
- Start the system in QEMU
- Display output on console

**To exit QEMU:** Press `Ctrl+A` then `X`

## Troubleshooting

### Kernel won't build
- Ensure Rust toolchain is installed: `rustup install stable`
- Check that `x86_64-aios.json` target exists
- May need to install `rust-src`: `rustup component add rust-src`

### QEMU not found
- Install QEMU: `brew install qemu`
- Or use the OrbStack container which includes QEMU

### Permission errors
- Some scripts use `sudo` for disk operations
- The OrbStack container runs as root, so this shouldn't be an issue

### Build errors
- Check that all dependencies are installed
- Ensure you're using the correct Rust version (1.70+)
- Check kernel target configuration

## Next Steps

Once AIOS is running:
1. Test kernel subsystems
2. Verify userland services start
3. Test agent spawning
4. Check observability metrics

## Notes

- The kernel is a work in progress - some features may not be fully functional yet
- QEMU emulation is slower than native hardware
- For better performance, use KVM (available in OrbStack container)
- macOS doesn't support KVM, so QEMU uses software emulation

