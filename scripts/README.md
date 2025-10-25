# RobustMQ Scripts Guide

This directory contains all build, release, and deployment scripts for the RobustMQ project.

## 📁 Scripts Overview

| Script | Function | Description |
|--------|----------|-------------|
| `build.sh` | Build & Package | Build installation package for current platform |
| `build-and-push-deps.sh` | Dependency Image | Build CI/CD dependency cache image |
| `build-and-push-app.sh` | Application Image | Build and push application  Docker image |
| `release.sh` | Release | Create GitHub release and upload packages |
| `install.sh` | Install | Auto-download and install RobustMQ |

---

## 🚀 Quick Start

### Basic Build
```bash
# Build current platform package
./scripts/build.sh

# Build with frontend
./scripts/build.sh --with-frontend
```

### Using Make (Recommended)
```bash
# Basic build
make build

# Full build with frontend
make build-full

# Build with specific version
make build-version VERSION=v0.1.30

# Clean rebuild
make build-clean

# Build dependency image
make docker-deps

# Build dependency image with tag
make docker-deps-tag TAG=2025-10-20

# Build application image (flexible)
make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'

# Build application image for GHCR
make docker-app-ghcr ORG=yourorg VERSION=0.2.0

# Build application image for Docker Hub
make docker-app-dockerhub ORG=yourorg VERSION=0.2.0

# Create new release
make release

# Create release with specific version
make release-version VERSION=v0.1.30

# Upload to existing release
make release-upload VERSION=v0.1.30

# Install RobustMQ
make install
```

---

## 📦 Build Script (build.sh)

### Usage
```bash
./scripts/build.sh [OPTIONS]

Options:
  -h, --help              Show help
  -v, --version VERSION   Specify version (default: auto-detect from Cargo.toml)
  --with-frontend         Include frontend build
  --clean                 Clean build directory
```

### Examples
```bash
# Basic build
./scripts/build.sh

# Build with frontend
./scripts/build.sh --with-frontend

# Clean rebuild
./scripts/build.sh --clean

# Specify version
./scripts/build.sh --version v0.1.30
```

### Output
Creates `build/robustmq-{version}-{platform}.tar.gz` package.

### Prerequisites
- Rust environment (`cargo`, `rustup`)
- For frontend: `pnpm`, `git`

---

## 🐳 Docker Images

### Dependency Image (build-and-push-deps.sh)
```bash
# Login to GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin

# Build and push
./scripts/build-and-push-deps.sh
./scripts/build-and-push-deps.sh 2025-10-20  # with specific tag

# Using Make
make docker-deps
```

### Application Image (build-and-push-app.sh)
```bash
# Using Make (Recommended)
make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'
make docker-app ARGS='--org yourorg --version 0.2.0 --registry dockerhub'

# Direct script usage
./scripts/build-and-push-app.sh --org socutes --version 0.2.0 --registry ghcr --push-latest
```

---

## 🚀 Release Script (release.sh)

### Usage
```bash
./scripts/release.sh [OPTIONS]

Options:
  -h, --help              Show help
  -v, --version VERSION   Specify version (default: from Cargo.toml)
  -t, --token TOKEN       GitHub Token
  --upload-only           Upload to existing release only
```

### Examples
```bash
# Create new release
./scripts/release.sh

# Upload to existing release
./scripts/release.sh --upload-only

# Specify version
./scripts/release.sh --version v0.1.30
```

### Prerequisites
```bash
export GITHUB_TOKEN="your_github_token_here"
```

Required tools: `curl`, `jq`

---

## 📋 Common Use Cases

### Development
```bash
# Quick build for testing
make build
```

### Release Preparation
```bash
# Build complete package with frontend
make build-full
```

### CI/CD Optimization
```bash
# Build dependency cache image
make docker-deps

# Build application image for GHCR
make docker-app-ghcr ORG=yourorg VERSION=0.2.0

# Build application image for Docker Hub
make docker-app-dockerhub ORG=yourorg VERSION=0.2.0
```

### Version Release
```bash
# Create new release
make release

# Create release with specific version
make release-version VERSION=v0.1.30

# Add platform package to existing release
make release-upload VERSION=v0.1.31
```

### Installation
```bash
# Auto-install RobustMQ
make install
```

---

## ⚠️ Notes

### Build Script
- ✅ Builds current platform only
- ✅ Auto-detects version from Cargo.toml
- ✅ Uses `cargo build --release`
- ✅ Auto-clones frontend code from GitHub
- ❌ No cross-compilation support

### Release Script
- ✅ Auto-detects version from Cargo.toml
- ✅ Always builds current platform
- ✅ Always includes frontend
- ❌ `--upload-only` requires existing release
- ❌ No multi-platform build support

### Error Handling
```bash
# Version doesn't exist
❌ Release v0.1.99 does not exist
# Solution: Create release first or remove --upload-only

# Missing token
❌ GitHub token is required
# Solution: export GITHUB_TOKEN="your_token"
```

---

## 🔗 Related Documentation

- [Main README](../README.md) - Project overview
- [Build Guide](https://robustmq.com/en/QuickGuide/Build-and-Package.html) - Detailed build instructions
- [Contributing Guide](https://robustmq.com/ContributionGuide/GitHub-Contribution-Guide.html) - How to contribute