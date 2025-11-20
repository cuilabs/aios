#!/bin/bash
# Create initramfs for AIOS

set -e

echo "📦 Creating AIOS initramfs..."

INITRAMFS_DIR="/tmp/aios_initramfs"
INITRAMFS_FILE="boot/initramfs.cpio.gz"

# Clean and create initramfs directory
rm -rf "${INITRAMFS_DIR}"
mkdir -p "${INITRAMFS_DIR}"/{bin,sbin,etc,lib,proc,sys,dev,tmp}

# Copy initd
echo "Copying initd..."
cp services/initd/target/release/initd "${INITRAMFS_DIR}/sbin/init" || echo "initd not built yet"

# Create init script
cat > "${INITRAMFS_DIR}/init" << 'EOF'
#!/bin/sh
# AIOS init script

mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t devtmpfs devtmpfs /dev

# Start initd
exec /sbin/init
EOF

chmod +x "${INITRAMFS_DIR}/init"

# Create basic /etc structure
echo "Creating /etc structure..."
cat > "${INITRAMFS_DIR}/etc/hostname" << 'EOF'
aios
EOF

# Create CPIO archive
echo "Creating CPIO archive..."
cd "${INITRAMFS_DIR}"
find . -print0 | cpio -0 -ov -H newc | gzip > "../../${INITRAMFS_FILE}"
cd - > /dev/null

echo "✅ Initramfs created: ${INITRAMFS_FILE}"

