#!/bin/bash
# Install runtime dependencies with mirror fallback

set -e

# Function to try installing packages with a specific mirror
try_install() {
    local mirror_name="$1"
    local mirror_url="$2"
    local packages="$3"
    
    echo "Trying to install with $mirror_name mirror..."
    
    # Update sources to use the mirror
    sed -i "s/deb.debian.org/$mirror_url/g" /etc/apt/sources.list.d/debian.sources
    
    # Update package list with retry
    local retry_count=0
    local max_retries=3
    
    while [ $retry_count -lt $max_retries ]; do
        if apt-get update; then
            break
        else
            echo "apt-get update failed, retrying... ($((retry_count + 1))/$max_retries)"
            retry_count=$((retry_count + 1))
            sleep 5
        fi
    done
    
    if [ $retry_count -eq $max_retries ]; then
        echo "Failed to update package list after $max_retries attempts"
        return 1
    fi
    
    # Install packages with retry
    retry_count=0
    while [ $retry_count -lt $max_retries ]; do
        if apt-get install -y $packages; then
            echo "Successfully installed with $mirror_name mirror"
            return 0
        else
            echo "Package installation failed, retrying... ($((retry_count + 1))/$max_retries)"
            retry_count=$((retry_count + 1))
            sleep 10
        fi
    done
    
    echo "Failed to install packages after $max_retries attempts"
    return 1
}

# Define packages to install
PACKAGES="ca-certificates curl clang lld libclang-dev"

# Try with multiple mirrors in order of reliability (most stable first)
# Skip Huawei if it's having 502 issues and try others first
try_install "Tencent" "mirrors.cloud.tencent.com" "$PACKAGES" || \
try_install "USTC" "mirrors.ustc.edu.cn" "$PACKAGES" || \
try_install "Tsinghua" "mirrors.tuna.tsinghua.edu.cn" "$PACKAGES" || \
try_install "Huawei" "mirrors.huaweicloud.com" "$PACKAGES" || \
try_install "Official" "deb.debian.org" "$PACKAGES" || {
    echo "All mirrors failed!"
    echo "Please check your network connection and try again."
    exit 1
}
