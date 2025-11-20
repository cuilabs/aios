#!/usr/bin/env bash
# Collect Artifacts Script
# Collects and packages test artifacts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Parse arguments
SRC_DIR="${1:-${REPO_ROOT}/tests/artifacts}"
DST_DIR="${2:-${REPO_ROOT}/tests/collected}"

cd "${REPO_ROOT}"

# Create destination directory
mkdir -p "${DST_DIR}"

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

# Copy artifacts
if [ -d "${SRC_DIR}" ]; then
    print_status "Collecting artifacts from ${SRC_DIR}..."
    cp -r "${SRC_DIR}"/* "${DST_DIR}/" 2>/dev/null || {
        # If source is empty or copy fails, create empty marker
        echo "No artifacts found in ${SRC_DIR}" > "${DST_DIR}/no_artifacts.txt"
    }
else
    print_status "Source directory ${SRC_DIR} not found, creating empty collection"
    echo "Source directory ${SRC_DIR} not found" > "${DST_DIR}/no_artifacts.txt"
fi

# Compress logs and artifacts
if [ "$(ls -A ${DST_DIR} 2>/dev/null)" ]; then
    TIMESTAMP=$(date +%s)
    ARCHIVE_NAME="collected-${TIMESTAMP}.tgz"
    
    print_status "Compressing artifacts..."
    tar -czf "${DST_DIR}/${ARCHIVE_NAME}" -C "${DST_DIR}" . 2>/dev/null || {
        print_status "Compression skipped (may not have tar)"
    }
    
    # Create summary
    cat > "${DST_DIR}/summary.txt" <<EOF
Artifacts Collection Summary
============================
Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Source: ${SRC_DIR}
Destination: ${DST_DIR}
Archive: ${ARCHIVE_NAME}
Files collected: $(find "${DST_DIR}" -type f ! -name "*.tgz" ! -name "summary.txt" | wc -l)
EOF
    
    print_status "Artifacts collected to ${DST_DIR}"
    print_status "Archive: ${ARCHIVE_NAME}"
else
    print_status "No artifacts to collect"
fi

echo "Artifacts collected to ${DST_DIR}"
