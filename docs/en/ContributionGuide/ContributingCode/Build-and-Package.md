# Build and Package

## Prerequisites

| Tool | Purpose | Required |
|------|---------|----------|
| Rust (`cargo`, `rustup`) | Compile Rust code | Required |
| `pnpm` + `git` | Build frontend (Web console) | Required for full builds |
| `docker` | Build Docker images | Optional |
| `GITHUB_TOKEN` | Publish to GitHub Releases | Required for releases |

---

## Common Commands

| Command | Description |
|---------|-------------|
| `make build` | Build a binary package for the current platform (no frontend) |
| `make build-full` | Build a complete package including the frontend Web console |
| `make release` | Build a complete package and publish to GitHub Releases |

The version number is read automatically from `Cargo.toml` — no need to specify it manually.

---

## Package Structure

Build artifacts are placed in the `build/` directory as `robustmq-{version}-{platform}.tar.gz`. The extracted layout:

```
robustmq-v0.3.0-linux-amd64/
  bin/
    robust-server     ← user-facing commands (added to PATH)
    robust-ctl
    robust-bench
  libs/
    broker-server     ← actual binaries (called by bin/ scripts)
    cli-command
    cli-bench
  config/
    server.toml
  dist/               ← frontend static files (build-full only)
```

---

## Docker Images

Images are built and pushed automatically by CI on every release, supporting both `linux/amd64` and `linux/arm64`:

```bash
docker pull ghcr.io/robustmq/robustmq:latest
docker pull ghcr.io/robustmq/robustmq:v0.3.0
```

To build locally:

```bash
docker build -f docker/robustmq/Dockerfile -t robustmq:local .
```

---

## Publishing to GitHub Releases

```bash
export GITHUB_TOKEN="your_token"
make release
```

The token needs `write:packages` and `contents:write` permissions. Once published, packages are available on the [Releases page](https://github.com/robustmq/robustmq/releases).

> Note: `make release` only builds for the current system platform — cross-compilation is not supported. Multi-platform releases are handled by GitHub Actions CI running in parallel on native runners.
