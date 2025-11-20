#!/bin/bash
# Build everything for AIOS

set -e

echo "🔨 Building AIOS..."

# Build kernel
echo "📦 Building kernel..."
cd kernel
cargo build --features alloc --release --target x86_64-aios.json
cd ..

# Build userland services
echo "📦 Building userland services..."
bun install
bun run build

# Create initramfs
echo "📦 Creating initramfs..."
./tools/build/create_initramfs.sh

echo "✅ Build complete!"
echo ""
echo "To run AIOS:"
echo "  ./tools/qemu/run_aios.sh"
echo ""
echo "Or with OrbStack:"
echo "  cd tools/orbstack && docker-compose up -d"

