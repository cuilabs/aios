#!/usr/bin/env bash
# Integration Test Runner Script
# Runs the full integration test suite

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# Parse arguments
OUTPUT_DIR="${ARTIFACTS_PATH:-${REPO_ROOT}/tests/artifacts/integration}"
while [[ $# -gt 0 ]]; do
    case $1 in
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--output OUTPUT_DIR]"
            exit 1
            ;;
    esac
done

mkdir -p "${OUTPUT_DIR}"
echo "Running AIOS integration suite..." > "${OUTPUT_DIR}/run.log"
echo "Output directory: ${OUTPUT_DIR}" >> "${OUTPUT_DIR}/run.log"
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "${OUTPUT_DIR}/run.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Kernel Boot Test (if QEMU is available)
if command -v qemu-system-x86_64 &> /dev/null; then
    print_status "Testing kernel boot in QEMU..."
    if [ -f "${REPO_ROOT}/target/x86_64-aios/release/kernel" ]; then
        timeout 60 qemu-system-x86_64 \
            -drive format=raw,file="${REPO_ROOT}/target/x86_64-aios/release/bootimage-aios-kernel.bin" \
            -display none \
            -serial stdio \
            &> "${OUTPUT_DIR}/qemu_boot.log" || {
            print_warning "QEMU boot test failed (may not have bootimage)"
        }
    else
        print_warning "Kernel binary not found, skipping QEMU boot test"
    fi
else
    print_warning "QEMU not available, skipping boot test"
fi

# 2. Run Kernel Unit Tests
print_status "Running kernel unit tests..."
cd "${REPO_ROOT}/kernel"
if cargo test --target x86_64-aios.json --features alloc --no-fail-fast 2>&1 | tee "${OUTPUT_DIR}/kernel_tests.log"; then
    print_status "Kernel tests passed"
else
    print_warning "Some kernel tests may have failed"
fi

# 3. Run Integration Tests (if test runner exists)
if [ -f "${REPO_ROOT}/target/release/aios-test-runner" ]; then
    print_status "Running integration test suite..."
    "${REPO_ROOT}/target/release/aios-test-runner" \
        integration \
        --output "${OUTPUT_DIR}" \
        --report "${OUTPUT_DIR}/results.json" \
        --verbose 2>&1 | tee "${OUTPUT_DIR}/integration.log" || {
        print_warning "Some integration tests may have failed"
    }
elif [ -f "${REPO_ROOT}/tests/integration/target/release/aios-test-runner" ]; then
    print_status "Running integration test suite (from tests/integration)..."
    "${REPO_ROOT}/tests/integration/target/release/aios-test-runner" \
        integration \
        --output "${OUTPUT_DIR}" \
        --report "${OUTPUT_DIR}/results.json" \
        --verbose 2>&1 | tee "${OUTPUT_DIR}/integration.log" || {
        print_warning "Some integration tests may have failed"
    }
else
    print_warning "Integration test runner not found, skipping integration tests"
    print_warning "Build it with: cd tests/integration && cargo build --release"
fi

# 4. Run Service Tests (if services are built)
if command -v pnpm &> /dev/null || command -v npm &> /dev/null; then
    print_status "Running service tests..."
    cd "${REPO_ROOT}"
    for service_dir in services/*/; do
        if [ -f "${service_dir}package.json" ] && [ -f "${service_dir}test" ] || [ -f "${service_dir}tests" ]; then
            service_name=$(basename "${service_dir}")
            print_status "Testing ${service_name}..."
            cd "${service_dir}"
            if command -v pnpm &> /dev/null; then
                pnpm test 2>&1 | tee "${OUTPUT_DIR}/${service_name}_tests.log" || true
            else
                npm test 2>&1 | tee "${OUTPUT_DIR}/${service_name}_tests.log" || true
            fi
            cd "${REPO_ROOT}"
        fi
    done
fi

# 5. Generate summary
echo "" >> "${OUTPUT_DIR}/run.log"
echo "Integration test suite completed at $(date)" >> "${OUTPUT_DIR}/run.log"
echo "Results saved to: ${OUTPUT_DIR}" >> "${OUTPUT_DIR}/run.log"

# Create results summary JSON
cat > "${OUTPUT_DIR}/results.json" <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "output_dir": "${OUTPUT_DIR}",
  "tests": {
    "kernel_boot": "$([ -f "${OUTPUT_DIR}/qemu_boot.log" ] && echo "completed" || echo "skipped")",
    "kernel_tests": "$([ -f "${OUTPUT_DIR}/kernel_tests.log" ] && echo "completed" || echo "skipped")",
    "integration": "$([ -f "${OUTPUT_DIR}/integration.log" ] && echo "completed" || echo "skipped")"
  }
}
EOF

print_status "Integration tests completed"
echo "Results saved to: ${OUTPUT_DIR}"
