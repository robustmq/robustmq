# RobustMQ Installation Guide

This guide explains how to use the `install.sh` script to install RobustMQ on your system. The script automatically downloads and installs pre-built binaries from GitHub releases.

## 📋 Overview

The `install.sh` script provides an automated way to:

- **Download** the correct binary package for your platform
- **Install** RobustMQ components to your system
- **Configure** the installation directory and PATH
- **Verify** installation integrity with checksums

## 🔧 Prerequisites

### Required Tools
- `curl` or `wget` - For downloading packages
- `tar` - For extracting archives
- `shasum` or `sha256sum` - For checksum verification (optional)

### Supported Platforms

| Platform | Architecture | Support Level | Package Name Format |
|----------|-------------|---------------|-------------------|
| Linux | x86_64 (amd64) | ✅ Full | `robustmq-{VERSION}-linux-amd64.tar.gz` |
| Linux | ARM64 | ✅ Full | `robustmq-{VERSION}-linux-arm64.tar.gz` |
| macOS | x86_64 (Intel) | ✅ Full | `robustmq-{VERSION}-darwin-amd64.tar.gz` |
| macOS | ARM64 (Apple Silicon) | ✅ Full | `robustmq-{VERSION}-darwin-arm64.tar.gz` |
| Windows | x86_64 | ✅ Full | `robustmq-{VERSION}-windows-amd64.tar.gz` |
| Linux | ARMv7 | ⚠️ Limited | `robustmq-{VERSION}-linux-armv7.tar.gz` |
| Linux | x86 (386) | ⚠️ Limited | `robustmq-{VERSION}-linux-386.tar.gz` |
| Windows | x86 (386) | ⚠️ Limited | `robustmq-{VERSION}-windows-386.tar.gz` |
| FreeBSD | x86_64 | ⚠️ Limited | `robustmq-{VERSION}-freebsd-amd64.tar.gz` |

## 🚀 Quick Start

### Method 1: One-line Installation (Latest Version)

```bash
# Install latest server component
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash

# Or using wget
wget -qO- https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
```

### Method 2: Download and Run

```bash
# Download the script
curl -fsSL -o install.sh https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
chmod +x install.sh

# Run installation
./install.sh
```

## 📖 Usage Options

### Command Line Options

| Option | Description | Default | Example |
|--------|-------------|---------|---------|
| `-h, --help` | Show help message | - | `./install.sh --help` |
| `-v, --version VERSION` | Install specific version | `latest` | `./install.sh --version v0.1.29` |
| `-c, --component COMP` | Component to install | `server` | `./install.sh --component operator` |
| `-d, --dir DIRECTORY` | Installation directory | auto-detect | `./install.sh --dir /usr/local/bin` |
| `-s, --silent` | Silent installation | `false` | `./install.sh --silent` |
| `-f, --force` | Force overwrite existing | `false` | `./install.sh --force` |
| `--dry-run` | Preview without installing | `false` | `./install.sh --dry-run` |

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `VERSION` | Version to install | `latest` | `VERSION=v0.1.29 ./install.sh` |
| `COMPONENT` | Component to install | `server` | `COMPONENT=operator ./install.sh` |
| `INSTALL_DIR` | Installation directory | auto-detect | `INSTALL_DIR=/opt/bin ./install.sh` |
| `SILENT` | Silent mode | `false` | `SILENT=true ./install.sh` |
| `FORCE` | Force installation | `false` | `FORCE=true ./install.sh` |
| `DRY_RUN` | Dry run mode | `false` | `DRY_RUN=true ./install.sh` |

## 🎯 Installation Examples

### Basic Examples

#### Install Latest Server
```bash
./install.sh
```

#### Install Specific Version
```bash
./install.sh --version v0.1.29
```

#### Install Operator Component
```bash
./install.sh --component operator
```

#### Install to Custom Directory
```bash
./install.sh --dir /usr/local/bin
```

#### Silent Installation
```bash
./install.sh --silent --version v0.1.29
```

### Advanced Examples

#### Install All Components
```bash
./install.sh --component all --version v0.1.29
```

#### Force Reinstall
```bash
./install.sh --force --version v0.1.29
```

