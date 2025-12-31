# Flatpak Publishing Guide

## Process Overview

This document details the complete process to publish an application to Flathub, including lessons learned and common pitfalls.

## Prerequisites

1. **GitHub Repository** with proper naming (no hyphens for `io.github.` IDs)
2. **Flatpak manifest** (`.yml` or `.json`)
3. **AppStream metadata** (`.metainfo.xml`)
4. **Desktop file** (`.desktop`)
5. **Icon** (SVG preferred, 128x128 minimum)
6. **Screenshot** (PNG, accessible via public URL)

## Step-by-Step Process

### 1. Prepare App ID

**Format:** `io.github.{username}.{AppName}`

**Critical Rules:**
- Use `io.github.` prefix (NOT `com.github.`)
- App name must match repository name exactly
- Repository name should NOT contain hyphens
- Example: `io.github.andresgarcia0313.ThermalMonitor` → repo: `thermalmonitor`

### 2. Create Required Files

```
flatpak/
├── io.github.username.AppName.yml      # Manifest
├── io.github.username.AppName.desktop  # Desktop entry
├── io.github.username.AppName.metainfo.xml  # Metadata
├── io.github.username.AppName.svg      # Icon
└── cargo-sources.json                   # For Rust apps
```

### 3. Generate cargo-sources.json (Rust Apps)

```bash
# Install generator
pip install aiohttp toml

# Generate from Cargo.lock
python flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
```

**Critical Path Configuration:**
- `dest` paths in cargo-sources.json must be relative to build root
- If project is in subdirectory, paths should NOT include subdirectory prefix
- Example: `"dest": "cargo/vendor/crate-name"` (NOT `"subdir/cargo/vendor/..."`)

### 4. Configure Manifest

```yaml
build-options:
  env:
    # CARGO_HOME must match cargo-sources.json dest base path
    CARGO_HOME: /run/build/module-name/cargo
```

### 5. Add Screenshot

**Requirements:**
- PNG format
- Accessible via public URL
- Minimum 624x351 pixels recommended

**Location:** `screenshots/main.png` in repository

**URL Format:**
```
https://raw.githubusercontent.com/user/repo/main/path/to/screenshot.png
```

### 6. Fork and Submit to Flathub

```bash
# Clone from new-pr branch (CRITICAL!)
gh repo fork flathub/flathub --clone
cd flathub
git checkout new-pr

# Create submission branch
git checkout -b add-io.github.username.AppName

# Add files at ROOT level (not in subdirectory!)
cp manifest.yml .
cp cargo-sources.json .

# Limit to x86_64 if aarch64 fails
echo '{"only-arches": ["x86_64"]}' > flathub.json

# Commit and push
git add -A
git commit -m "Add io.github.username.AppName"
git push origin add-io.github.username.AppName

# Create PR against new-pr branch
gh pr create --base new-pr --title "Add io.github.username.AppName" --body "..."
```

### 7. Trigger Builds

Comment on PR: `bot, build`

## Lessons Learned

### Error: `appid-uses-code-hosting-domain`
**Cause:** Using `com.github.` instead of `io.github.`
**Fix:** Change app ID prefix to `io.github.`

### Error: `appid-url-not-reachable`
**Cause:** Repository name doesn't match app ID (often due to hyphens)
**Fix:** Rename repository to match app ID without hyphens

### Error: `failed to read cargo/vendor`
**Cause:** Mismatch between CARGO_HOME and cargo-sources.json dest paths
**Fix:** Ensure paths align:
- CARGO_HOME: `/run/build/module/cargo`
- cargo-sources dest: `cargo/vendor/crate-name`

### Error: `appstream-missing-screenshots`
**Cause:** Screenshot URL returns 404
**Fix:** Add screenshot to repository and verify URL is accessible

### Error: PR auto-closed
**Cause:** PR created against `master` instead of `new-pr`
**Fix:** Always use `--base new-pr` when creating PR

### Error: Files not found during build
**Cause:** Files in subdirectory instead of repository root
**Fix:** Place manifest and cargo-sources.json at repository root

## Validation Commands

```bash
# Install linter
flatpak install -y flathub org.flatpak.Builder

# Validate manifest
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest file.yml

# Test local build
flatpak-builder --user --install --force-clean build-dir manifest.yml
```

## Post-Approval

After PR is merged:
1. Repository created at `github.com/flathub/io.github.username.AppName`
2. App appears on Flathub within hours
3. Install: `flatpak install flathub io.github.username.AppName`

## Updating the App

1. Create new tag in source repository
2. Update manifest in flathub repository with new tag/commit
3. Regenerate cargo-sources.json if dependencies changed
4. Create PR, wait for build, merge
