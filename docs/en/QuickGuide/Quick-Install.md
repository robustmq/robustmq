# Quick Install Guide

This guide helps you quickly install and start RobustMQ.

## Table of Contents

- [Installation Methods](#installation-methods)
- [Verify Installation](#verify-installation)
- [Troubleshooting](#troubleshooting)

---

## Installation Methods

### Method 1: Automated Install Script (Recommended)

```bash
# Install the latest version (installs to ~/robustmq-{version}, e.g. ~/robustmq-v0.3.0)
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

The script extracts the full package into a versioned directory (e.g. `~/robustmq-v0.3.0`) and adds its `bin/` subdirectory to your PATH. After installation, three commands are globally available:

| Command | Description |
|---------|-------------|
| `robust-server` | Start and manage the RobustMQ service |
| `robust-ctl` | CLI management tool (cluster status, Topic management, etc.) |
| `robust-bench` | Benchmarking tool |

#### Common Install Options

```bash
# Install a specific version
VERSION=v0.3.0 curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Install to a custom directory (the full package goes there; bin/ is added to PATH)
INSTALL_DIR=/opt/robustmq curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# View all install options
wget https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh && ./install.sh --help
```

Installed directory layout:
```
~/robustmq-v0.3.0/    ← versioned install directory
  bin/
    robust-server
    robust-ctl
    robust-bench
  libs/
    broker-server     ← actual binaries (called by wrapper scripts, not in PATH directly)
    cli-command
    cli-bench
  config/
    server.toml       ← configuration file
  logs/               ← runtime logs (created automatically)

~/robustmq  ->  ~/robustmq-v0.3.0   ← stable symlink
```

`~/robustmq/bin` (the symlink path) is added to PATH — not the versioned path. Upgrading only updates the symlink; no PATH changes needed.

**Script options:**

| Option | Description |
|--------|-------------|
| `--version VERSION` | Install specific version (default: latest) |
| `--dir DIRECTORY` | Installation directory (default: auto-detect) |
| `--silent` | Silent installation |
| `--force` | Force reinstall even if already installed |
| `--dry-run` | Preview what would be installed without installing |

---

### Method 2: Manual Binary Download

Visit [GitHub Releases](https://github.com/robustmq/robustmq/releases) and download the package for your platform:

```bash
# Linux x86_64 example — replace with your actual version and platform
wget https://github.com/robustmq/robustmq/releases/latest/download/robustmq-v0.3.0-linux-amd64.tar.gz

tar -xzf robustmq-v0.3.0-linux-amd64.tar.gz
cd robustmq-v0.3.0-linux-amd64

# Start the server
./bin/robust-server start
```

**Supported platforms:**

| Platform | Description |
|----------|-------------|
| `linux-amd64` | Linux x86_64 |
| `linux-arm64` | Linux ARM64 |
| `darwin-arm64` | macOS Apple Silicon (M1/M2/M3) |

---

### Method 3: Docker

```bash
# Pull the latest image
docker pull ghcr.io/robustmq/robustmq:latest

# Start a container
docker run -d \
  -p 1883:1883 \
  -p 8080:8080 \
  --name robustmq \
  ghcr.io/robustmq/robustmq:latest

# Use a specific version
docker pull ghcr.io/robustmq/robustmq:v0.3.0
```

Images support both `linux/amd64` and `linux/arm64` architectures and are hosted on [GitHub Container Registry](https://github.com/robustmq/robustmq/pkgs/container/robustmq).

---

### Method 4: Build from Source

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# Build and run
cargo run --package cmd --bin broker-server -- --conf config/server.toml
```

---

## Verify Installation

### Start the Server

```bash
# Foreground
robust-server start

# Background
robust-server start &

# With a custom config file
robust-server start /path/to/server.toml
```

On successful startup you should see:

```
[INFO] RobustMQ Broker starting...
[INFO] MQTT server listening on 0.0.0.0:1883
[INFO] Admin server listening on 0.0.0.0:8080
[INFO] Broker started successfully
```

### Check Cluster Status

```bash
robust-ctl status
```

### Test MQTT Connectivity

```bash
# Publish a message
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# Subscribe to messages
mqttx sub -h localhost -p 1883 -t "test/topic"
```

### Web Management Console

Open `http://localhost:8080` in your browser.

---

## Troubleshooting

**Q: The install script fails — what should I do?**

Check your network connection, or download the binary package manually (Method 2).

**Q: Insufficient permissions?**

```bash
# Install to user directory, no sudo needed
INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

**Q: How do I uninstall?**

```bash
# Remove the symlink and the versioned directory
rm ~/robustmq
rm -rf ~/robustmq-v0.3.0

# Also remove the ~/robustmq/bin line from ~/.bashrc or ~/.zshrc
```

**Q: How do I upgrade to a new version?**

Just run the install script again — it installs the new version and updates the `~/robustmq` symlink automatically:

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

**Q: Port already in use?**

```bash
# Check port 1883 (MQTT)
netstat -tlnp | grep 1883

# Check port 8080 (Admin)
netstat -tlnp | grep 8080
```

**Q: How do I change the configuration?**

```bash
# Edit the config file
vim config/server.toml

# Start with a custom config
robust-server start /path/to/server.toml
```

---

## Next Steps

- [Experience MQTT](Experience-MQTT.md)
- [What is RobustMQ](../OverView/What-is-RobustMQ.md)

## Get Help

- **Documentation**: https://robustmq.com
- **GitHub Issues**: https://github.com/robustmq/robustmq/issues
- **GitHub Discussions**: https://github.com/robustmq/robustmq/discussions
- **Discord**: https://discord.gg/sygeGRh5
