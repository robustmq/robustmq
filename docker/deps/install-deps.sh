#!/bin/bash
# Install system dependencies with mirror fallback

set -e

# Function to try installing packages with a specific mirror
try_install() {
    local mirror_name="$1"
    local mirror_url="$2"
    local packages="$3"
    
    echo "Trying to install with $mirror_name mirror..."
    
    # Update sources to use the mirror
    sed -i "s/deb.debian.org/$mirror_url/g" /etc/apt/sources.list.d/debian.sources
    
    # Update package list
    apt-get update
    
    # Install packages
    apt-get install -y $packages
    
    echo "Successfully installed with $mirror_name mirror"
}

# Define packages to install
PACKAGES="protobuf-compiler llvm libclang-dev cmake pkg-config libssl-dev clang lld"

# Try with official Debian repository first
echo "Trying with official Debian repository..."
if apt-get update && apt-get install -y $PACKAGES; then
    echo "Successfully installed with official Debian repository"
    exit 0
fi

# Try with Aliyun mirror
try_install "Aliyun" "mirrors.aliyun.com" "$PACKAGES" || \
try_install "Tsinghua" "mirrors.tuna.tsinghua.edu.cn" "$PACKAGES" || \
try_install "USTC" "mirrors.ustc.edu.cn" "$PACKAGES" || \
try_install "163" "mirrors.163.com" "$PACKAGES" || {
    echo "All mirrors failed!"
    exit 1
}
