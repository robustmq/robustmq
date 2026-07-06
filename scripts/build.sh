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

# RobustMQ Build Script (Simplified)
#
# This script builds and packages RobustMQ for the current system only.
#
# Usage:
#   ./build.sh [OPTIONS]
#
# Examples:
#   ./build.sh                    # Build for current platform
#   ./build.sh --version v0.1.0  # Build with specific version
#   ./build.sh --fast            # Build a faster local package
#   ./build.sh --diagnose-cache  # Show build cache settings
#   ./build.sh --with-frontend   # Build with frontend

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get script and project directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration variables (simplified)
VERSION="${VERSION:-}"
BUILD_FRONTEND="${BUILD_FRONTEND:-false}"
OUTPUT_DIR="${OUTPUT_DIR:-${PROJECT_ROOT}/build}"
BUILD_PROFILE="${BUILD_PROFILE:-release}"
USE_SCCACHE="${USE_SCCACHE:-auto}"
DIAGNOSE_CACHE="false"

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}" >&2
}

log_step() {
    echo -e "${BOLD}${PURPLE}🚀 $1${NC}"
}

show_help() {
    echo -e "${BOLD}${BLUE}RobustMQ Build Script (Simplified)${NC}"
    echo
    echo -e "${BOLD}USAGE:${NC}"
    echo "    $0 [OPTIONS]"
    echo
    echo -e "${BOLD}OPTIONS:${NC}"
    echo "    -h, --help              Show this help message"
    echo "    -v, --version VERSION   Build version (default: auto-detect from Cargo.toml)"
    echo "    --fast                  Use the release-fast profile for faster local packaging"
    echo "    --profile PROFILE       Cargo build profile (default: release)"
    echo "    --sccache auto|on|off   Use sccache when available (default: auto)"
    echo "    --diagnose-cache        Print build cache settings and exit"
    echo "    --with-frontend         Build with frontend"
    echo "    --clean                 Clean build directory before building"
    echo
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    # Build for current platform"
    echo "    $0"
    echo
    echo "    # Build with specific version"
    echo "    $0 --version v0.1.30"
    echo
    echo "    # Faster local package build"
    echo "    $0 --fast"
    echo
    echo "    # Inspect local cache settings"
    echo "    $0 --diagnose-cache"
    echo
    echo "    # Build with frontend"
    echo "    $0 --with-frontend"
    echo
    echo
    echo -e "${BOLD}NOTES:${NC}"
    echo "    - Always builds for current platform only"
    echo "    - Default release builds keep full optimization and LTO"
    echo "    - Output directory: $OUTPUT_DIR"
}

extract_version_from_cargo() {
    local cargo_file="$PROJECT_ROOT/Cargo.toml"

    if [ ! -f "$cargo_file" ]; then
        log_error "Cargo.toml not found at $cargo_file"
        return 1
    fi

    local version=""

    # Method 1: Look for workspace.package version
    version=$(grep -A 10 "^\[workspace\.package\]" "$cargo_file" | grep "^version" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

    # Method 2: Look for regular package version if workspace version not found
    if [ -z "$version" ]; then
        version=$(grep -A 10 "^\[package\]" "$cargo_file" | grep "^version" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
    fi

    # Method 3: Simple fallback
    if [ -z "$version" ]; then
        version=$(grep "^version\s*=" "$cargo_file" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
    fi

    if [ -z "$version" ]; then
        log_error "Could not extract version from Cargo.toml"
        log_error "Please ensure version is defined in [package] or [workspace.package] section"
        return 1
    fi

    echo "$version"
}

detect_current_platform() {
    local os_type arch_type

    case "$(uname -s)" in
        Darwin)
            os_type="darwin"
            ;;
        Linux)
            os_type="linux"
            ;;
        FreeBSD)
            os_type="freebsd"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os_type="windows"
            ;;
        *)
            log_error "Unsupported OS: $(uname -s)"
            return 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch_type="amd64"
            ;;
        arm64|aarch64)
            arch_type="arm64"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            return 1
            ;;
    esac

    echo "${os_type}-${arch_type}"
}

