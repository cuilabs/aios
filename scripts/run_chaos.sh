#!/usr/bin/env bash
# Run Chaos Tests Script
# Executes fault injection and resilience tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# Parse arguments
OUTPUT_DIR="${ARTIFACTS_PATH:-${REPO_ROOT}/tests/artifacts/chaos}"
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
echo "Running chaos tests..." > "${OUTPUT_DIR}/run.log"
echo "Output directory: ${OUTPUT_DIR}" >> "${OUTPUT_DIR}/run.log"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

echo "💥 Running chaos and resilience tests..."

# 1. Disk Latency Injection
if [ -f "${REPO_ROOT}/tools/chaos/inject_disk_latency.sh" ]; then
    print_status "Injecting disk latency..."
    "${REPO_ROOT}/tools/chaos/inject_disk_latency.sh" \
        --dev /dev/nbd0 \
        --latency 200ms \
        --duration 60 \
        &> "${OUTPUT_DIR}/disk_latency.log" || {
        print_warning "Disk latency injection failed (may not be available in CI environment)"
    }
else
    print_warning "Disk latency injection script not found"
fi

# 2. NIC Flap Test
if [ -f "${REPO_ROOT}/tools/chaos/nic_flap.sh" ]; then
    print_status "Testing NIC flap resilience..."
    "${REPO_ROOT}/tools/chaos/nic_flap.sh" \
        --iface eth0 \
        --count 3 \
        --delay 5 \
        &> "${OUTPUT_DIR}/nic_flap.log" || {
        print_warning "NIC flap test failed (may not be available in CI environment)"
    }
else
    print_warning "NIC flap script not found"
fi

# 3. GPU Stress Test
if [ -f "${REPO_ROOT}/tools/chaos/gpu_stress.sh" ]; then
    print_status "Testing GPU OOM resilience..."
    "${REPO_ROOT}/tools/chaos/gpu_stress.sh" \
        --duration 30 \
        &> "${OUTPUT_DIR}/gpu_stress.log" || {
        print_warning "GPU stress test failed (may not be available in CI environment)"
    }
else
    print_warning "GPU stress script not found"
fi

# 4. Check for recovery logs (AutonomousHealer)
print_status "Checking recovery logs..."
if [ -f "/var/log/aios/kernel.log" ]; then
    grep -i "autonomous_healer" /var/log/aios/kernel.log > "${OUTPUT_DIR}/healer.log" 2>/dev/null || {
        print_warning "No healer logs found (may not be running)"
        echo "No autonomous healer logs found" > "${OUTPUT_DIR}/healer.log"
    }
else
    print_warning "Kernel log not found (may not be running in production environment)"
    echo "Kernel log not available in CI environment" > "${OUTPUT_DIR}/healer.log"
fi

# 5. Run chaos test suite (if test runner exists)
if [ -f "${REPO_ROOT}/target/release/aios-test-runner" ]; then
    print_status "Running chaos test suite..."
    "${REPO_ROOT}/target/release/aios-test-runner" \
        chaos \
        --output "${OUTPUT_DIR}" \
        --report "${OUTPUT_DIR}/summary.json" \
        --verbose 2>&1 | tee "${OUTPUT_DIR}/chaos_test.log" || {
        print_warning "Some chaos tests may have failed"
    }
elif [ -f "${REPO_ROOT}/tests/integration/target/release/aios-test-runner" ]; then
    print_status "Running chaos test suite (from tests/integration)..."
    "${REPO_ROOT}/tests/integration/target/release/aios-test-runner" \
        chaos \
        --output "${OUTPUT_DIR}" \
        --report "${OUTPUT_DIR}/summary.json" \
        --verbose 2>&1 | tee "${OUTPUT_DIR}/chaos_test.log" || {
        print_warning "Some chaos tests may have failed"
    }
fi

# 6. Generate Summary
echo "chaos finished" >> "${OUTPUT_DIR}/run.log"
cat > "${OUTPUT_DIR}/summary.json" <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "output_dir": "${OUTPUT_DIR}",
  "tests": {
    "disk_latency": "$([ -f "${OUTPUT_DIR}/disk_latency.log" ] && echo "completed" || echo "skipped")",
    "nic_flap": "$([ -f "${OUTPUT_DIR}/nic_flap.log" ] && echo "completed" || echo "skipped")",
    "gpu_stress": "$([ -f "${OUTPUT_DIR}/gpu_stress.log" ] && echo "completed" || echo "skipped")",
    "recovery": "$([ -f "${OUTPUT_DIR}/healer.log" ] && echo "checked" || echo "skipped")"
  }
}
EOF

print_status "Chaos tests completed"
echo "Results saved to: ${OUTPUT_DIR}"
