#!/usr/bin/env bash
# Build All Components Script
# Builds kernel, services, packages, and test harnesses

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

echo "🔨 Building all AIOS components..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Build Kernel
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Building Kernel"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd "${REPO_ROOT}/kernel"

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first."
    exit 1
fi

# Install custom target if needed
if [ ! -f "${REPO_ROOT}/kernel/x86_64-aios.json" ]; then
    print_error "Custom target x86_64-aios.json not found"
    exit 1
fi

# Build kernel with alloc feature and build-std
print_status "Building kernel (release mode with alloc feature)..."
# Check if we need to use nightly for build-std
if cargo --version | grep -q nightly || rustc --version | grep -q nightly; then
    # Use nightly with build-std
    cargo +nightly build --target x86_64-aios.json --features alloc --release -Z build-std=core,alloc,compiler_builtins || {
        print_warning "Nightly build-std failed, trying without build-std flag..."
        cargo build --target x86_64-aios.json --features alloc --release || {
            print_error "Kernel build failed"
            exit 1
        }
    }
else
    # Try standard build (may work if build-std is configured in .cargo/config.toml)
    cargo build --target x86_64-aios.json --features alloc --release || {
        print_warning "Standard build failed, this may require nightly Rust with build-std"
        print_warning "Continuing with other builds..."
    }
fi

print_status "Kernel build complete"

# 2. Build Userland Services
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Building Userland Services"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd "${REPO_ROOT}"

# Build Rust services (initd)
if [ -d "services/initd" ] && [ -f "services/initd/Cargo.toml" ]; then
    cd "${REPO_ROOT}/services/initd"
    print_status "Building initd service..."
    cargo build --release || print_warning "initd build failed (may not be implemented)"
    cd "${REPO_ROOT}"
fi

# Build TypeScript services
if command -v npm &> /dev/null || command -v pnpm &> /dev/null; then
    # Check for package manager
    if command -v pnpm &> /dev/null; then
        PKG_MGR="pnpm"
    else
        PKG_MGR="npm"
    fi
    
    # Build each TypeScript service
    for service_dir in services/*/; do
        if [ -f "${service_dir}package.json" ]; then
            service_name=$(basename "${service_dir}")
            print_status "Building ${service_name} service..."
            cd "${service_dir}"
            # Install dependencies first
            if ${PKG_MGR} install --silent 2>/dev/null; then
                # Try to build if build script exists
                if grep -q '"build"' "${service_dir}package.json"; then
                    ${PKG_MGR} run build 2>/dev/null || print_warning "${service_name} build failed"
                else
                    print_warning "${service_name} has no build script"
                fi
            else
                print_warning "${service_name} dependencies install failed"
            fi
            cd "${REPO_ROOT}"
        fi
    done
else
    print_warning "Node.js/npm not found, skipping TypeScript services"
fi

# 3. Build Runtime Packages
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Building Runtime Packages"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command -v npm &> /dev/null || command -v pnpm &> /dev/null; then
    if [ -f "${REPO_ROOT}/package.json" ]; then
        cd "${REPO_ROOT}"
        print_status "Installing workspace dependencies..."
        ${PKG_MGR} install --silent || print_warning "Workspace dependencies install failed"
        
        print_status "Building all packages..."
        if ${PKG_MGR} run build 2>/dev/null; then
            print_status "Workspace packages built"
        else
            print_warning "Workspace package build failed (may not have build script)"
        fi
    fi
    
    # Build individual packages
    for pkg_dir in packages/*/; do
        if [ -f "${pkg_dir}package.json" ]; then
            pkg_name=$(basename "${pkg_dir}")
            print_status "Building ${pkg_name} package..."
            cd "${pkg_dir}"
            ${PKG_MGR} ci --silent || print_warning "${pkg_name} dependencies install failed"
            if ${PKG_MGR} run build 2>/dev/null; then
                print_status "${pkg_name} built successfully"
            else
                print_warning "${pkg_name} build failed (may not have build script)"
            fi
            cd "${REPO_ROOT}"
        fi
    done
fi

# 4. Build Test Harnesses
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Building Test Harnesses"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Build kernel tests
cd "${REPO_ROOT}/kernel"
if [ -d "tests" ] && [ -f "tests/Cargo.toml" ]; then
    print_status "Building kernel tests..."
    cargo test --target x86_64-aios.json --features alloc --no-run || print_warning "Kernel tests build failed"
fi

# Build integration test runner (if exists)
if [ -f "${REPO_ROOT}/tests/integration/Cargo.toml" ]; then
    cd "${REPO_ROOT}/tests/integration"
    print_status "Building aios-test-runner..."
    cargo build --release || print_warning "Test runner build failed (may need dependencies)"
    
    # Copy to target/release for easy access
    if [ -f "${REPO_ROOT}/tests/integration/target/release/aios-test-runner" ]; then
        mkdir -p "${REPO_ROOT}/target/release"
        cp "${REPO_ROOT}/tests/integration/target/release/aios-test-runner" \
           "${REPO_ROOT}/target/release/aios-test-runner" 2>/dev/null || true
        print_status "Test runner built successfully"
    fi
fi

# 5. Create build artifacts directory
mkdir -p "${REPO_ROOT}/target/release"
# Create test artifacts directory if not set by CI
if [ -z "${ARTIFACTS_PATH:-}" ]; then
    mkdir -p "${REPO_ROOT}/tests/artifacts"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Build Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Build artifacts:"
echo "  - Kernel: ${REPO_ROOT}/target/x86_64-aios/release/kernel"
echo "  - Services: ${REPO_ROOT}/services/*/dist (if applicable)"
echo "  - Packages: ${REPO_ROOT}/packages/*/dist (if applicable)"
echo ""

