# RobustMQ Build and Package Guide

This guide provides detailed instructions on how to compile and package RobustMQ, including local builds, cross-platform compilation, and release processes.

## Table of Contents

- [Environment Setup](#environment-setup)
- [Quick Start](#quick-start)
- [Build Script Details](#build-script-details)
- [Supported Platforms](#supported-platforms)
- [Build Components](#build-components)
- [Release Process](#release-process)
- [Common Issues](#common-issues)

## Environment Setup

### Required Dependencies

#### Rust Environment (for building server components)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Go Environment (for building Kubernetes Operator)

```bash
# Install Go (version >= 1.19)
# Download and install from https://golang.org/dl/

# Verify installation
go version
```

#### Other Tools

```bash
# Required tools
sudo apt-get install curl jq git tar  # Ubuntu/Debian
# or
brew install curl jq git  # macOS
```

install `protoc`

```bash
# Ubuntu(has been verified)
PB_REL="https://github.com/protocolbuffers/protobuf/releases"
curl -LO $PB_REL/download/v26.1/protoc-26.1-linux-x86_64.zip
# Replace the local version
unzip protoc-26.1-linux-x86_64.zip -d $HOME/.local
```

resolve: `failed to run custom build command for zstd-sysv2.0.15+zstd.1.5.7`

```bash
# resolve: failed to run custom build command for zstd-sysv2.0.15+zstd.1.5.7
# Ubuntu(has been verified)
sudo apt install build-essential clang pkg-config libssl-dev
# Fedora/RHEL
sudo dnf install clang pkg-config zstd-devel # Fedora/RHEL
# macOS
brew install zstd pkg-config 
```

### Optional Dependencies

#### Cross-platform Compilation Toolchain

```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-gnu
```

## Quick Start

### 1. Clone the Project

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq
```

### 2. Local Build (Current Platform)

```bash
# Build server component (default)
./scripts/build.sh

# Build all components
./scripts/build.sh --component all

# Build specific components
./scripts/build.sh --component server
./scripts/build.sh --component operator
```

### 3. View Build Results

```bash
ls -la build/
# Example output:
# robustmq-v1.0.0-darwin-arm64.tar.gz
# robustmq-v1.0.0-darwin-arm64.tar.gz.sha256
```

## Build Script Details

### build.sh Script

`scripts/build.sh` is the main build script that supports various build options:

#### Basic Usage

```bash
./scripts/build.sh [OPTIONS]
```

#### Main Options

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --help` | Show help information | - |
| `-v, --version VERSION` | Specify build version | Auto-detect from git |
| `-c, --component COMP` | Build component: server/operator/all | server |
| `-p, --platform PLATFORM` | Target platform | Auto-detect current platform |
| `-a, --all-platforms` | Build for all supported platforms | - |
| `-t, --type TYPE` | Build type: release/debug | release |
| `-o, --output DIR` | Output directory | ./build |
| `--clean` | Clean output directory before building | false |
| `--verbose` | Enable verbose output | false |
| `--dry-run` | Show what would be built without actually building | false |
| `--no-parallel` | Disable parallel builds | false |

#### Usage Examples

```bash
# Build release version for current platform
./scripts/build.sh

# Build debug version for specific platform
./scripts/build.sh --platform linux-amd64 --type debug

# Build for all platforms
./scripts/build.sh --all-platforms

# Build with specific version
./scripts/build.sh --version v1.0.0

# Clean build
./scripts/build.sh --clean

# View what would be built (without actually building)
./scripts/build.sh --dry-run --all-platforms
```

## Supported Platforms

### Server Component (Rust)

| Platform ID | Operating System | Architecture | Rust Target |
|-------------|------------------|--------------|-------------|
| `linux-amd64` | Linux | x86_64 | x86_64-unknown-linux-gnu |
| `linux-amd64-musl` | Linux | x86_64 (musl) | x86_64-unknown-linux-musl |
| `linux-arm64` | Linux | ARM64 | aarch64-unknown-linux-gnu |
| `linux-arm64-musl` | Linux | ARM64 (musl) | aarch64-unknown-linux-musl |
| `linux-armv7` | Linux | ARMv7 | armv7-unknown-linux-gnueabihf |
| `darwin-amd64` | macOS | x86_64 | x86_64-apple-darwin |
| `darwin-arm64` | macOS | ARM64 (Apple Silicon) | aarch64-apple-darwin |
| `windows-amd64` | Windows | x86_64 | x86_64-pc-windows-gnu |
| `windows-386` | Windows | x86 | i686-pc-windows-gnu |
| `windows-arm64` | Windows | ARM64 | aarch64-pc-windows-gnullvm |
| `freebsd-amd64` | FreeBSD | x86_64 | x86_64-unknown-freebsd |

### Operator Component (Go)

| Platform ID | Operating System | Architecture | Go Target |
|-------------|------------------|--------------|-----------|
| `linux-amd64` | Linux | x86_64 | linux/amd64 |
| `linux-arm64` | Linux | ARM64 | linux/arm64 |
| `linux-armv7` | Linux | ARMv7 | linux/arm |
| `darwin-amd64` | macOS | x86_64 | darwin/amd64 |
| `darwin-arm64` | macOS | ARM64 | darwin/arm64 |
| `windows-amd64` | Windows | x86_64 | windows/amd64 |
| `windows-386` | Windows | x86 | windows/386 |
| `freebsd-amd64` | FreeBSD | x86_64 | freebsd/amd64 |

## Build Components

### Server Component

The server component includes the following binaries:

- `broker-server` - RobustMQ main server
- `cli-command` - Command-line management tool
- `cli-bench` - Performance testing tool

#### Build Process

1. Check Rust environment and target platform
2. Install necessary Rust targets (if not installed)
3. Compile binaries using `cargo build`
4. Create package structure and copy files
5. Generate tarball and checksums

#### Package Structure

```text
robustmq-v1.0.0-linux-amd64/
├── bin/           # Startup scripts
├── libs/          # Binary files
├── config/        # Configuration files
├── docs/          # Documentation
├── package-info.txt  # Package information
└── version.txt    # Version information
```

### Operator Component

The Operator component is a Kubernetes operator for managing RobustMQ in K8s environments.

#### Operator Build Process

1. Check Go environment
2. Set cross-compilation environment variables
3. Compile binary using `go build`
4. Create package structure and copy related files
5. Generate tarball and checksums

#### Operator Package Structure

```text
robustmq-operator-v1.0.0-linux-amd64/
├── bin/           # Binary files
├── config/        # Configuration files
├── manifests/     # K8s manifest files
├── docs/          # Documentation
├── package-info.txt  # Package information
└── version.txt    # Version information
```

## Release Process

### release.sh Script

`scripts/release.sh` is used to automate the GitHub release process:

#### Main Features

1. Extract version number from `Cargo.toml`
2. Check or create GitHub Release
3. Call `build.sh` to build distribution packages
4. Upload tarball files to GitHub Release

#### Usage

```bash
# Basic usage
./scripts/release.sh

# Specify version
./scripts/release.sh --version v1.0.0

# Specify platform
./scripts/release.sh --platform linux-amd64

# Build for all platforms
./scripts/release.sh --platform all

# Dry run (view what would be executed)
./scripts/release.sh --dry-run

# Force recreate existing Release
./scripts/release.sh --force
```

#### Environment Variables

```bash
# Required: GitHub personal access token
export GITHUB_TOKEN="your_github_token"

# Optional: GitHub repository (default: robustmq/robustmq)
export GITHUB_REPO="owner/repo"

# Other options
export VERSION="v1.0.0"
export PLATFORM="linux-amd64"
export DRY_RUN="true"
export FORCE="true"
export VERBOSE="true"
export SKIP_BUILD="true"
```

#### GitHub Token Permissions

When creating a GitHub personal access token, the following permissions are required:

- `repo` - Full control of private repositories
- `public_repo` - Access to public repositories

### Release Steps

1. **Prepare Environment**

   ```bash
   # Set GitHub Token
   export GITHUB_TOKEN="your_token_here"
   
   # Ensure you're in the project root directory
   cd robustmq
   ```

2. **Execute Release**

   ```bash
   # Release current version to all platforms
   ./scripts/release.sh --platform all
   
   # Or release specific platform
   ./scripts/release.sh --platform linux-amd64
   ```

3. **Verify Release**

   - Visit GitHub Releases page
   - Check uploaded files
   - Verify download links

## Common Issues

### Q: Build fails with missing Rust target

**A:** Install the corresponding Rust target:

```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
# etc...
```

### Q: Cross-compilation fails

**A:** Ensure the corresponding toolchain is installed:

```bash
# Compile Windows on Linux
sudo apt-get install gcc-mingw-w64-x86-64

# Compile Linux on macOS
brew install FiloSottile/musl-cross/musl-cross
```

### Q: GitHub release fails

**A:** Check the following items:

1. Is GitHub Token correctly set?
2. Does the token have sufficient permissions?
3. Is the network connection normal?
4. Does the repository exist and is accessible?

### Q: How to skip build and upload existing packages

**A:** Use the `--skip-build` option:

```bash
./scripts/release.sh --skip-build
```

### Q: How to view detailed build logs

**A:** Use the `--verbose` option:

```bash
./scripts/build.sh --verbose
./scripts/release.sh --verbose
```

### Q: Where are the build artifacts

**A:** By default in the `./build/` directory:

```bash
ls -la build/
# View all build artifacts
find build/ -name "*.tar.gz"
```

### Q: How to verify build artifacts

**A:** Use checksum files:

```bash
# Verify SHA256
sha256sum -c robustmq-v1.0.0-linux-amd64.tar.gz.sha256

# Extract and test
tar -xzf robustmq-v1.0.0-linux-amd64.tar.gz
cd robustmq-v1.0.0-linux-amd64
./libs/broker-server --help
```

## Advanced Usage

### Custom Build Configuration

```bash
# Configure using environment variables
export VERSION="v1.0.0"
export BUILD_TYPE="release"
export OUTPUT_DIR="/custom/build/path"
export VERBOSE="true"

./scripts/build.sh
```

### Parallel Build for Multiple Platforms

```bash
# Build multiple specific platforms (parallel)
./scripts/build.sh --platform linux-amd64 &
./scripts/build.sh --platform darwin-arm64 &
./scripts/build.sh --platform windows-amd64 &
wait
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Build RobustMQ
  run: |
    export GITHUB_TOKEN=${{ secrets.GITHUB_TOKEN }}
    ./scripts/release.sh --platform all
```

## Summary

RobustMQ provides a complete build and release toolchain:

- **build.sh**: Flexible build script supporting multi-platform, multi-component builds
- **release.sh**: Automated release script integrated with GitHub Releases
- **Cross-platform Support**: Supports mainstream operating systems and architectures
- **Component-based Build**: Can build server or Operator components separately
- **Complete Package Management**: Automatically generates package information, checksums, etc.

Through this guide, you can easily build and release RobustMQ to various platforms, meeting the needs of different deployment environments.
