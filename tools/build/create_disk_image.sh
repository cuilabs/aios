#!/bin/bash
# Create bootable disk image for AIOS

set -e

echo "📦 Creating AIOS disk image..."

# Configuration
IMAGE_SIZE="1G"
IMAGE_NAME="aios.img"
MOUNT_POINT="/tmp/aios_mount"

# Create disk image
echo "Creating ${IMAGE_SIZE} disk image..."
dd if=/dev/zero of="${IMAGE_NAME}" bs=1M count=1024 status=progress

# Create partition table (GPT)
echo "Creating partition table..."
parted -s "${IMAGE_NAME}" mklabel gpt
parted -s "${IMAGE_NAME}" mkpart primary ext4 1MiB 100%

# Setup loop device
LOOP_DEVICE=$(sudo losetup -f --show "${IMAGE_NAME}")
LOOP_PARTITION="${LOOP_DEVICE}p1"

# Create filesystem
echo "Creating ext4 filesystem..."
sudo mkfs.ext4 -F "${LOOP_PARTITION}"

# Mount image
echo "Mounting image..."
mkdir -p "${MOUNT_POINT}"
sudo mount "${LOOP_PARTITION}" "${MOUNT_POINT}"

# Create directory structure
echo "Creating directory structure..."
sudo mkdir -p "${MOUNT_POINT}"/{boot,bin,sbin,etc,lib,usr/{bin,sbin,lib},var,proc,sys,dev,tmp,root}

# Copy kernel
echo "Copying kernel..."
sudo cp kernel/target/x86_64-aios/release/kernel "${MOUNT_POINT}/boot/"

# Copy initramfs
echo "Copying initramfs..."
sudo cp boot/initramfs.cpio.gz "${MOUNT_POINT}/boot/" || echo "Initramfs not found, will create later"

# Install GRUB or bootloader
echo "Installing bootloader..."
# TODO: Install GRUB or custom bootloader

# Unmount
echo "Unmounting..."
sudo umount "${MOUNT_POINT}"
sudo losetup -d "${LOOP_DEVICE}"

echo "✅ Disk image created: ${IMAGE_NAME}"

