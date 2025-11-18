#!/bin/bash

# AIOS Pre-Push Check Script
# Run this manually to check before pushing

set -e

echo "🔍 AIOS Pre-Push Checks"
echo "========================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERROR_COUNT=0

# Function to check and report errors
check_error() {
	if [ $? -ne 0 ]; then
		echo -e "${RED}❌ FAILED: $1${NC}"
		ERROR_COUNT=$((ERROR_COUNT + 1))
		return 1
	else
		echo -e "${GREEN}✅ PASSED: $1${NC}"
		return 0
	fi
}

# 1. Build Check
echo "1️⃣  Checking build..."
pnpm build
check_error "Build check"
echo ""

# 2. Lint Check
echo "2️⃣  Checking lint..."
pnpm lint
check_error "Lint check"
echo ""

# 3. TypeScript Check
echo "3️⃣  Checking TypeScript..."
pnpm typecheck
check_error "TypeScript check"
echo ""

# 4. Audit Check
echo "4️⃣  Checking security audit..."
if pnpm audit --audit-level=moderate > /dev/null 2>&1; then
	echo -e "${GREEN}✅ PASSED: Security audit${NC}"
else
	echo -e "${YELLOW}⚠️  WARNING: Security audit found vulnerabilities${NC}"
	echo "   Run 'pnpm audit' for details"
fi
echo ""

# 5. Dependency Update Check
echo "5️⃣  Checking for outdated dependencies..."
OUTDATED_OUTPUT=$(pnpm outdated --format=table 2>/dev/null || echo "")
OUTDATED_LINES=$(echo "$OUTDATED_OUTPUT" | grep -E "^\│" | grep -v "Package" | grep -v "────" | wc -l | tr -d ' ')

if [ "$OUTDATED_LINES" = "0" ] || [ -z "$OUTDATED_LINES" ]; then
	echo -e "${GREEN}✅ PASSED: All dependencies are up to date${NC}"
else
	echo -e "${RED}❌ FAILED: Found outdated dependency/dependencies${NC}"
	echo -e "${YELLOW}   Run 'pnpm outdated' to see details${NC}"
	echo -e "${YELLOW}   Run 'pnpm update-deps' to update all dependencies${NC}"
	echo ""
	echo "   Outdated packages:"
	echo "$OUTDATED_OUTPUT" | grep -E "^\│" | grep -v "Package" | grep -v "────" | head -10 | sed 's/^/   /' || true
	ERROR_COUNT=$((ERROR_COUNT + 1))
fi
echo ""

# 6. Code Quality Check - No Placeholder Code
echo "6️⃣  Checking for placeholder/fake code..."
echo "   Scanning for: TODO, FIXME, mock, placeholder, simulation, etc."

PLACEHOLDER_PATTERNS=(
	"TODO"
	"FIXME"
	"XXX"
	"\\bmock\\b"
	"\\bMock\\b"
	"\\bMOCK\\b"
	"\\bplaceholder\\b"
	"\\bPlaceholder\\b"
	"\\bPLACEHOLDER\\b"
	"\\bsimulation\\b"
	"\\bSimulation\\b"
	"\\bSIMULATION\\b"
	"\\bstub\\b"
	"\\bStub\\b"
	"\\bSTUB\\b"
	"\\bfake\\b"
	"\\bFake\\b"
	"\\bFAKE\\b"
	"\\bdummy\\b"
	"\\bDummy\\b"
	"\\bDUMMY\\b"
	"\\btest\\s+only"
	"\\bskip\\s+test"
	"\\bin\\s+production"
	"\\bnot\\s+implemented"
	"\\bNotImplemented"
	"\\bnot\\s+ready"
	"\\bNotReady"
	"\\btemporary"
	"\\bTemporary"
	"\\bTEMPORARY"
)

