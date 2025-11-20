#!/usr/bin/env bash
# Build Performance Test Harness Script
# Builds performance testing tools and benchmarks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

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

echo "🔨 Building performance test harness..."

# 1. Build kernel benchmarks
if [ -d "${REPO_ROOT}/tests/kernel/benchmarks" ]; then
    print_status "Building kernel benchmarks..."
    cd "${REPO_ROOT}/tests/kernel"
    if cargo build --target x86_64-aios.json --features alloc --release --bin benchmark 2>&1; then
        print_status "Kernel benchmarks built"
    else
        print_warning "Kernel benchmarks build failed (may not be implemented)"
    fi
    cd "${REPO_ROOT}"
fi

# 2. Build performance harness (if exists as separate tool)
if [ -f "${REPO_ROOT}/tools/profiling/Cargo.toml" ]; then
    print_status "Building profiling tools..."
    cd "${REPO_ROOT}/tools/profiling"
    cargo build --release || print_warning "Profiling tools build failed"
    cd "${REPO_ROOT}"
fi

# 3. Build TypeScript performance tests (if any)
if command -v pnpm &> /dev/null || command -v npm &> /dev/null; then
    if command -v pnpm &> /dev/null; then
        PKG_MGR="pnpm"
    else
        PKG_MGR="npm"
    fi
    
    # Check for performance test packages
    for pkg_dir in packages/*/; do
        if [ -f "${pkg_dir}package.json" ]; then
            # Check if package has performance/benchmark scripts
            if grep -q "bench" "${pkg_dir}package.json" || grep -q "perf" "${pkg_dir}package.json"; then
                pkg_name=$(basename "${pkg_dir}")
                print_status "Building ${pkg_name} performance tests..."
                cd "${pkg_dir}"
                ${PKG_MGR} ci --silent || print_warning "${pkg_name} dependencies install failed"
                if ${PKG_MGR} run build 2>/dev/null; then
                    print_status "${pkg_name} built"
                else
                    print_warning "${pkg_name} build failed (may not have build script)"
                fi
                cd "${REPO_ROOT}"
            fi
        fi
    done
fi

# 4. Create performance test directory structure (if ARTIFACTS_PATH not set)
if [ -z "${ARTIFACTS_PATH:-}" ]; then
    mkdir -p "${REPO_ROOT}/tests/artifacts/perf"
fi

print_status "Performance test harness build complete"

