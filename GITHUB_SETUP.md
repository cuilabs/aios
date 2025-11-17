# GitHub Setup Guide

**Repository:** https://github.com/cuilabs/aios

## Pre-Push Checklist

✅ All legal documents created (LICENSE, NOTICE, CLA, etc.)  
✅ Company information updated (CUI Labs (Pte.) Ltd., Singapore)  
✅ Contact information updated (contact@cuilabs.io, https://cuilabs.io)  
✅ GitHub URLs updated (github.com/cuilabs/aios)  
✅ README.md updated with copyright and trademark notices  

## Initial Push Steps

### 1. Initialize Git Repository (if not already done)

```bash
cd /Users/christopherfrost/Desktop/AIOS
git init
```

### 2. Add Remote Repository

```bash
git remote add origin https://github.com/cuilabs/aios.git
```

Or if using SSH:
```bash
git remote add origin git@github.com:cuilabs/aios.git
```

### 3. Add All Files

```bash
git add .
```

### 4. Create Initial Commit

```bash
git commit -m "Initial open-source release

- Core kernel architecture with 24 subsystems
- Agent-first operating system foundation
- Complete documentation and architecture specs
- Legal documents (LICENSE, CLA, TRADEMARK, etc.)
- Contributing guidelines and code of conduct

Copyright (c) 2025 CUI Labs (Pte.) Ltd., Singapore"
```

### 5. Push to GitHub

```bash
git branch -M main
git push -u origin main
```

## Post-Push Setup

### 1. GitHub Repository Settings

1. **Go to Settings → General**
   - Enable Issues
   - Enable Discussions
   - Enable Wiki (optional)
   - Enable Projects (optional)

2. **Go to Settings → Branches**
   - Add branch protection rule for `main` branch
   - Require pull request reviews
   - Require status checks (if you set up CI/CD)
   - Require signed commits (optional but recommended)

3. **Go to Settings → Security**
   - Enable dependency graph
   - Enable Dependabot alerts
   - Enable secret scanning

### 2. Create GitHub Topics

Add topics to repository:
- `operating-system`
- `kernel`
- `rust`
- `ai`
- `agents`
- `aios`
- `post-quantum-crypto`
- `agent-first`

### 3. Create GitHub Release

1. Go to Releases → Create a new release
2. Tag: `v0.1.0`
3. Title: `AIOS v0.1.0 - Initial Open-Source Release`
4. Description: Use content from CHANGELOG.md
5. Mark as "Latest release"

### 4. Enable GitHub Features

- **Issues:** Already enabled via templates
- **Discussions:** Enable for community Q&A
- **Actions:** Already configured (CLA check, Dependabot)
- **Security:** Enable security advisories

### 5. Set Up GitHub Pages (Optional)

For documentation website:
1. Go to Settings → Pages
2. Source: Deploy from a branch
3. Branch: `main` / `docs` folder
4. Save

## Repository Description

**Suggested GitHub repository description:**
```
AI-Native Operating System - The OS layer for agent-first computing. AIOS = Linux + Kubernetes + LangChain + post-quantum crypto + cognitive runtime.
```

## Repository Topics

Add these topics:
- `operating-system`
- `kernel`
- `rust`
- `typescript`
- `ai`
- `artificial-intelligence`
- `agents`
- `ai-agents`
- `aios`
- `post-quantum-cryptography`
- `agent-first`
- `os-kernel`
- `microkernel`

## Social Preview

Create a social preview image (1280x640px) with:
- AIOS logo
- Tagline: "AI-Native Operating System"
- CUI Labs branding

## Next Steps After Push

1. **Share on Social Media**
   - Twitter/X
   - LinkedIn
   - Hacker News
   - Reddit (r/rust, r/programming, r/artificial)

2. **Submit to Directories**
   - Awesome lists
   - GitHub trending
   - Tech news sites

3. **Community Building**
   - Create Discord/Slack community
   - Set up mailing list
   - Start blog posts

4. **Documentation**
   - Ensure all docs are clear
   - Add getting started guide
   - Create video tutorials

---

**Repository URL:** https://github.com/cuilabs/aios

