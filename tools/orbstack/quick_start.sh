#!/bin/bash
# Quick start script for AIOS on macOS with OrbStack

set -e

echo "🚀 AIOS Quick Start for macOS"
echo ""

# Check if OrbStack/Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker/OrbStack not found. Please install OrbStack from https://orbstack.dev"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "tools/orbstack/docker-compose.yml" ]; then
    echo "❌ Please run this script from the AIOS root directory"
    exit 1
fi

echo "📦 Starting OrbStack development environment..."
cd tools/orbstack

# Build and start container
docker-compose up -d --build

echo ""
echo "✅ Container started!"
echo ""
echo "To enter the container:"
echo "  docker-compose exec aios-dev bash"
echo ""
echo "Inside the container, run:"
echo "  cd /workspace"
echo "  ./tools/build/build_all.sh"
echo "  ./tools/qemu/run_aios.sh"
echo ""

# Optionally enter the container
read -p "Enter container now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    docker-compose exec aios-dev bash
fi

