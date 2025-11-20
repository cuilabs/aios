# Running AIOS with OrbStack

This directory contains configuration for running AIOS development environment in OrbStack.

## Quick Start

1. **Start OrbStack development container:**
   ```bash
   cd tools/orbstack
   docker-compose up -d
   docker-compose exec aios-dev bash
   ```

2. **Build the kernel:**
   ```bash
   cd /workspace/kernel
   cargo build --features alloc --release --target x86_64-aios.json
   ```

3. **Create initramfs:**
   ```bash
   cd /workspace
   ./tools/build/create_initramfs.sh
   ```

4. **Run in QEMU:**
   ```bash
   ./tools/qemu/run_aios.sh
   ```

## Alternative: Direct QEMU on macOS

If you prefer to run QEMU directly on macOS (without OrbStack):

1. **Install QEMU:**
   ```bash
   brew install qemu
   ```

2. **Build kernel:**
   ```bash
   cd kernel
   cargo build --features alloc --release --target x86_64-aios.json
   ```

3. **Run:**
   ```bash
   ./tools/qemu/run_aios.sh
   ```

## Creating Bootable Image

To create a bootable disk image:

```bash
./tools/build/create_disk_image.sh
```

This creates `aios.img` which can be used with QEMU or converted to other formats.

## Notes

- The kernel target `x86_64-aios.json` needs to be configured for your architecture
- KVM acceleration requires hardware support (not available in macOS, but works in OrbStack)
- For macOS, QEMU will use software emulation (slower but works)

