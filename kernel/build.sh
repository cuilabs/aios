#!/bin/bash
# Build script for AIOS kernel

set -e

echo "Building AIOS Kernel..."

# Build the kernel
cargo build --target x86_64-aios.json

echo "Kernel build complete!"

