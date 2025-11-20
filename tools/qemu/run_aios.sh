#!/bin/bash
# Run AIOS in QEMU

set -e

echo "🚀 Starting AIOS in QEMU..."

# Configuration
KERNEL="kernel/target/x86_64-aios/release/kernel"
INITRAMFS="boot/initramfs.cpio.gz"
DISK_IMAGE="aios.img"
MEMORY="2G"
CPUS=2

# Check if kernel exists
if [ ! -f "${KERNEL}" ]; then
    echo "❌ Kernel not found. Building kernel..."
    cd kernel
    
    # Check if target JSON exists
    if [ ! -f "x86_64-aios.json" ]; then
        echo "❌ Kernel target file not found. Please ensure x86_64-aios.json exists."
        exit 1
    fi
    
    # Build kernel
    cargo build --features alloc --release --target x86_64-aios.json || {
        echo "⚠️  Kernel build failed. This is expected if the kernel target isn't fully configured yet."
        echo "   The kernel structure is in place, but may need additional configuration."
        exit 1
    }
    cd ..
fi

# Check if initramfs exists
if [ ! -f "${INITRAMFS}" ]; then
    echo "❌ Initramfs not found. Creating initramfs..."
    ./tools/build/create_initramfs.sh
fi

# QEMU command
QEMU_CMD=(
    qemu-system-x86_64
    -machine q35
    -cpu qemu64
    -smp "${CPUS}"
    -m "${MEMORY}"
    -kernel "${KERNEL}"
    -initrd "${INITRAMFS}"
    -append "console=ttyS0"
    -nographic
    -serial stdio
    -monitor unix:/tmp/qemu-monitor,server,nowait
)

# Add disk image if it exists
if [ -f "${DISK_IMAGE}" ]; then
    QEMU_CMD+=(-drive "file=${DISK_IMAGE},format=raw,if=virtio")
fi

# Enable KVM if available (faster)
if [ -c /dev/kvm ]; then
    QEMU_CMD+=(-enable-kvm)
    echo "✅ Using KVM acceleration"
fi

echo "Starting QEMU..."
echo "Command: ${QEMU_CMD[*]}"
echo ""
echo "Press Ctrl+A then X to exit QEMU"
echo ""

exec "${QEMU_CMD[@]}"

