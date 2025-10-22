# RobustMQ Build and Package Guide

This guide covers how to build and package RobustMQ.

## 📦 Build Artifacts Overview

RobustMQ build process generates the following types of artifacts:

| Artifact Type | File Format | Build Command | Purpose |
|---------------|-------------|---------------|---------|
| **Installation Package** | `.tar.gz` archive | `make build` / `make build-full` | Binary package for user download and installation |
| **Docker Image** | Docker image | `make docker-app-*` | Containerized deployment |
| **Dependency Image** | Docker image | `make docker-deps` | CI/CD build acceleration |
| **GitHub Release** | Online release page | `make release` | User download and view releases |

### Artifact Details

- **`.tar.gz` Installation Package**: Contains Rust-compiled binaries, configuration files, startup scripts, etc. Users can extract and run directly
- **Docker Image**: Containerized RobustMQ application, supports Docker and Kubernetes deployment
- **Dependency Image**: Pre-compiled Rust dependency cache, used to accelerate CI/CD build process
- **GitHub Release**: Online release page, users can download installation packages through browser

## 🚀 Quick Start

### Using Make Commands (Recommended)

| Command | Function | Version Source | Description |
|---------|----------|----------------|-------------|
| `make build` | Basic build | Auto-read from Cargo.toml | Build current platform package (without frontend) |
| `make build-full` | Full build | Auto-read from Cargo.toml | Build complete package with frontend |
| `make build-version VERSION=v0.1.30` | Specific version build | Manual specification | Build package with specific version |
| `make build-clean` | Clean rebuild | Auto-read from Cargo.toml | Clean and rebuild |

> **Version Note**: When version is not specified, all build commands automatically read the current version number from the `Cargo.toml` file in the project root directory.

## 🐳 Docker Image Build

### Dependency Image (CI/CD Optimization)

| Command | Function | Version Source | Description |
|---------|----------|----------------|-------------|
| `make docker-deps` | Build dependency image | Auto-read from Cargo.toml | Build CI/CD dependency cache image |
| `make docker-deps-tag TAG=2025-10-20` | Build tagged dependency image | Manual tag specification | Build dependency image with specific tag |

### Application Image

| Command | Function | Version Source | Description |
|---------|----------|----------------|-------------|
| `make docker-app ARGS='--org yourorg --version 0.2.0 --registry ghcr'` | Flexible app image build | Manual specification | Application image build with custom parameters |
| `make docker-app-ghcr ORG=yourorg VERSION=0.2.0` | GHCR app image | Manual specification | Build and push to GitHub Container Registry |
| `make docker-app-dockerhub ORG=yourorg VERSION=0.2.0` | Docker Hub app image | Manual specification | Build and push to Docker Hub |

## 🚀 Version Release

| Command | Function | Version Source | Description |
|---------|----------|----------------|-------------|
| `make release` | Create new release | Auto-read from Cargo.toml | Create GitHub release and upload package |
| `make release-version VERSION=v0.1.30` | Specific version release | Manual specification | Create GitHub release with specific version |
| `make release-upload VERSION=v0.1.30` | Upload to existing release | Manual specification | Upload package to existing GitHub release |

### Prerequisites

```bash
# Set GitHub Token (required)
export GITHUB_TOKEN="your_github_token_here"
```

> **Permission Note**:
> - Dependency images are pushed to fixed organization: `ghcr.io/robustmq/robustmq/rust-deps`
> - Application images are pushed to specified organization or user account
> - Ensure your `GITHUB_TOKEN` has `write:packages` permission
> - Ensure you have write access to the `robustmq` organization

## 📦 Output Results

### Build Artifacts

| Artifact Type | File Location | Content Description | Purpose |
|---------------|---------------|---------------------|---------|
| **Installation Package** | `build/robustmq-{version}-{platform}.tar.gz` | Compressed binary installation package | User download and install RobustMQ |
| **Package Info** | `build/robustmq-{version}-{platform}/package-info.txt` | Version, platform, build time metadata | Understand package details |
| **Docker Image** | `robustmq/robustmq:{version}` | Containerized RobustMQ application | Docker deployment and running |
| **Dependency Image** | `ghcr.io/robustmq/robustmq/rust-deps:latest` | Rust dependency cache image | Accelerate CI/CD builds, stored under robustmq organization |
| **GitHub Release** | `https://github.com/robustmq/robustmq/releases/tag/{version}` | Online release page | User download and view release notes |

### Installation Package Structure

| Directory/File | Content Type | Specific Content | Purpose |
|----------------|--------------|-----------------|---------|
| `bin/` | Startup scripts | `robust-server`, `robust-ctl`, `robust-bench` | System startup and management scripts |
| `libs/` | Binary executables | `broker-server`, `cli-command`, `cli-bench` | Core Rust-compiled binary programs |
| `config/` | Configuration files | `server.toml`, `server-tracing.toml` | Service configuration and logging configuration |
| `dist/` | Frontend static files | HTML, CSS, JavaScript files | Web management interface (if frontend included) |
| `LICENSE` | License file | Apache 2.0 license text | Legal license information |
| `package-info.txt` | Metadata file | Version, platform, build time, binary list | Package detailed information |

