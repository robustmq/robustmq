#!/bin/sh
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

set -e
# Get the directory of the script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Get the project root directory
PROJECT_ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

platform=$1
version=$2
build_dir="${PROJECT_ROOT_DIR}/build"
target="robustmq"

timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

log() {
    echo "[$(timestamp)] $1"
}

if [ -z "$version" ]; then
    log "Package version cannot be null, eg: sh scripts/build-release.sh mac-x86_64 0.1.0"
    exit 1
fi

prepare_target() {
    local arc=$1
    if ! rustup target list | grep -q "${arc} (installed)"; then
        log "Installing Rust target ${arc}..."
        rustup target add ${arc}
    fi
}

cross_build(){
    local arc=$1
    local platform_package_name=$2
    local version=$3

    local package_name=${target}-${platform_package_name}-${version}
    local package_path=${build_dir}/${package_name}

    log "Architecture: ${arc}"
    log "Building ${package_name} ..."

    # Build
    cargo build --release --target ${arc}

    # Check if the build was successful
    if [ $? -ne 0 ]; then
        log "Cargo build failed for ${arc}"
        exit 1
    fi

    mkdir -p ${package_path}/{bin,libs,config}


    binaries="mqtt-server placement-center journal-server cli-command"

    for binary in ${binaries}; do
        local bin_path="target/${arc}/release/${binary}"
        if [ -f "${bin_path}" ]; then
        cp "${bin_path}" ${package_path}/libs/
        else
            log "Error: ${binary} not found in target/${arc}/release/"
            exit 1
        fi
    done

    cp -rf "${PROJECT_ROOT_DIR}/bin/"* ${package_path}/bin/
    cp -rf "${PROJECT_ROOT_DIR}/config/"* ${package_path}/config/
    cp -rf "${PROJECT_ROOT_DIR}/example/"* ${package_path}/config/example

    # Recommended to use the 755 in the production environment
    chmod -R 777 ${package_path}/bin/*

    # Compress the package
    (cd ${build_dir} && tar zcvf ${package_name}.tar.gz ${package_name} && rm -rf ${package_name})
    cd ..

    log "Build ${package_name} successfully"
    log "You can find the package in ${build_dir}"
}

build_linux_x86_64_release() {
    # Amd64 gnu
    prepare_target "x86_64-unknown-linux-gnu"
    cross_build "x86_64-unknown-linux-gnu" "linux-x86_64-gnu" "$1"

    # Amd64 musl
    prepare_target "x86_64-unknown-linux-musl"
    cross_build "x86_64-unknown-linux-musl" "linux-x86_64-musl" "$1"
}

build_linux_arm64_release() {
    # Arm64 gnu
    prepare_target "aarch64-unknown-linux-gnu"
    cross_build "aarch64-unknown-linux-gnu" "linux-arm64-gnu" "$1"

    # Arm64 musl
    prepare_target "aarch64-unknown-linux-musl"
    cross_build "aarch64-unknown-linux-musl" "linux-arm64-musl" "$1"
}

build_mac_x86_64_release() {
    # Amd64 apple
    prepare_target "x86_64-apple-darwin"
    cross_build "x86_64-apple-darwin" "mac-x86_64-apple" "$1"
}

build_mac_arm64_release() {
    # Arm64 apple silicon
    prepare_target "aarch64-apple-darwin"
    cross_build "aarch64-apple-darwin" "mac-arm64-apple" "$1"
}

build_win_x86_64_release() {
    # Amd64 gnu
    prepare_target "x86_64-pc-windows-gnu"
    cross_build "x86_64-pc-windows-gnu" "windows-x86_64-gnu" "$1"
}

build_win_x86_release() {
    # x86 32bit gnu
    prepare_target "i686-pc-windows-gnu"
    cross_build "i686-pc-windows-gnu" "windows-x86-gnu" "$1"
}

build_win_arm64_release() {
    # Arm64 gnu
    prepare_target "aarch64-pc-windows-gnullvm"
    cross_build "aarch64-pc-windows-gnullvm" "windows-arm64-gnu" "$1"
}

build_local(){
    local package_name=${target}-${version}
    local package_path=${build_dir}/${package_name}

    log "Building ${package_name} ..."

    # Build
    cargo build
    if [ $? -ne 0 ]; then
        log "Cargo build failed"
        exit 1
    fi

    mkdir -p ${package_path}/{bin,libs,config}

    binaries="mqtt-server placement-center journal-server cli-command"

    for binary in ${binaries}; do
        local bin_path="target/debug/${binary}"
        if [ -f "${bin_path}" ]; then
            cp "${bin_path}" ${package_path}/libs/
        else
            log "Error: ${binary} not found in target/debug/"
            exit 1
        fi
    done

    cp -rf "${PROJECT_ROOT_DIR}/bin/"* ${package_path}/bin/
    cp -rf config/* ${package_path}/config/
    cp -rf example/* ${package_path}/config/example

    # Recommended to use the 755
    chmod -R 777 ${package_path}/bin/*

    # Compress the package
    (cd ${build_dir} && tar zcvf ${package_name}.tar.gz ${package_name} && rm -rf ${package_name})

    log "Build ${package_name} successfully"
    log "You can find the package in ${build_dir}"
}

# Create the build directory if it doesn't exist
mkdir -p ${build_dir}

case "$platform" in
    linux-x86_64) build_linux_x86_64_release "$version" ;;
    linux-arm64)  build_linux_arm64_release "$version" ;;
    mac-x86_64)   build_mac_x86_64_release "$version" ;;
    mac-arm64)    build_mac_arm64_release "$version" ;;
    win-x86_64)   build_win_x86_64_release "$version" ;;
    win-x86)      build_win_x86_release "$version" ;;
    win-arm64)    build_win_arm64_release "$version" ;;
    local)
        rm -rf ${build_dir}
        mkdir -p ${build_dir}
        build_local "$version"
        ;;
    *)
        log "Platform Error, optional: mac-x86_64, mac-arm64, linux-x86_64, linux-arm64, win-x86_64, win-x86, win-arm64, local"
        exit 1
        ;;
esac
