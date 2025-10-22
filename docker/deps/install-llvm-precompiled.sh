#!/bin/bash
# Install LLVM/Clang using precompiled binaries

set -e

echo "=== Installing LLVM/Clang using precompiled binaries ==="

# Function to try downloading with different mirrors
try_download() {
    local url="$1"
    local output="$2"
    local max_retries=3
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        if wget -q --timeout=30 --tries=3 "$url" -O "$output"; then
            return 0
        else
            echo "Download failed, retrying... ($((retry_count + 1))/$max_retries)"
            retry_count=$((retry_count + 1))
            sleep 5
        fi
    done
    
    return 1
}

# Install wget if not available
apt-get update && apt-get install -y wget

# Download and install LLVM 14 (matching Debian bookworm version)
LLVM_VERSION="14.0.6"
LLVM_ARCH="x86_64-linux-gnu"

echo "Downloading LLVM $LLVM_VERSION..."

# Try different download sources
if try_download "https://github.com/llvm/llvm-project/releases/download/llvmorg-$LLVM_VERSION/clang+llvm-$LLVM_VERSION-$LLVM_ARCH.tar.xz" "/tmp/llvm.tar.xz"; then
    echo "✅ Downloaded from GitHub releases"
elif try_download "https://releases.llvm.org/$LLVM_VERSION/clang+llvm-$LLVM_VERSION-$LLVM_ARCH.tar.xz" "/tmp/llvm.tar.xz"; then
    echo "✅ Downloaded from LLVM releases"
else
    echo "❌ Failed to download LLVM"
    exit 1
fi

# Extract and install
echo "Extracting LLVM..."
cd /tmp
tar -xf llvm.tar.xz
mv clang+llvm-$LLVM_VERSION-$LLVM_ARCH /opt/llvm

# Create symlinks
ln -sf /opt/llvm/bin/clang /usr/local/bin/clang
ln -sf /opt/llvm/bin/clang++ /usr/local/bin/clang++
ln -sf /opt/llvm/bin/lld /usr/local/bin/lld
ln -sf /opt/llvm/bin/llvm-config /usr/local/bin/llvm-config

# Set environment variables
echo 'export LLVM_CONFIG=/opt/llvm/bin/llvm-config' >> /etc/environment
echo 'export PATH=/opt/llvm/bin:$PATH' >> /etc/environment

# Verify installation
echo "Verifying installation..."
clang --version
lld --version

echo "✅ LLVM/Clang installed successfully!"