## 📋 Use Cases

| Scenario | Command | Artifact | Description |
|----------|---------|----------|-------------|
| **Development Testing** | `make build` | Local `.tar.gz` installation package | Quick build test package for local development and testing |
| **Release Preparation** | `make build-full` | Local complete `.tar.gz` installation package | Build complete release package with frontend for official release |
| **CI/CD Optimization** | `make docker-deps` | Docker dependency cache image | Build Rust dependency cache image, push to robustmq organization, accelerate CI/CD build process |
| **Application Deployment** | `make docker-app-ghcr ORG=yourorg VERSION=0.2.0` | Docker application image | Build and push application image to GitHub Container Registry |
| **Version Release** | `make release` | GitHub release page + installation package | Create GitHub release and upload installation package for user download |
| **Multi-platform Release** | `make release-upload VERSION=v0.1.31` | Update GitHub release | Add current platform installation package to existing GitHub release |

## 🔧 Docker Build Improvements

### Permission Issue Fix

**Issue**: Previously encountered `permission_denied: create_package` errors when building dependency images.

**Solution**:
- Use fixed organization name: `ghcr.io/robustmq/robustmq/rust-deps`
- Unified image naming for easier CI/CD management
- Ensure builders have write access to the robustmq organization

### Network Issue Fix

**Issue**: Network connection problems during build process (e.g., 502 Bad Gateway).

**Solution**:
- Implemented automatic mirror switching
- Support for official Debian, Aliyun, Tsinghua, USTC, 163, Huawei Cloud, Tencent Cloud mirrors
- Automatic retry mechanism to improve build success rate

### Build Optimization

**Improvements**:
- Separated dependency and application image build logic
- Optimized `.dockerignore` files to reduce build context
- Added pre-build checks to ensure base images are available
- Automatic GitHub Container Registry login

## ⚠️ Notes

### Build Script Limitations

| Feature | Status | Description |
|---------|--------|-------------|
| Current system platform | ✅ | Only builds for current system platform |
| Auto version detection | ✅ | Automatically reads version from Cargo.toml |
| Release mode build | ✅ | Uses `cargo build --release` |
| Auto frontend clone | ✅ | Automatically clones frontend code |
| Cross-compilation | ❌ | Does not support cross-compilation |

### Release Script Limitations

| Feature | Status | Description |
|---------|--------|-------------|
| Frontend build | ✅ | Always includes frontend build |
| Current platform | ✅ | Always builds current system platform |
| Existing release upload | ❌ | `--upload-only` requires existing release |

## 🔧 Environment Requirements

| Type | Tool | Purpose | Required |
|------|------|---------|----------|
| **Basic** | Rust (`cargo`, `rustup`) | Rust compilation | ✅ Required |
| **Frontend** | `pnpm` | Frontend package management | 🔶 Optional |
| **Frontend** | `git` | Clone frontend code | 🔶 Optional |
| **Docker** | `docker` | Docker environment | 🔶 Optional |
| **Release** | `curl` | API requests | 🔶 Optional |
| **Release** | `jq` | JSON parsing | 🔶 Optional |
| **Release** | `GITHUB_TOKEN` | GitHub access token | 🔶 Optional |

## 🆘 Common Issues

### Build Failure Troubleshooting

| Issue | Check Command | Solution |
|-------|---------------|----------|
| Rust environment issue | `cargo --version` | Install Rust environment |
| Docker environment issue | `docker info` | Start Docker service |
| Network connection issue | `ping github.com` | Check network connection |

### Docker Build Failure Troubleshooting

| Issue | Check Command | Solution |
|-------|---------------|----------|
| Docker environment issue | `docker info` | Start Docker service |
| Permission issue | `echo $GITHUB_TOKEN` | Set correct GitHub Token |
| Network connection issue | `docker pull rust:1.90.0-bookworm` | Check Docker Hub connection |
| Image push failure | `docker login ghcr.io` | Manually login to GitHub Container Registry |

### Release Failure Troubleshooting

| Issue | Check Command | Solution |
|-------|---------------|----------|
| GitHub Token issue | `echo $GITHUB_TOKEN` | Set correct Token |
| Token permission issue | `curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user` | Check Token permissions |
| Network connection issue | `curl -I https://api.github.com` | Check network connection |

### View Build Artifacts

| Operation | Command | Description |
|-----------|---------|-------------|
| View build directory | `ls -la build/` | View all build artifacts |
| Extract and test | `tar -xzf build/robustmq-*.tar.gz` | Extract installation package |
| Test binary | `./robustmq-*/libs/broker-server --help` | Test executable files |
| View package info | `cat robustmq-*/package-info.txt` | View package detailed information |

### View Docker Images

| Operation | Command | Description |
|-----------|---------|-------------|
| View local images | `docker images | grep robustmq` | View locally built images |
| View dependency images | `docker images | grep rust-deps` | View dependency cache images |
| Test image | `docker run --rm ghcr.io/robustmq/robustmq/rust-deps:latest rustc --version` | Test dependency image |
| View image history | `docker history ghcr.io/robustmq/robustmq/rust-deps:latest` | View image build history |