# RobustMQ Dependency Cache Image

This directory contains all files for building the Rust dependency cache image.

## Files

- `Dockerfile.deps` - Main dependency cache image build file
- `install-deps.sh` - System dependency installation script (with mirror fallback)
- `install-runtime.sh` - Runtime dependency installation script (with mirror fallback)

## Purpose

This image is used to accelerate CI/CD build processes, containing all pre-compiled Rust dependencies.

## Build Command

```bash
make docker-deps
```

## Image Information

- **Image Name**: `ghcr.io/socutes/robustmq/rust-deps:latest`
- **Size**: ~8-10GB
- **Build Time**: 20-40 minutes (first time)
- **Contains**: 864 pre-compiled dependencies
