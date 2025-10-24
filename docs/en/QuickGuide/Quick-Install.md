# Quick Install Guide

This guide will help you quickly install and start RobustMQ, including multiple installation methods and detailed verification steps.

## Table of Contents

- [Installation Methods](#installation-methods)
- [Verify Installation](#verify-installation)
- [Troubleshooting](#troubleshooting)

## Installation Methods

### Method 1: Automated Install Script (Recommended)

#### One-Click Install Latest Version

```bash
# Install latest version automatically
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Start the server
broker-server start
```

#### Install Specific Version

```bash
# Install specific version
VERSION=v0.1.35 curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Install to custom directory
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

#### Installation Options

```bash
# Download script first to review all options
wget https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh
./install.sh --help
```

**Available Options:**
- `--version VERSION`: Install specific version (default: latest)
- `--dir DIRECTORY`: Installation directory (default: auto-detect)
- `--silent`: Silent installation
- `--force`: Force installation even if already exists
- `--dry-run`: Show what would be installed without actually installing

### Method 2: Pre-built Binaries

#### Manual Download

Visit the [releases page](https://github.com/robustmq/robustmq/releases) and download the appropriate package for your platform:

```bash
# Linux x86_64 example (replace with your platform)
wget https://github.com/robustmq/robustmq/releases/latest/download/robustmq-v0.1.35-linux-amd64.tar.gz

# Extract the package
tar -xzf robustmq-v0.1.35-linux-amd64.tar.gz
cd robustmq-v0.1.35-linux-amd64

# Run the server
./bin/robust-server start
```

**Supported platforms:** `linux-amd64`, `linux-arm64`, `darwin-amd64`, `darwin-arm64`, `windows-amd64`

### Method 3: Build from Source

```bash
# Clone the repository
git clone https://github.com/robustmq/robustmq.git
cd robustmq

# Build and run
cargo run --package cmd --bin broker-server
```

### Method 4: Docker (Coming Soon)

```bash
# Docker run (coming soon)
docker run -p 1883:1883 -p 9092:9092 robustmq/robustmq:latest
```

## Verify Installation

### Check Binaries

```bash
# Check if installation was successful
broker-server --version
cli-command --help
cli-bench --help
```

### Start Server

```bash
# Start the server
broker-server start

# Start in background
nohup broker-server start > broker.log 2>&1 &

# Start with config file
broker-server start config/server.toml
```

### Verify Server Status

After starting successfully, you should see output similar to:

```
[INFO] RobustMQ Broker starting...
[INFO] MQTT server listening on 0.0.0.0:1883
[INFO] Admin server listening on 0.0.0.0:8080
[INFO] Broker started successfully
```

### Check Cluster Status

```bash
# Check cluster running status
cli-command status
```

Expected output:
```
🚀 Checking RobustMQ status...
✅ RobustMQ Status: Online
📋 Version: RobustMQ 0.1.35
🌐 Server: 127.0.0.1:8080
```

### Connection Testing

#### Test with MQTT Client

```bash
# Test connection with MQTTX
mqttx pub -h localhost -p 1883 -t "test/topic" -m "Hello RobustMQ!"

# Subscribe to messages
mqttx sub -h localhost -p 1883 -t "test/topic"
```

#### Use Web Console

Visit `http://localhost:8080` to access the Web management interface.

## Troubleshooting

### Installation Issues

**Q: What if the install script fails?**
A: Check your network connection, or manually download pre-built packages.

**Q: What if I don't have sufficient permissions?**
A: Use `sudo` or install to user directory:
```bash
INSTALL_DIR=$HOME/.local/bin curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

**Q: How to uninstall?**
A: Remove the installed binaries:
```bash
rm -f /usr/local/bin/broker-server
rm -f /usr/local/bin/cli-command
rm -f /usr/local/bin/cli-bench
```

### Startup Issues

**Q: What if ports are already in use?**
A: Check port usage:
```bash
# Check port 1883
netstat -tlnp | grep 1883

# Check port 8080
netstat -tlnp | grep 8080
```

**Q: How to modify configuration?**
A: Edit configuration file:
```bash
# Edit default config
vim config/server.toml

# Start with custom config
broker-server start /path/to/your/config.toml
```

### Connection Issues

**Q: Can't connect to MQTT server?**
A: Check firewall settings and port configuration.

**Q: Can't access Web console?**
A: Confirm the management port (default 8080) is running properly.

## Next Steps

After installation, you can:

1. **Experience MQTT features**: Check out [MQTT Experience Guide](Experience-MQTT.md)
2. **Learn configuration options**: Check out [Configuration Documentation](../Configuration/COMMON.md)
3. **Explore advanced features**: Check out [Complete Documentation](../OverView/What-is-RobustMQ.md)

## Get Help

- **📖 [Official Documentation](https://robustmq.com/)** - Complete guides and API references
- **🐛 [GitHub Issues](https://github.com/robustmq/robustmq/issues)** - Bug reports
- **💡 [GitHub Discussions](https://github.com/robustmq/robustmq/discussions)** - Discussions and suggestions
- **🎮 [Discord](https://discord.gg/sygeGRh5)** - Real-time chat and support
