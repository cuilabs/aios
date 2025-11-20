#!/bin/bash
# Test Runner Script
# Runs all kernel tests

set -e

echo "🧪 Running Kernel Tests..."

# Unit tests
echo "📦 Running unit tests..."
cd tests/kernel
cargo test --features alloc --lib unit_tests

# Integration tests
echo "🔗 Running integration tests..."
cargo test --features alloc --lib integration_tests

echo "✅ All tests passed!"

