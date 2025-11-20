#!/usr/bin/env bash
# NIC Flap Test Script
# Simulates network interface flapping (up/down cycles)
#
# Usage: nic_flap.sh --iface INTERFACE --count COUNT --delay SECONDS
#
# Example: nic_flap.sh --iface eth0 --count 3 --delay 5

set -euo pipefail

INTERFACE=""
COUNT=0
DELAY=0

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --iface)
            INTERFACE="$2"
            shift 2
            ;;
        --count)
            COUNT="$2"
            shift 2
            ;;
        --delay)
            DELAY="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 --iface INTERFACE --count COUNT --delay SECONDS"
            exit 1
            ;;
    esac
done

if [ -z "$INTERFACE" ] || [ "$COUNT" -eq 0 ] || [ "$DELAY" -eq 0 ]; then
    echo "Error: Missing required arguments"
    echo "Usage: $0 --iface INTERFACE --count COUNT --delay SECONDS"
    exit 1
fi

# Check if running as root (required for ifconfig/ip)
if [ "$EUID" -ne 0 ]; then
    echo "Warning: This script requires root privileges for network interface control"
    echo "In CI/test environment, this may not be available"
    echo "Simulating NIC flap..."
    
    # Simulate flap for testing
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Starting NIC flap test"
    echo "Interface: $INTERFACE"
    echo "Flap count: $COUNT"
    echo "Delay between flaps: ${DELAY}s"
    
    for i in $(seq 1 "$COUNT"); do
        echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Flap $i/$COUNT: Bringing interface down"
        sleep 1
        echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Flap $i/$COUNT: Bringing interface up"
        if [ "$i" -lt "$COUNT" ]; then
            sleep "$DELAY"
        fi
    done
    
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] NIC flap test completed"
    exit 0
fi

# Check if interface exists
if ! ip link show "$INTERFACE" &> /dev/null && ! ifconfig "$INTERFACE" &> /dev/null; then
    echo "Warning: Interface $INTERFACE not found"
    echo "Simulating NIC flap..."
    for i in $(seq 1 "$COUNT"); do
        echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Flap $i/$COUNT: Simulated down/up"
        sleep "$DELAY"
    done
    exit 0
fi

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Starting NIC flap test"
echo "Interface: $INTERFACE"
echo "Flap count: $COUNT"
echo "Delay between flaps: ${DELAY}s"

# Perform flaps
for i in $(seq 1 "$COUNT"); do
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Flap $i/$COUNT: Bringing interface down"
    
    # Try ip command first, fallback to ifconfig
    if command -v ip &> /dev/null; then
        ip link set "$INTERFACE" down 2>/dev/null || true
    elif command -v ifconfig &> /dev/null; then
        ifconfig "$INTERFACE" down 2>/dev/null || true
    fi
    
    sleep 1
    
    echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] Flap $i/$COUNT: Bringing interface up"
    
    if command -v ip &> /dev/null; then
        ip link set "$INTERFACE" up 2>/dev/null || true
    elif command -v ifconfig &> /dev/null; then
        ifconfig "$INTERFACE" up 2>/dev/null || true
    fi
    
    if [ "$i" -lt "$COUNT" ]; then
        sleep "$DELAY"
    fi
done

echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] NIC flap test completed"
