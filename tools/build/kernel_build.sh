#!/bin/bash
# Kernel Build Script
# Builds the AIOS kernel with proper feature flags

set -e

echo "🔨 Building AIOS Kernel..."

# Build with alloc feature
cargo build --features alloc --release

# Run tests
echo "🧪 Running tests..."
cargo test --features alloc

# Check formatting
echo "📝 Checking formatting..."
cargo fmt --check

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --features alloc -- -D warnings

echo "✅ Build complete!"

