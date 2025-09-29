# RobustMQ Build and Package Guide

This guide covers how to compile and package RobustMQ, including local builds, Docker image builds, and release processes.

## Table of Contents

- [Environment Setup](#environment-setup)
- [Quick Start](#quick-start)
- [Build Options](#build-options)
- [Docker Build](#docker-build)
- [Output Results](#output-results)
- [Release Process](#release-process)
- [Common Issues](#common-issues)

## Environment Setup

Please refer to [Build Development Environment](../ContributionGuide/ContributingCode/Build-Develop-Env.md) to complete the development environment configuration.

### Additional Dependencies (Optional)

- **Frontend Build**: `pnpm` and `git` installed (only required when using `--with-frontend`)
- **Docker Build**: `docker` installed and running (only required when using `--with-docker`)

> **Note**: If required tools are missing, the build script will automatically detect and display detailed installation instructions, including commands for different operating systems.

## Quick Start

### 1. Clone Project

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
```

### 2. Basic Build

```bash
# Build current platform package (using version from Cargo.toml)
./scripts/build.sh

# Build complete package with frontend
./scripts/build.sh --with-frontend

# Build Docker image
./scripts/build.sh --with-docker

# Build with specific version
./scripts/build.sh --version v0.1.31
```

> **Version Note**: If `--version` parameter is not specified, the build script will automatically read the version number from the `Cargo.toml` file in the project root directory.

### 3. View Build Results

```bash
ls -la build/
# Example output:
# robustmq-0.1.35-darwin-arm64.tar.gz
```

## Build Options

### build.sh Script

`scripts/build.sh` is the main build script that supports the following options:

```bash
./scripts/build.sh [OPTIONS]

Options:
  -h, --help              Show help message
  -v, --version VERSION   Specify version (default: read from Cargo.toml in project root)
  --with-frontend         Include frontend build
  --with-docker           Build Docker image
  --clean                 Clean build directory
```

### Usage Scenarios

#### Scenario 1: Development Testing
```bash
# Quick build for current platform (automatically uses version from Cargo.toml)
./scripts/build.sh
```

#### Scenario 2: Complete Release Package
```bash
# Build complete package with frontend (automatically clones frontend code)
./scripts/build.sh --with-frontend

# Build complete release package with specific version
./scripts/build.sh --with-frontend --version v0.1.31
```

#### Scenario 3: Docker Image Build
```bash
# Build Docker image
./scripts/build.sh --with-docker

# Build Docker image with specific version
./scripts/build.sh --with-docker --version v0.1.31
```

#### Scenario 4: Clean Rebuild
```bash
# Clean and rebuild
./scripts/build.sh --clean --with-frontend
```

## Docker Build

### Build Docker Image

```bash
# Build Docker image
./scripts/build.sh --with-docker
```

### Output Results

After build completion, Docker images will be generated:
- `robustmq/robustmq:{version}` - Version tagged image
- `robustmq/robustmq:latest` - Latest tagged image

### Prerequisites

- `docker` installed and running
- Docker build based on `docker/Dockerfile` file
- Script automatically checks Docker command and daemon status

## Output Results

### Regular Build
After build completion, files will be generated in the `build/` directory:
- `robustmq-{version}-{platform}.tar.gz` - Installation package

### Package Structure

```text
robustmq-0.1.35-darwin-arm64/
├── bin/           # Source bin directory (startup scripts, etc.)
├── libs/          # Rust compiled binary files
│   ├── broker-server
│   ├── cli-command
│   └── cli-bench
├── config/        # Source config directory (configuration files)
├── dist/          # Frontend build artifacts (if frontend included)
├── LICENSE        # License file
└── package-info.txt # Package information file (includes version info)
```

## Release Process

### release.sh Script

`scripts/release.sh` is used for automated GitHub release process:

#### Basic Usage

```bash
# Release new version (create release and upload package)
./scripts/release.sh

# Only upload current system package to existing release
./scripts/release.sh --upload-only

# Specify version
./scripts/release.sh --version v0.1.31
```

#### Environment Variables

```bash
# Required: GitHub personal access token
export GITHUB_TOKEN="your_github_token"
```

#### Release Steps

1. **Set GitHub Token**
   ```bash
   export GITHUB_TOKEN="your_token_here"
   ```

2. **Execute Release**
   ```bash
   # Release new version
   ./scripts/release.sh
   
   # Or only upload package to existing version
   ./scripts/release.sh --upload-only
   ```

## Common Issues

### Q: Docker build fails

**A:** Check the following:

1. Is Docker installed and running
2. Does `docker/Dockerfile` file exist
3. Is Docker daemon running normally

```bash
# Check Docker status
docker info
```

### Q: GitHub release fails

**A:** Check the following:

1. Is GitHub Token set correctly
2. Does Token have sufficient permissions
3. Is network connection normal

### Q: Where are build artifacts

**A:** Default location is `./build/` directory:

```bash
ls -la build/
# View all build artifacts
find build/ -name "*.tar.gz"
```

### Q: How to verify build artifacts

**A:** Extract and test:

```bash
# Extract and test
tar -xzf robustmq-0.1.35-darwin-arm64.tar.gz
cd robustmq-0.1.35-darwin-arm64

# Test binary files
./libs/broker-server --help
./libs/cli-command --help
./libs/cli-bench --help

# View package information
cat package-info.txt
cat VERSION
```

## Notes

- ✅ Only builds for current system platform
- ✅ When `--version` is not specified, automatically reads version from Cargo.toml file in project root
- ✅ Uses `cargo build --release` for Rust project builds
- ✅ Frontend build is optional, automatically clones robustmq-copilot code from GitHub
- ✅ Always pulls latest code before building frontend (git pull)
- ✅ Docker build based on `docker/Dockerfile` file
- ✅ Docker build checks Docker command and daemon status
- ❌ Does not support cross-compilation