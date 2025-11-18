#!/bin/bash

# AIOS Dependency Update Script
# Updates all dependencies to latest stable versions

set -e

echo "🔄 Updating AIOS Dependencies"
echo "=============================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if jq is installed (for JSON parsing)
if ! command -v jq &> /dev/null; then
	echo -e "${YELLOW}⚠️  Warning: jq is not installed. Install it for better output formatting.${NC}"
	echo ""
fi

# Show current outdated packages
echo "📋 Checking for outdated packages..."
echo ""
pnpm outdated --format=table 2>/dev/null || echo "No outdated packages found or pnpm outdated failed"
echo ""

# Confirm update
read -p "Do you want to update all dependencies to latest stable versions? (y/N): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
	echo -e "${YELLOW}Cancelled.${NC}"
	exit 0
fi

# Update dependencies
echo ""
echo "🔄 Updating dependencies..."
echo ""

# Update all dependencies to latest
pnpm update --latest

# Update dev dependencies
echo ""
echo "🔄 Updating dev dependencies..."
pnpm update --latest --dev

echo ""
echo -e "${GREEN}✅ Dependencies updated!${NC}"
echo ""
echo "📋 Updated packages:"
pnpm outdated --format=table 2>/dev/null || echo "All packages are up to date"
echo ""
echo "⚠️  Next steps:"
echo "   1. Review the changes in package.json and pnpm-lock.yaml"
echo "   2. Test your code: pnpm build && pnpm test"
echo "   3. Run pre-push checks: pnpm pre-push-check"
echo "   4. Commit the changes if everything works"
echo ""

