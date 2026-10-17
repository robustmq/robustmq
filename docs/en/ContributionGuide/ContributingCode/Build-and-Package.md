# Build and Package

## Prerequisites

| Tool | Purpose | Required |
|------|---------|----------|
| `make` | Run the repository build and release targets | Required |
| Rust (`cargo`, `rustup`) | Compile Rust code | Required |
| `pnpm` + `git` | Build frontend (Web console) | Required for full builds |
| `docker` | Build Docker images | Optional |
| `GITHUB_TOKEN` | Publish to GitHub Releases | Required for releases |

---

## Common Commands

| Command | Description |
|---------|-------------|
| `make build` | Build a binary package for the current platform (no frontend) |
| `make build-full` | Build a package including the frontend Web console |
| `make release` | Build and publish a release package (requires release credentials) |

The version number is read automatically from `Cargo.toml` — no need to specify it manually.

## Build and Connect the Web Console

The RobustMQ Web Console lives in the separate
[`robustmq-copilot`](https://github.com/robustmq/robustmq-copilot) repository. The broker exposes
the Admin HTTP API; a full binary package includes the compiled frontend under `dist/` and serves
the page and API from the same Admin HTTP port.

A full package build requires Rust (`cargo` and `rustup`), `make`, `git`, Node.js 20 or newer, and
`pnpm` 10 or newer.

### Build a package with the frontend

From the RobustMQ repository root, run:

```bash
make build-full
```

The target clones or updates `robustmq-copilot` in the ignored `build/robustmq-copilot/` directory,
runs `pnpm install --no-frozen-lockfile` and `pnpm ui:build`, compiles the Rust server, and copies
`packages/web-ui/dist/` into the package's `dist/` directory. The resulting tarball is
`build/robustmq-{version}-{platform}.tar.gz`.

Extract and start the full package (the filename below is an example for macOS ARM64):

```bash
tar -xzf build/robustmq-0.4.11-darwin-arm64.tar.gz
cd robustmq-0.4.11-darwin-arm64
./bin/robust-server start
```

At startup, the wrapper reads `http_port` from `config/server.toml` and writes `dist/config.js`.
The console uses the current page hostname and that port for Admin HTTP requests. The default page
URL is `http://127.0.0.1:58080/`.

Verify the page and backend:

```bash
curl -i http://127.0.0.1:58080/health/ready
curl -i http://127.0.0.1:58080/
```

`/health/ready` should return `HTTP 200`, and the root path should return the frontend HTML. A `404`
from `/` usually means that an API-only package without `dist/` is running, or that the process was
started from a directory other than the package root containing `dist/`. The Admin API may still be
healthy; use `make build-full` and start the broker from the full package root when the Web Console
is required.

### Develop the frontend and backend separately

Start the backend from the RobustMQ repository:

```bash
cargo run --package cmd --bin broker-server -- --conf=config/server.toml
```

In a second terminal, clone and start the frontend:

```bash
git clone https://github.com/robustmq/robustmq-copilot.git
cd robustmq-copilot
pnpm install
ROBUSTMQ_API_PORT=58080 pnpm ui:dev
```

The development page runs at `http://127.0.0.1:4000/` by default. To connect to another backend,
set the complete URL:

```bash
ROBUSTMQ_API_URL=http://192.168.1.10:58080 pnpm ui:dev
```

`ROBUSTMQ_API_URL` takes precedence over `ROBUSTMQ_API_PORT`. Without a complete URL, the frontend
uses the current page hostname and configured port. Admin HTTP enables CORS, so no additional
development proxy is needed.

### Build an API-only backend package

```bash
make build
```

This package does not include `dist/` and is suitable for API or protocol testing. Its root path may
return `404`; that does not mean the Admin HTTP API is unavailable. Request `/health/ready` to verify
the service. Use `make build-full` when the Web Console is required.

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
docker build -f docker/robustmq/Dockerfile --build-arg VERSION=v0.4.11 \
  -t robustmq:local .
```

---

## Publishing to GitHub Releases

```bash
export GITHUB_TOKEN="your_token"
make release
```

The token needs `write:packages` and `contents:write` permissions. Once published, packages are available on the [Releases page](https://github.com/robustmq/robustmq/releases).

> Note: `make release` only builds for the current system platform — cross-compilation is not supported. Multi-platform releases are handled by GitHub Actions CI running in parallel on native runners.
