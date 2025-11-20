#!/usr/bin/env bash
# Build Chaos Testing Harness Script
# Builds fault injection and chaos testing tools

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

echo "🔨 Building chaos testing harness..."

# 1. Build chaos test binaries (if they exist)
if [ -d "${REPO_ROOT}/tests/chaos" ] && [ -f "${REPO_ROOT}/tests/chaos/Cargo.toml" ]; then
    print_status "Building chaos test binaries..."
    cd "${REPO_ROOT}/tests/chaos"
    cargo build --release || print_warning "Chaos tests build failed"
    cd "${REPO_ROOT}"
fi

# 2. Create chaos tools directory structure
# Tools go in tools/chaos (chaos testing helper scripts)
mkdir -p "${REPO_ROOT}/tools/chaos"
# Test artifacts go in tests/artifacts (unless CI sets ARTIFACTS_PATH)
if [ -z "${ARTIFACTS_PATH:-}" ]; then
    mkdir -p "${REPO_ROOT}/tests/artifacts/chaos"
fi

# 3. Create placeholder chaos injection scripts (if they don't exist)
# Note: These are created in tools/chaos as they're referenced by run_chaos.sh
if [ ! -f "${REPO_ROOT}/tools/chaos/inject_disk_latency.sh" ]; then
    print_status "Creating disk latency injection script..."
    cat > "${REPO_ROOT}/tools/chaos/inject_disk_latency.sh" <<'EOF'
#!/usr/bin/env bash
# Disk Latency Injection Script
# Injects artificial latency into disk I/O

set -euo pipefail

DEVICE="${1:-/dev/nbd0}"
LATENCY="${2:-200ms}"
DURATION="${3:-60}"

echo "Injecting ${LATENCY} latency on ${DEVICE} for ${DURATION}s"
# This would use tc (traffic control) or similar tools in a real environment
# For CI, we'll just log the intent
echo "Would inject latency: device=${DEVICE}, latency=${LATENCY}, duration=${DURATION}s"
EOF
    chmod +x "${REPO_ROOT}/tools/chaos/inject_disk_latency.sh"
fi

if [ ! -f "${REPO_ROOT}/tools/chaos/nic_flap.sh" ]; then
    print_status "Creating NIC flap script..."
    cat > "${REPO_ROOT}/tools/chaos/nic_flap.sh" <<'EOF'
#!/usr/bin/env bash
# NIC Flap Script
# Simulates network interface failures

set -euo pipefail

INTERFACE="${1:-eth0}"
COUNT="${2:-3}"
DELAY="${3:-5}"

echo "Flapping ${INTERFACE} ${COUNT} times with ${DELAY}s delay"
# This would use ifconfig/ip commands in a real environment
# For CI, we'll just log the intent
echo "Would flap interface: interface=${INTERFACE}, count=${COUNT}, delay=${DELAY}s"
EOF
    chmod +x "${REPO_ROOT}/tools/chaos/nic_flap.sh"
fi

if [ ! -f "${REPO_ROOT}/tools/chaos/gpu_stress.sh" ]; then
    print_status "Creating GPU stress script..."
    cat > "${REPO_ROOT}/tools/chaos/gpu_stress.sh" <<'EOF'
#!/usr/bin/env bash
# GPU Stress Script
# Simulates GPU OOM conditions

set -euo pipefail

DURATION="${1:-30}"

echo "Stressing GPU for ${DURATION}s"
# This would use GPU stress tools in a real environment
# For CI, we'll just log the intent
echo "Would stress GPU: duration=${DURATION}s"
EOF
    chmod +x "${REPO_ROOT}/tools/chaos/gpu_stress.sh"
fi

print_status "Chaos testing harness build complete"