#### Preview Installation
```bash
./install.sh --dry-run --version v0.1.29 --component all
```

#### Custom Installation with Environment Variables
```bash
VERSION=v0.1.29 \
COMPONENT=server \
INSTALL_DIR=/opt/robustmq/bin \
SILENT=true \
./install.sh
```

## 🗂️ Components

### Server Component
- **Binary**: `robustmq`
- **Description**: Main RobustMQ server (MQTT broker, Kafka compatibility, etc.)
- **Usage**: `robustmq --help`

### Operator Component  
- **Binary**: `robustmq-operator`
- **Description**: Kubernetes operator for managing RobustMQ clusters
- **Usage**: Deploy to Kubernetes cluster

### All Components
- Installs both server and operator binaries

## 📁 Installation Directories

The script automatically determines the best installation directory:

### Priority Order
1. **`/usr/local/bin`** - If writable (system-wide, requires sudo)
2. **`$HOME/.local/bin`** - User-local directory
3. **`$HOME/bin`** - User bin directory  
4. **`$HOME/.robustmq/bin`** - Fallback directory

### Custom Directory
```bash
# Install to specific directory
./install.sh --dir /opt/robustmq/bin

# Using environment variable
INSTALL_DIR=/usr/local/bin ./install.sh
```

## 🔒 Security and Verification

### Checksum Verification
The script automatically verifies package integrity:

```bash
# Downloads both files:
# - robustmq-v0.1.29-linux-amd64.tar.gz
# - robustmq-v0.1.29-linux-amd64.tar.gz.sha256

# Verifies checksum automatically
```

### Manual Verification
```bash
# Verify checksum manually
sha256sum -c robustmq-v0.1.29-linux-amd64.tar.gz.sha256
```

## 🔄 Workflow Examples

### Development Workflow
```bash
# Install latest development version
./install.sh --version latest

# Test with dry run first
./install.sh --dry-run --version v0.1.30-beta

# Install beta version
./install.sh --version v0.1.30-beta --force
```

### Production Workflow
```bash
# Install specific stable version
./install.sh --version v0.1.29 --silent

# Install to system directory (requires sudo)
sudo ./install.sh --version v0.1.29 --dir /usr/local/bin

# Verify installation
robustmq --version
```

### CI/CD Pipeline
```bash
#!/bin/bash
# CI/CD installation script

set -euo pipefail

# Install specific version for CI
VERSION=v0.1.29 \
COMPONENT=server \
SILENT=true \
FORCE=true \
./install.sh

# Verify installation
if ! command -v robustmq >/dev/null 2>&1; then
    echo "Installation failed"
    exit 1
fi

echo "RobustMQ installed successfully: $(robustmq --version)"
```

## 🐛 Troubleshooting

### Common Issues

#### 1. Permission Denied
```bash
Error: Failed to install robustmq to /usr/local/bin/robustmq
```
**Solution**: Use sudo or install to user directory
```bash
# Use sudo for system installation
sudo ./install.sh

# Or install to user directory
./install.sh --dir "$HOME/.local/bin"
```

#### 2. Platform Not Supported
```bash
Error: Platform linux-mips64 is not supported
```
**Solution**: Check supported platforms or build from source
```bash
# Check supported platforms
./install.sh --help

# See full list in this guide
```

#### 3. Download Failed
```bash
Error: Failed to download robustmq
```
**Solutions**:
```bash
# Check version exists
curl -s https://api.github.com/repos/robustmq/robustmq/releases/tags/v0.1.29

# Check internet connection
curl -I https://github.com

# Try specific version
./install.sh --version v0.1.28
```

#### 4. Binary Not Found in Archive
```bash
Error: Binary robustmq not found in the archive
```
**Solution**: This indicates a package format issue
```bash
# Check archive contents
tar -tzf robustmq-v0.1.29-linux-amd64.tar.gz

# Report issue with details
```

### Debug Mode

#### Dry Run
```bash
# Preview installation without changes
./install.sh --dry-run --version v0.1.29
```

#### Verbose Output
```bash
# Enable detailed logging (modify script to add debug)
VERBOSE=true ./install.sh --version v0.1.29
```

## 🔧 Customization

