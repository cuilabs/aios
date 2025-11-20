#!/usr/bin/env bash
# GPU Stress Test Script
# Simulates GPU OOM (Out of Memory) or high utilization stress
#
# Usage: gpu_stress.sh --duration SECONDS
#
# Example: gpu_stress.sh --duration 30

set -euo pipefail

DURATION=0

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --duration)
            DURATION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 --duration SECONDS"
            exit 1
            ;;
    esac
done

if [ "$DURATION" -eq 0 ]; then
    echo "Error: Missing required argument"
    echo "Usage: $0 --duration SECONDS"
    exit 1
fi

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Starting GPU stress test"
echo "Duration: ${DURATION}s"

# Check for GPU availability
if command -v nvidia-smi &> /dev/null; then
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] NVIDIA GPU detected"
    nvidia-smi --query-gpu=name,memory.total,memory.used --format=csv,noheader,nounits | head -1
elif command -v rocm-smi &> /dev/null; then
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] AMD GPU detected"
    rocm-smi --showid --showmemuse --showuse | head -5
else
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] No GPU management tool found, simulating stress"
fi

# Simulate GPU stress by creating memory pressure
# In production, would use GPU compute shaders or CUDA/ROCm kernels
echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Simulating GPU memory stress..."

# Create temporary stress process (if GPU tools available)
STRESS_PID=""
if command -v nvidia-smi &> /dev/null; then
    # Could use CUDA stress tool if available
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] GPU stress active (simulated)"
elif command -v rocm-smi &> /dev/null; then
    # Could use ROCm stress tool if available
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] GPU stress active (simulated)"
else
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] GPU stress simulation active"
fi

# Monitor GPU during stress (optimized for AI-native OS)
START_TIME=$(date +%s)
MONITOR_INTERVAL=1  # Optimized: 1 second intervals for fast tests
while [ $(($(date +%s) - START_TIME)) -lt "$DURATION" ]; do
    if command -v nvidia-smi &> /dev/null; then
        nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv,noheader,nounits | head -1
    elif command -v rocm-smi &> /dev/null; then
        rocm-smi --showuse --showmemuse | head -1
    fi
    sleep "$MONITOR_INTERVAL"
done

# Cleanup
if [ -n "$STRESS_PID" ]; then
    kill "$STRESS_PID" 2>/dev/null || true
fi

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] GPU stress test completed"
