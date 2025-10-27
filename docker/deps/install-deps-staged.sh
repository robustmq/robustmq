#!/bin/bash
# Copyright 2023 RobustMQ Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Install system dependencies with staged approach

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
    local max_retries=5

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

# Stage 1: Install basic build tools (most stable)
echo "=== Stage 1: Installing basic build tools ==="
BASIC_PACKAGES="cmake pkg-config libssl-dev"
try_install "Huawei" "mirrors.huaweicloud.com" "$BASIC_PACKAGES" || \
try_install "Tencent" "mirrors.cloud.tencent.com" "$BASIC_PACKAGES" || \
try_install "USTC" "mirrors.ustc.edu.cn" "$BASIC_PACKAGES" || \
try_install "Tsinghua" "mirrors.tuna.tsinghua.edu.cn" "$BASIC_PACKAGES" || \
try_install "Official" "deb.debian.org" "$BASIC_PACKAGES" || {
    echo "Failed to install basic packages!"
    exit 1
}

# Stage 2: Install protobuf (usually stable)
echo "=== Stage 2: Installing protobuf ==="
PROTOBUF_PACKAGES="protobuf-compiler"
try_install "Huawei" "mirrors.huaweicloud.com" "$PROTOBUF_PACKAGES" || \
try_install "Tencent" "mirrors.cloud.tencent.com" "$PROTOBUF_PACKAGES" || \
try_install "USTC" "mirrors.ustc.edu.cn" "$PROTOBUF_PACKAGES" || \
try_install "Tsinghua" "mirrors.tuna.tsinghua.edu.cn" "$PROTOBUF_PACKAGES" || \
try_install "Official" "deb.debian.org" "$PROTOBUF_PACKAGES" || {
    echo "Failed to install protobuf!"
    exit 1
}

# Stage 3: Install LLVM/Clang (most problematic)
echo "=== Stage 3: Installing LLVM/Clang ==="
LLVM_PACKAGES="llvm clang lld"
try_install "Huawei" "mirrors.huaweicloud.com" "$LLVM_PACKAGES" || \
try_install "Tencent" "mirrors.cloud.tencent.com" "$LLVM_PACKAGES" || \
try_install "USTC" "mirrors.ustc.edu.cn" "$LLVM_PACKAGES" || \
try_install "Tsinghua" "mirrors.tuna.tsinghua.edu.cn" "$LLVM_PACKAGES" || \
try_install "Official" "deb.debian.org" "$LLVM_PACKAGES" || {
    echo "Failed to install LLVM/Clang!"
    exit 1
}

# Stage 4: Install libclang-dev (most problematic)
echo "=== Stage 4: Installing libclang-dev ==="
CLANG_DEV_PACKAGES="libclang-dev"
try_install "Huawei" "mirrors.huaweicloud.com" "$CLANG_DEV_PACKAGES" || \
try_install "Tencent" "mirrors.cloud.tencent.com" "$CLANG_DEV_PACKAGES" || \
try_install "USTC" "mirrors.ustc.edu.cn" "$CLANG_DEV_PACKAGES" || \
try_install "Tsinghua" "mirrors.tuna.tsinghua.edu.cn" "$CLANG_DEV_PACKAGES" || \
try_install "Official" "deb.debian.org" "$CLANG_DEV_PACKAGES" || {
    echo "Failed to install libclang-dev!"
    exit 1
}

echo "✅ All packages installed successfully!"