### Custom Installation Script
```bash
#!/bin/bash
# Custom installation wrapper

set -euo pipefail

# Configuration
ROBUSTMQ_VERSION="v0.1.29"
INSTALL_BASE="/opt/robustmq"

# Create directories
sudo mkdir -p "$INSTALL_BASE"/{bin,config,data,logs}

# Install RobustMQ
sudo INSTALL_DIR="$INSTALL_BASE/bin" \
     VERSION="$ROBUSTMQ_VERSION" \
     ./install.sh

# Create symlink
sudo ln -sf "$INSTALL_BASE/bin/robustmq" /usr/local/bin/robustmq

# Set up systemd service (optional)
sudo tee /etc/systemd/system/robustmq.service > /dev/null << EOF
[Unit]
Description=RobustMQ Server
After=network.target

[Service]
Type=simple
User=robustmq
WorkingDirectory=$INSTALL_BASE
ExecStart=$INSTALL_BASE/bin/robustmq
Restart=always

[Install]
WantedBy=multi-user.target
EOF

echo "RobustMQ installed and configured successfully!"
```

### Docker Integration
```bash
# Use in Dockerfile
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y curl tar

# Install RobustMQ
RUN curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | \
    VERSION=v0.1.29 INSTALL_DIR=/usr/local/bin bash

CMD ["robustmq"]
```

## 📚 Integration Examples

### Kubernetes Deployment
```yaml
# robustmq-install-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: robustmq-install
spec:
  template:
    spec:
      containers:
      - name: installer
        image: alpine:latest
        command:
          - /bin/sh
          - -c
          - |
            apk add --no-cache curl tar
            curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | \
            VERSION=v0.1.29 INSTALL_DIR=/shared/bin bash
        volumeMounts:
        - name: shared
          mountPath: /shared
      volumes:
      - name: shared
        emptyDir: {}
      restartPolicy: Never
```

### Ansible Playbook
```yaml
# install-robustmq.yml
---
- name: Install RobustMQ
  hosts: all
  tasks:
    - name: Download installation script
      get_url:
        url: https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh
        dest: /tmp/install.sh
        mode: '0755'

    - name: Install RobustMQ
      shell: |
        VERSION=v0.1.29 \
        COMPONENT=server \
        INSTALL_DIR=/usr/local/bin \
        /tmp/install.sh
      become: yes

    - name: Verify installation
      command: robustmq --version
      register: version_output

    - name: Display version
      debug:
        msg: "RobustMQ installed: {{ version_output.stdout }}"
```

## 🚀 Next Steps

After successful installation:

### For Server Component
```bash
# Check version
robustmq --version

# View help
robustmq --help

# Start server (development)
robustmq

# Start with custom config
robustmq --config /path/to/config.toml
```

### For Operator Component
```bash
# Deploy operator to Kubernetes
kubectl apply -f https://raw.githubusercontent.com/robustmq/robustmq/main/operator/robustmq.yaml

# Create RobustMQ instance
kubectl apply -f https://raw.githubusercontent.com/robustmq/robustmq/main/operator/sample-robustmq.yaml

# Check operator status
kubectl get pods -n robustmq-system
```

## 📞 Support

- **📖 Documentation**: https://robustmq.com/docs
- **🐛 Issues**: https://github.com/robustmq/robustmq/issues
- **💬 Discussions**: https://github.com/robustmq/robustmq/discussions
- **📦 Releases**: https://github.com/robustmq/robustmq/releases

## ✨ Tips and Best Practices

### Version Management
- Always specify exact versions in production
- Use `latest` only for development/testing
- Check [releases page](https://github.com/robustmq/robustmq/releases) for available versions

### Security
- Verify checksums in production environments
- Use HTTPS URLs for downloads
- Consider running installations in sandboxed environments

### Performance
- Install to local SSDs for better performance
- Ensure sufficient disk space (packages are typically 10-50MB)
- Use closest mirror if available

### Automation
- Use dry-run mode to test automation scripts
- Implement proper error handling in CI/CD
- Log installation details for troubleshooting

---

> **💡 Pro Tip**: Use `./install.sh --dry-run` to preview any installation before making changes to your system!