PLACEHOLDER_ERRORS=0
EXCLUDED_PATHS=(
	"node_modules"
	"dist"
	"build"
	".git"
	".changeset"
	"docs"
	"*.md"
	"*.json"
	"*.lock"
	"*.log"
	"CHANGELOG"
	"CONTRIBUTING"
	"CODE_OF_CONDUCT"
	"SECURITY"
	"LICENSE"
	"NOTICE"
	"CLA"
	"TRADEMARK"
	".github"
	"scripts"
)

# Get list of files to check
FILES_TO_CHECK=$(find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.rs" -o -name "*.js" -o -name "*.jsx" \) ! -path "*/node_modules/*" ! -path "*/dist/*" ! -path "*/.git/*" ! -path "*/.changeset/*" ! -path "*/docs/*" ! -path "*/scripts/*" 2>/dev/null || echo "")

for file in $FILES_TO_CHECK; do
	# Skip excluded paths
	skip=false
	for excluded in "${EXCLUDED_PATHS[@]}"; do
		if [[ "$file" == *"$excluded"* ]] || [[ "$file" == "$excluded"* ]]; then
			skip=true
			break
		fi
	done
	
	if [ "$skip" = true ]; then
		continue
	fi
	
	# Check if file exists and is readable
	if [ ! -f "$file" ] || [ ! -r "$file" ]; then
		continue
	fi
	
	# Check for placeholder patterns
	for pattern in "${PLACEHOLDER_PATTERNS[@]}"; do
		if grep -iE "$pattern" "$file" > /dev/null 2>&1; then
			# Allow TODO/FIXME in comments for documentation purposes only
			if [[ "$pattern" == "TODO" ]] || [[ "$pattern" == "FIXME" ]]; then
				# Check if it's in a comment (allow in markdown/docs)
				if [[ "$file" == *.md ]] || [[ "$file" == *.mdx ]]; then
					continue
				fi
				# Check if it's in a Rust comment
				if [[ "$file" == *.rs ]]; then
					# Allow in doc comments but not in regular code
					if ! grep -iE "^\s*//.*$pattern|^\s*///.*$pattern|^\s*/\*.*$pattern" "$file" > /dev/null 2>&1; then
						echo -e "${RED}   ❌ Found $pattern in: $file${NC}"
						grep -n -iE "$pattern" "$file" | head -3 | sed 's/^/      /'
						PLACEHOLDER_ERRORS=$((PLACEHOLDER_ERRORS + 1))
					fi
				else
					# For other files, be strict
					echo -e "${RED}   ❌ Found $pattern in: $file${NC}"
					grep -n -iE "$pattern" "$file" | head -3 | sed 's/^/      /'
					PLACEHOLDER_ERRORS=$((PLACEHOLDER_ERRORS + 1))
				fi
			else
				# For other patterns, always fail
				echo -e "${RED}   ❌ Found $pattern in: $file${NC}"
				grep -n -iE "$pattern" "$file" | head -3 | sed 's/^/      /'
				PLACEHOLDER_ERRORS=$((PLACEHOLDER_ERRORS + 1))
			fi
		fi
	done
done

if [ $PLACEHOLDER_ERRORS -gt 0 ]; then
	echo -e "${RED}❌ FAILED: Code quality check - Found $PLACEHOLDER_ERRORS placeholder/fake code instances${NC}"
	echo -e "${YELLOW}   ⚠️  Enterprise-grade production code required. Please implement full functionality.${NC}"
	ERROR_COUNT=$((ERROR_COUNT + PLACEHOLDER_ERRORS))
else
	echo -e "${GREEN}✅ PASSED: Code quality check - No placeholder/fake code found${NC}"
fi
echo ""

# Summary
echo "========================"
if [ $ERROR_COUNT -eq 0 ]; then
	echo -e "${GREEN}✅ All pre-push checks passed!${NC}"
	echo ""
	exit 0
else
	echo -e "${RED}❌ Pre-push checks failed with $ERROR_COUNT error(s)${NC}"
	echo ""
	echo -e "${YELLOW}Please fix the errors above before pushing.${NC}"
	echo ""
	exit 1
fi

