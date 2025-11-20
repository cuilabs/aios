# Changesets

This repository uses [Changesets](https://github.com/changesets/changesets) for version management and changelog generation.

## How to Use

### Creating a Changeset

When you make changes that should be released, create a changeset:

```bash
pnpm changeset
```

This will:
1. Ask you what kind of change this is (patch, minor, major)
2. Ask which packages should be bumped
3. Create a changeset file in `.changeset/`

### Releasing

When you're ready to release:

1. **Version packages** (updates version numbers and CHANGELOG):
   ```bash
   pnpm version
   ```

2. **Publish to npm** (if applicable):
   ```bash
   pnpm release
   ```

### Automatic Releases

The GitHub Actions workflow (`.github/workflows/changeset.yml`) automatically:
- Creates a PR with version bumps when changesets are merged
- Publishes to npm when the version PR is merged

## Changeset Files

Changeset files are located in `.changeset/` and follow this format:

```
---
"package-name": patch
---

Description of the change
```

These files are automatically processed during the release process.