get_rust_target() {
    local platform="$1"
    case "$platform" in
        "linux-amd64") echo "x86_64-unknown-linux-gnu" ;;
        "linux-arm64") echo "aarch64-unknown-linux-gnu" ;;
        "darwin-amd64") echo "x86_64-apple-darwin" ;;
        "darwin-arm64") echo "aarch64-apple-darwin" ;;
        "freebsd-amd64") echo "x86_64-unknown-freebsd" ;;
        "windows-amd64") echo "x86_64-pc-windows-msvc" ;;
        "windows-arm64") echo "aarch64-pc-windows-msvc" ;;
        *)
            log_error "Unsupported platform: $platform"
            return 1
            ;;
    esac
}

get_host_rust_target() {
    rustc -vV | awk '/^host:/ { print $2 }'
}

get_profile_target_dir() {
    local profile="$1"
    case "$profile" in
        "dev") echo "debug" ;;
        "release") echo "release" ;;
        *) echo "$profile" ;;
    esac
}

get_target_dir() {
    local rust_target="$1"
    local build_profile="$2"
    local host_target="$3"
    local target_base="${CARGO_TARGET_DIR:-$PROJECT_ROOT/target}"
    local target_profile_dir
    target_profile_dir="$(get_profile_target_dir "$build_profile")"

    if [[ "$target_base" != /* ]]; then
        target_base="$PROJECT_ROOT/$target_base"
    fi

    if [[ "$rust_target" == "$host_target" ]]; then
        echo "$target_base/$target_profile_dir"
    else
        echo "$target_base/$rust_target/$target_profile_dir"
    fi
}

configure_rustc_wrapper() {
    case "$USE_SCCACHE" in
        auto)
            if [[ -z "${RUSTC_WRAPPER:-}" ]] && command -v sccache >/dev/null 2>&1; then
                export RUSTC_WRAPPER="sccache"
                log_info "Using sccache via RUSTC_WRAPPER"
            fi
            ;;
        on)
            if [[ -n "${RUSTC_WRAPPER:-}" ]]; then
                log_info "Using existing RUSTC_WRAPPER: $RUSTC_WRAPPER"
            elif command -v sccache >/dev/null 2>&1; then
                export RUSTC_WRAPPER="sccache"
                log_info "Using sccache via RUSTC_WRAPPER"
            else
                log_error "--sccache on requested, but sccache was not found in PATH"
                return 1
            fi
            ;;
        off)
            ;;
        *)
            log_error "Invalid --sccache value: $USE_SCCACHE"
            return 1
            ;;
    esac
}

show_cache_diagnostics() {
    local platform="$1"
    local rust_target="$2"
    local host_target="$3"
    local target_dir
    target_dir="$(get_target_dir "$rust_target" "$BUILD_PROFILE" "$host_target")"

    log_step "Build cache diagnostics"
    log_info "Platform: $platform"
    log_info "Rust Target: $rust_target"
    log_info "Host Rust Target: $host_target"
    log_info "Build Profile: $BUILD_PROFILE"
    log_info "Target Directory: $target_dir"
    log_info "CARGO_TARGET_DIR: ${CARGO_TARGET_DIR:-<unset>}"
    log_info "CARGO_BUILD_JOBS: ${CARGO_BUILD_JOBS:-<auto>}"
    log_info "CARGO_INCREMENTAL: ${CARGO_INCREMENTAL:-<profile/default>}"
    log_info "RUSTFLAGS: ${RUSTFLAGS:-<unset>}"
    log_info "RUSTC_WRAPPER: ${RUSTC_WRAPPER:-<unset>}"
    log_info "RUSTC_WORKSPACE_WRAPPER: ${RUSTC_WORKSPACE_WRAPPER:-<unset>}"

    if command -v sccache >/dev/null 2>&1; then
        log_info "sccache: $(command -v sccache)"
        sccache --show-stats || true
    else
        log_warning "sccache not found; compiler artifact caching is disabled"
    fi

    if [ -d "$PROJECT_ROOT/target" ]; then
        du -sh "$PROJECT_ROOT/target" 2>/dev/null || true
    fi
    if [ -d "$target_dir" ]; then
        du -sh "$target_dir" 2>/dev/null || true
    fi
}

check_dependencies() {
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "cargo not found. Please install Rust."
        echo
        log_info "Installation instructions:"
        log_info "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        log_info "  source ~/.cargo/env"
        log_info "  Or visit: https://rustup.rs/"
        echo
        return 1
    fi

    if ! command -v rustup >/dev/null 2>&1; then
        log_error "rustup not found. Please install Rust."
        echo
        log_info "Installation instructions:"
        log_info "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        log_info "  source ~/.cargo/env"
        log_info "  Or visit: https://rustup.rs/"
        echo
        return 1
    fi

    configure_rustc_wrapper

    if [ "$BUILD_FRONTEND" = "true" ]; then
        if ! command -v pnpm >/dev/null 2>&1; then
            log_error "pnpm not found. Please install pnpm for frontend build."
            echo
            log_info "Installation instructions:"
            log_info "  macOS:   brew install pnpm"
            log_info "  Linux:   curl -fsSL https://get.pnpm.io/install.sh | sh -"
            log_info "  Windows: npm install -g pnpm"
            log_info "  Or visit: https://pnpm.io/installation"
            echo
            return 1
        fi

        if ! command -v git >/dev/null 2>&1; then
            log_error "git not found. Please install git for frontend repository cloning."
            echo
            log_info "Installation instructions:"
            log_info "  macOS:   brew install git"
            log_info "  Ubuntu:  sudo apt-get install git"
            log_info "  CentOS:  sudo yum install git"
            log_info "  Windows: Download from https://git-scm.com/"
            echo
            return 1
        fi
    fi

}

build_frontend() {
    if [ "$BUILD_FRONTEND" != "true" ]; then
        return 0
    fi

    log_step "Building frontend"

    local frontend_dir="$PROJECT_ROOT/build/robustmq-copilot"
    local frontend_repo="https://github.com/robustmq/robustmq-copilot.git"

    # Check if frontend directory exists, if not clone it
    if [ ! -d "$frontend_dir" ]; then
        log_info "Frontend directory not found, cloning from $frontend_repo"

        # Ensure build directory exists
        mkdir -p "$PROJECT_ROOT/build"

        # Clone the frontend repository
        if ! git clone "$frontend_repo" "$frontend_dir"; then
            log_error "Failed to clone frontend repository from $frontend_repo"
            return 1
        fi

        log_success "Frontend repository cloned successfully"
    fi

    # Always pull latest code before building
    log_info "Updating frontend code to latest version"
    cd "$frontend_dir"
    if ! git pull origin main; then
        log_warning "Failed to update frontend repository, continuing with existing code"
    else
        log_success "Frontend code updated successfully"
    fi
    cd "$PROJECT_ROOT"

    cd "$frontend_dir"

    if [ ! -f "package.json" ]; then
        log_error "package.json not found in frontend directory"
        return 1
    fi

    log_info "Installing frontend dependencies..."
    if ! pnpm install --no-frozen-lockfile; then
        log_error "Failed to install frontend dependencies"
        return 1
    fi

    log_info "Building frontend..."
    if ! pnpm ui:build; then
        log_error "Failed to build frontend"
        return 1
    fi

    cd "$PROJECT_ROOT"
    log_success "Frontend built successfully"
}

build_server() {
    local version="$1"
    local platform="$2"
    local rust_target="$3"
    local build_profile="$4"
    local host_target="$5"

    log_step "Building server for $platform"

    # Install cross target if not available. The host target is already available.
    if [[ "$rust_target" != "$host_target" ]] && ! rustup target list --installed | grep -q "$rust_target"; then
        log_info "Installing Rust target: $rust_target"
        rustup target add "$rust_target"
    fi

    # Build server binaries
    log_info "Building server binaries with profile: $build_profile"

    local cargo_args=(
        build
        --profile "$build_profile"
        --bin broker-server
        --bin cli-command
        --bin cli-bench
    )

    if [[ "$rust_target" != "$host_target" ]]; then
        cargo_args+=(--target "$rust_target")
    else
        log_info "Using host target cache: target/$(get_profile_target_dir "$build_profile")"
    fi

    if ! cargo "${cargo_args[@]}"; then
        log_error "Failed to build server binaries"
        return 1
    fi

    log_success "Server binaries built successfully"
}

create_package() {
    local version="$1"
    local platform="$2"
    local rust_target="$3"
    local build_profile="$4"
    local host_target="$5"

    log_step "Creating package for $platform"

    local package_name="robustmq-$version-$platform"
    local package_dir="$OUTPUT_DIR/$package_name"
    local target_dir
    target_dir="$(get_target_dir "$rust_target" "$build_profile" "$host_target")"

    # Create package directory structure
    mkdir -p "$package_dir"/{bin,libs,config,dist}

    # Copy bin directory from source code (scripts, startup files, etc.)
    # On Windows the shell scripts in bin/ are not executable; skip them and
    # leave a note explaining how to start the server directly.
    if [[ "$platform" == windows-* ]]; then
        cat > "$package_dir/bin/README.txt" << 'WINEOF'
Windows users: the shell scripts in this directory are not supported on Windows.
Start the server directly from the libs/ directory:

  libs\broker-server.exe --config config\server.toml

Management tool:
  libs\cli-command.exe --help

Benchmark tool:
  libs\cli-bench.exe --help
WINEOF
        log_info "Skipped shell scripts in bin/ for Windows; added README.txt"
    elif [ -d "$PROJECT_ROOT/bin" ]; then
        cp -r "$PROJECT_ROOT/bin"/* "$package_dir/bin/" 2>/dev/null || true
        log_info "Copied source bin directory"
    fi

    # Copy Rust compiled binaries to libs directory
    local binaries=("broker-server" "cli-command" "cli-bench")
    local found_binaries=()
    for binary in "${binaries[@]}"; do
        local binary_path="$target_dir/$binary"
        if [[ "$platform" == windows-* ]]; then
            binary_path="${binary_path}.exe"
        fi

        if [ -f "$binary_path" ]; then
            cp "$binary_path" "$package_dir/libs/"
            found_binaries+=("$binary")
            log_info "Copied binary $binary to libs/"
        else
            log_warning "Binary not found: $binary_path"
        fi
    done

    if [ ${#found_binaries[@]} -eq 0 ]; then
        log_error "No binaries found for $platform"
        return 1
    fi

    # Copy entire config directory into package
    if [ -d "$PROJECT_ROOT/config" ]; then
        cp -r "$PROJECT_ROOT/config"/. "$package_dir/config/" 2>/dev/null || true
        log_info "Copied config directory"
    else
        log_warning "config directory not found"
    fi

    # Copy LICENSE to root directory
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        cp "$PROJECT_ROOT/LICENSE" "$package_dir/"
        log_info "Copied LICENSE to root directory"
    fi

    # Copy frontend build results to dist directory
    if [ "$BUILD_FRONTEND" = "true" ]; then
        local frontend_dist="$PROJECT_ROOT/build/robustmq-copilot/packages/web-ui/dist"
        if [ -d "$frontend_dist" ]; then
            cp -r "$frontend_dist"/* "$package_dir/dist/" 2>/dev/null || true
            log_info "Copied frontend files to dist/"
        else
            log_warning "Frontend dist directory not found at $frontend_dist"
        fi
    fi


    # Create package info
    local frontend_status="Not included"
    if [ -d "$package_dir/dist" ] && [ "$(ls -A "$package_dir/dist" 2>/dev/null)" ]; then
        frontend_status="Included"
    fi

    cat > "$package_dir/package-info.txt" << EOF
Package: robustmq-server
Version: $version
Platform: $platform
Target: $rust_target
Build Profile: $build_profile
Build Date: $(TZ='Asia/Shanghai' date '+%Y-%m-%d %H:%M:%S CST')
Binaries: ${found_binaries[*]}
Frontend Web UI: $frontend_status
EOF

    # Set permissions for executable files
    chmod -R 755 "$package_dir/bin/"* 2>/dev/null || true
    chmod -R 755 "$package_dir/libs/"* 2>/dev/null || true

    # Create tarball
    cd "$OUTPUT_DIR"
    local tarball="$package_name.tar.gz"

    if tar -czf "$tarball" "$package_name"; then
        log_success "Created package: $tarball"

        # Clean up directory
        rm -rf "$package_name"
    else
        log_error "Failed to create tarball"
        return 1
    fi

    cd "$PROJECT_ROOT"
}


main() {
    cd "$PROJECT_ROOT"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            --with-frontend)
                BUILD_FRONTEND="true"
                shift
                ;;
            --fast)
                BUILD_PROFILE="release-fast"
                shift
                ;;
            --profile)
                if [[ $# -lt 2 || -z "$2" ]]; then
                    log_error "--profile requires a Cargo profile name"
                    exit 1
                fi
                BUILD_PROFILE="$2"
                shift 2
                ;;
            --sccache)
                if [[ $# -lt 2 || -z "$2" ]]; then
                    log_error "--sccache requires one of: auto, on, off"
                    exit 1
                fi
                USE_SCCACHE="$2"
                shift 2
                ;;
            --diagnose-cache)
                DIAGNOSE_CACHE="true"
                shift
                ;;
            --clean)
                log_info "Cleaning build directory..."
                rm -rf "$OUTPUT_DIR"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Extract version if not provided
    if [ -z "$VERSION" ]; then
        VERSION=$(extract_version_from_cargo)
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi

    # Detect current platform
    local platform=$(detect_current_platform)
    if [ $? -ne 0 ]; then
        exit 1
    fi

    local rust_target=$(get_rust_target "$platform")
    if [ $? -ne 0 ]; then
        exit 1
    fi
    local host_target=$(get_host_rust_target)
    if [ -z "$host_target" ]; then
        log_error "Could not detect host Rust target"
        exit 1
    fi

    # Show configuration
    echo -e "${BOLD}${BLUE}🚀 RobustMQ Build Script (Simplified)${NC}"
    echo
    log_info "Version: $VERSION"
    log_info "Platform: $platform"
    log_info "Rust Target: $rust_target"
    log_info "Host Rust Target: $host_target"
    log_info "Build Profile: $BUILD_PROFILE"
    log_info "sccache Mode: $USE_SCCACHE"
    log_info "Build Frontend: $BUILD_FRONTEND"
    log_info "Output Directory: $OUTPUT_DIR"
    echo

    if [ "$DIAGNOSE_CACHE" = "true" ]; then
        configure_rustc_wrapper
        show_cache_diagnostics "$platform" "$rust_target" "$host_target"
        exit 0
    fi

    # Check dependencies
    log_step "Checking dependencies..."
    check_dependencies

    # Create output directory
    mkdir -p "$OUTPUT_DIR"

    # Build frontend if requested
    if [ "$BUILD_FRONTEND" = "true" ]; then
        build_frontend
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi

    # Build server
    build_server "$VERSION" "$platform" "$rust_target" "$BUILD_PROFILE" "$host_target"
    if [ $? -ne 0 ]; then
        exit 1
    fi

    # Create package
    create_package "$VERSION" "$platform" "$rust_target" "$BUILD_PROFILE" "$host_target"
    if [ $? -ne 0 ]; then
        exit 1
    fi

    # Show completion message
    echo
    log_success "Build completed successfully!"
    log_info "Package created: $OUTPUT_DIR/robustmq-$VERSION-$platform.tar.gz"
}

# Run main function
main "$@"
