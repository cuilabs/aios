#!/bin/bash

# AIOS GitHub Push Script
# This script initializes the git repository and pushes to GitHub

set -e

echo "🚀 AIOS GitHub Push Script"
echo "=========================="
echo ""

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "❌ Git is not installed. Please install Git first."
    exit 1
fi

# Check if we're in the AIOS directory
if [ ! -f "README.md" ] || [ ! -f "LICENSE" ]; then
    echo "❌ Please run this script from the AIOS root directory"
    exit 1
fi

echo "📋 Pre-flight checks..."
echo ""

# Initialize git if not already done
if [ ! -d ".git" ]; then
    echo "✅ Initializing git repository..."
    git init
else
    echo "✅ Git repository already initialized"
fi

# Add remote if not already added
if ! git remote get-url origin &> /dev/null; then
    echo "✅ Adding GitHub remote..."
    git remote add origin https://github.com/cuilabs/aios.git
    echo "   Remote URL: https://github.com/cuilabs/aios.git"
else
    echo "✅ Remote already configured"
    echo "   Current remote: $(git remote get-url origin)"
    read -p "   Update remote URL? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git remote set-url origin https://github.com/cuilabs/aios.git
        echo "   ✅ Remote URL updated"
    fi
fi

echo ""
echo "📦 Staging files..."
git add .

echo ""
echo "📝 Creating initial commit..."
git commit -m "Initial open-source release

- Core kernel architecture with 24 subsystems
- Agent-first operating system foundation
- Complete documentation and architecture specs
- Legal documents (LICENSE, CLA, TRADEMARK, etc.)
- Contributing guidelines and code of conduct

Copyright (c) 2025 CUI Labs (Pte.) Ltd., Singapore"

echo ""
echo "🌿 Setting default branch to 'main'..."
git branch -M main

echo ""
echo "✅ Ready to push!"
echo ""
echo "📤 To push to GitHub, run:"
echo "   git push -u origin main"
echo ""
echo "⚠️  Make sure you have:"
echo "   1. GitHub authentication set up (SSH key or personal access token)"
echo "   2. Write access to https://github.com/cuilabs/aios"
echo ""
read -p "Push now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "🚀 Pushing to GitHub..."
    git push -u origin main
    echo ""
    echo "✅ Successfully pushed to GitHub!"
    echo "   Repository: https://github.com/cuilabs/aios"
else
    echo ""
    echo "⏸️  Skipped push. Run 'git push -u origin main' when ready."
fi

