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

# RobustMQ Build Script
#
# This script builds and packages RobustMQ for different operating systems and architectures.
# It supports both server and operator components with cross-compilation capabilities.
#
# Usage:
#   ./build.sh [OPTIONS]
#
# Examples:
#   ./build.sh                           # Build for current platform
#   ./build.sh --platform linux-amd64   # Build for specific platform
#   ./build.sh --component server        # Build only server component
#   ./build.sh --all-platforms           # Build for all supported platforms
#   ./build.sh --version v0.1.0         # Build with specific version

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get script and project directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration variables
VERSION="${VERSION:-$(git describe --tags --always 2>/dev/null || echo 'dev')}"
COMPONENT="${COMPONENT:-server}"  # server, operator, or all
PLATFORM="${PLATFORM:-auto}"  # auto, specific platform, or all
BUILD_TYPE="${BUILD_TYPE:-release}"  # release or debug
OUTPUT_DIR="${OUTPUT_DIR:-${PROJECT_ROOT}/build}"
CLEAN="${CLEAN:-false}"
VERBOSE="${VERBOSE:-false}"
DRY_RUN="${DRY_RUN:-false}"
PARALLEL="${PARALLEL:-true}"

# Global state variables
FRONTEND_BUILT="${FRONTEND_BUILT:-false}"

# Platform mappings (compatible with bash 3.2+)
get_rust_target() {
    case "$1" in
        "linux-amd64") echo "x86_64-unknown-linux-gnu" ;;
        "linux-amd64-musl") echo "x86_64-unknown-linux-musl" ;;
        "linux-arm64") echo "aarch64-unknown-linux-gnu" ;;
        "linux-arm64-musl") echo "aarch64-unknown-linux-musl" ;;
        "linux-armv7") echo "armv7-unknown-linux-gnueabihf" ;;
        "darwin-amd64") echo "x86_64-apple-darwin" ;;
        "darwin-arm64") echo "aarch64-apple-darwin" ;;
        "windows-amd64") echo "x86_64-pc-windows-gnu" ;;
        "windows-386") echo "i686-pc-windows-gnu" ;;
        "windows-arm64") echo "aarch64-pc-windows-gnullvm" ;;
        "freebsd-amd64") echo "x86_64-unknown-freebsd" ;;
        *) echo "" ;;
    esac
}

get_go_target() {
    case "$1" in
        "linux-amd64") echo "linux/amd64" ;;
        "linux-arm64") echo "linux/arm64" ;;
        "linux-armv7") echo "linux/arm" ;;
        "darwin-amd64") echo "darwin/amd64" ;;
        "darwin-arm64") echo "darwin/arm64" ;;
        "windows-amd64") echo "windows/amd64" ;;
        "windows-386") echo "windows/386" ;;
        "freebsd-amd64") echo "freebsd/amd64" ;;
        *) echo "" ;;
    esac
}

# Default platforms to build when --all-platforms is specified
ALL_PLATFORMS=(
    "linux-amd64"
    "linux-arm64"
    "darwin-amd64"
    "darwin-arm64"
    "windows-amd64"
)

# Helper functions
timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

log_info() {
    echo -e "${BLUE}[$(timestamp)] ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}[$(timestamp)] ✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}[$(timestamp)] ⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}[$(timestamp)] ❌ Error: $1${NC}" >&2
}

log_step() {
    echo -e "${BOLD}${PURPLE}[$(timestamp)] 🚀 $1${NC}"
}

log_debug() {
    if [ "$VERBOSE" = "true" ]; then
        echo -e "${CYAN}[$(timestamp)] 🔍 $1${NC}"
    fi
}

show_help() {
    echo -e "${BOLD}${BLUE}RobustMQ Build Script${NC}"
    echo
    echo -e "${BOLD}USAGE:${NC}"
    echo "    $0 [OPTIONS]"
    echo
    echo -e "${BOLD}OPTIONS:${NC}"
    echo "    -h, --help                  Show this help message"
    echo "    -v, --version VERSION       Build version (default: auto-detect from git)"
    echo "    -c, --component COMP        Component to build: server, operator, or all (default: server)"
    echo "    -p, --platform PLATFORM    Target platform (default: auto-detect)"
    echo "    -a, --all-platforms         Build for all supported platforms"
    echo "    -t, --type TYPE            Build type: release or debug (default: release)"
    echo "    -o, --output DIR           Output directory (default: ./build)"
    echo "    --clean                    Clean output directory before building"
    echo "    --verbose                  Enable verbose output"
    echo "    --dry-run                  Show what would be built without actually building"
    echo "    --no-parallel              Disable parallel builds"
    echo
    echo -e "${BOLD}PLATFORMS:${NC}"
    echo "    linux-amd64               Linux x86_64"
    echo "    linux-amd64-musl          Linux x86_64 (musl)"
    echo "    linux-arm64               Linux ARM64"
    echo "    linux-arm64-musl          Linux ARM64 (musl)"
    echo "    linux-armv7               Linux ARMv7"
    echo "    darwin-amd64              macOS x86_64"
    echo "    darwin-arm64              macOS ARM64 (Apple Silicon)"
    echo "    windows-amd64             Windows x86_64"
    echo "    windows-386               Windows x86"
    echo "    windows-arm64             Windows ARM64"
    echo "    freebsd-amd64             FreeBSD x86_64"
    echo
    echo -e "${BOLD}COMPONENTS:${NC}"
    echo "    server                    RobustMQ server binaries (Rust) with embedded web UI"
    echo "    operator                  RobustMQ Kubernetes operator (Go)"
    echo "    all                       Both server and operator components"
    echo
    echo -e "${BOLD}ENVIRONMENT VARIABLES:${NC}"
    echo "    VERSION                   Build version"
    echo "    COMPONENT                 Component to build"
    echo "    PLATFORM                  Target platform"
    echo "    BUILD_TYPE                Build type (release/debug)"
    echo "    OUTPUT_DIR                Output directory"
    echo "    CLEAN                     Clean before build (true/false)"
    echo "    VERBOSE                   Verbose output (true/false)"
    echo "    DRY_RUN                   Dry run mode (true/false)"
    echo "    PARALLEL                  Parallel builds (true/false)"
    echo
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    # Build for current platform (server only)"
    echo "    $0"
    echo
    echo "    # Build specific component for specific platform"
    echo "    $0 --component server --platform linux-amd64"
    echo
    echo "    # Build all platforms"
    echo "    $0 --all-platforms"
    echo
    echo "    # Build with custom version"
    echo "    $0 --version v1.0.0"
    echo
    echo "    # Build for development (debug mode)"
    echo "    $0 --type debug"
    echo
    echo "    # Clean build"
    echo "    $0 --clean"
    echo
    echo "    # Dry run to see what would be built"
    echo "    $0 --dry-run --all-platforms"
    echo
    echo "For more information, visit: https://github.com/robustmq/robustmq"
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
        MINGW*|MSYS*|CYGWIN*)
            os_type="windows"
            ;;
        FreeBSD)
            os_type="freebsd"
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
        armv7l)
            arch_type="armv7"
            ;;
        i386|i686)
            arch_type="386"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            return 1
            ;;
    esac

    echo "${os_type}-${arch_type}"
}

check_dependencies() {
    local missing_deps=()

    # Check for required tools
    if [ "$COMPONENT" = "server" ] || [ "$COMPONENT" = "all" ]; then
        if ! command -v cargo >/dev/null 2>&1; then
            missing_deps+=("cargo (Rust)")
        fi
        if ! command -v rustup >/dev/null 2>&1; then
            missing_deps+=("rustup")
        fi
    fi

    if [ "$COMPONENT" = "operator" ] || [ "$COMPONENT" = "all" ]; then
        if ! command -v go >/dev/null 2>&1; then
            missing_deps+=("go")
        fi
    fi

    # Frontend dependencies are checked in build_frontend function when needed

    if ! command -v tar >/dev/null 2>&1; then
        missing_deps+=("tar")
    fi

    if ! command -v git >/dev/null 2>&1; then
        missing_deps+=("git")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again."
        return 1
    fi
}

build_frontend() {
    # Check if frontend is already built
    if [ "$FRONTEND_BUILT" = "true" ]; then
        log_info "Frontend already built, skipping..."
        return 0
    fi
    
    log_step "Building frontend web UI"
    
    local copilot_dir="${PROJECT_ROOT}/../robustmq-copilot"
    local dist_dir="${copilot_dir}/packages/web-ui/dist"
    
    # Check if robustmq-copilot directory exists
    if [ ! -d "$copilot_dir" ]; then
        log_warning "Frontend directory does not exist, will clone from GitHub"
        log_info "Cloning RobustMQ Copilot from GitHub..."
        
        if [ "$DRY_RUN" = "true" ]; then
            log_debug "[DRY RUN] Would clone: git clone https://github.com/robustmq/robustmq-copilot.git $copilot_dir"
            FRONTEND_BUILT="true"
            return 0
        fi
        
        if ! git clone https://github.com/robustmq/robustmq-copilot.git "$copilot_dir"; then
            log_error "Failed to clone robustmq-copilot repository"
            return 1
        fi
        
        log_success "Successfully cloned robustmq-copilot"
    else
        log_info "Found existing robustmq-copilot directory"
    fi
    
    # Check if pnpm is available
    if ! command -v pnpm >/dev/null 2>&1; then
        log_error "pnpm is required to build the frontend. Please install pnpm first."
        log_info "Install pnpm: npm install -g pnpm"
        return 1
    fi
    
    if [ "$DRY_RUN" = "true" ]; then
        log_debug "[DRY RUN] Would build frontend in: $copilot_dir"
        FRONTEND_BUILT="true"
        return 0
    fi
    
    # Change to copilot directory and build
    log_info "Building frontend with pnpm..."
    (
        cd "$copilot_dir" || exit 1
        
        # Install dependencies if node_modules doesn't exist
        if [ ! -d "node_modules" ]; then
            log_info "Installing frontend dependencies..."
            if ! pnpm install; then
                log_error "Failed to install frontend dependencies"
                exit 1
            fi
        fi
        
        # Build the frontend
        log_info "Running: pnpm ui:build"
        if ! pnpm ui:build; then
            log_error "Failed to build frontend"
            exit 1
        fi
    )
    
    # Check if build was successful
    if [ -d "$dist_dir" ]; then
        log_success "Frontend build completed successfully"
        log_info "Frontend build output: $dist_dir"
        FRONTEND_BUILT="true"
        return 0
    else
        log_error "Frontend build failed - dist directory not found: $dist_dir"
        return 1
    fi
}

prepare_rust_target() {
    local target="$1"

    if [ "$DRY_RUN" = "true" ]; then
        log_debug "[DRY RUN] Would prepare Rust target: $target"
        return 0
    fi

    log_debug "Checking Rust target: $target"
    if ! rustup target list | grep -q "${target} (installed)"; then
        log_info "Installing Rust target: $target"
        if ! rustup target add "$target"; then
            log_error "Failed to install Rust target: $target"
            return 1
        fi
    fi
}

build_server_component() {
    local platform="$1"
    local rust_target="$(get_rust_target "$platform")"
    local output_name="robustmq-${VERSION}-${platform}"
    local package_path="${OUTPUT_DIR}/${output_name}"

    if [ -z "$rust_target" ]; then
        log_error "Unsupported platform for server component: $platform"
        return 1
    fi

    log_step "Building server component for $platform"

    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] Would build server for $platform using target $rust_target"
        log_info "[DRY RUN] Would build frontend web UI"
        # Mark frontend as built in dry-run mode to avoid duplicate builds
        FRONTEND_BUILT="true"
        return 0
    fi

    # Build frontend first
    if ! build_frontend; then
        log_error "Failed to build frontend for server component"
        return 1
    fi

    # Prepare Rust target
    prepare_rust_target "$rust_target"

    # Build
    log_info "Compiling Rust binaries for $rust_target..."
    local cargo_cmd="cargo build --target $rust_target"
    if [ "$BUILD_TYPE" = "release" ]; then
        cargo_cmd="$cargo_cmd --release"
    fi

    log_info "Running: $cargo_cmd"
    if ! $cargo_cmd; then
        log_error "Failed to build server component for $platform"
        return 1
    fi

    # Create package structure
    mkdir -p "$package_path"/{bin,libs,config,docs,dist}

    # Copy binaries
    local target_dir="target/$rust_target"
    if [ "$BUILD_TYPE" = "release" ]; then
        target_dir="$target_dir/release"
    else
        target_dir="$target_dir/debug"
    fi

    local binaries="broker-server cli-command cli-bench"
    local found_binaries=()

    for binary in $binaries; do
        local bin_file="$binary"
        if [[ "$platform" == windows-* ]]; then
            bin_file="$binary.exe"
        fi

        local bin_path="$target_dir/$bin_file"
        if [ -f "$bin_path" ]; then
            cp "$bin_path" "$package_path/libs/"
            found_binaries+=("$binary")
            log_debug "Copied binary: $binary"
        else
            log_warning "Binary not found: $bin_path"
        fi
    done

    if [ ${#found_binaries[@]} -eq 0 ]; then
        log_error "No binaries found for $platform"
        return 1
    fi

    # Copy additional files
    if [ -d "$PROJECT_ROOT/bin" ]; then
        cp -r "$PROJECT_ROOT/bin/"* "$package_path/bin/" 2>/dev/null || true
    fi

    if [ -d "$PROJECT_ROOT/config" ]; then
        cp -r "$PROJECT_ROOT/config/"* "$package_path/config/" 2>/dev/null || true
    fi

    # Copy frontend build results
    local copilot_dist_dir="${PROJECT_ROOT}/../robustmq-copilot/packages/web-ui/dist"
    if [ -d "$copilot_dist_dir" ]; then
        log_info "Copying frontend build results to package..."
        cp -r "$copilot_dist_dir/"* "$package_path/dist/" 2>/dev/null || true
        log_success "Frontend assets copied to dist directory"
    else
        log_warning "Frontend build directory not found: $copilot_dist_dir"
    fi

    # Create version file
    echo "$VERSION" > "$package_path/config/version.txt"

    # Create package info
    local frontend_status="Not included"
    if [ -d "$package_path/dist" ] && [ "$(ls -A "$package_path/dist" 2>/dev/null)" ]; then
        frontend_status="Included"
    fi
    
    cat > "$package_path/package-info.txt" << EOF
Package: robustmq-server
Version: $VERSION
Platform: $platform
Target: $rust_target
Build Type: $BUILD_TYPE
Build Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
Binaries: ${found_binaries[*]}
Frontend Web UI: $frontend_status
EOF

    # Set permissions
    chmod -R 755 "$package_path/bin/"* 2>/dev/null || true
    chmod -R 755 "$package_path/libs/"* 2>/dev/null || true

    # Create tarball
    log_info "Creating tarball for $platform..."
    (cd "$OUTPUT_DIR" && tar -czf "${output_name}.tar.gz" "$(basename "$package_path")")

    # Calculate checksum
    (cd "$OUTPUT_DIR" && sha256sum "${output_name}.tar.gz" > "${output_name}.tar.gz.sha256")

    # Clean up directory
    rm -rf "$package_path"

    log_success "Server component built successfully: ${output_name}.tar.gz"
}

build_operator_component() {
    local platform="$1"
    local go_target="$(get_go_target "$platform")"
    local output_name="robustmq-operator-${VERSION}-${platform}"
    local package_path="${OUTPUT_DIR}/${output_name}"

    if [ -z "$go_target" ]; then
        log_error "Unsupported platform for operator component: $platform"
        return 1
    fi

    log_step "Building operator component for $platform"

    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] Would build operator for $platform using target $go_target"
        return 0
    fi

    # Build operator
    log_info "Compiling Go binary for $go_target..."

    local goos="${go_target%/*}"
    local goarch="${go_target#*/}"

    # Handle special case for ARMv7
    local goarm=""
    if [ "$goarch" = "arm" ] && [[ "$platform" == *"armv7"* ]]; then
        goarm="7"
    fi

    local binary_name="robustmq-operator"
    if [ "$goos" = "windows" ]; then
        binary_name="robustmq-operator.exe"
    fi

    local build_cmd="env GOOS=$goos GOARCH=$goarch"
    if [ -n "$goarm" ]; then
        build_cmd="$build_cmd GOARM=$goarm"
    fi

    # Set Go build flags
    local ldflags="-s -w -X main.version=$VERSION -X main.buildDate=$(date -u '+%Y-%m-%d_%H:%M:%S')"
    build_cmd="$build_cmd go build -ldflags \"$ldflags\""

    if [ "$BUILD_TYPE" = "debug" ]; then
        build_cmd="$build_cmd -gcflags=\"-N -l\""
    fi

    build_cmd="$build_cmd -o $binary_name ."

    log_debug "Running: $build_cmd"

    # Change to operator directory and build
    (
        cd "$PROJECT_ROOT/operator"
        if ! eval "$build_cmd"; then
            log_error "Failed to build operator component for $platform"
            exit 1
        fi
    )

    # Create package structure
    mkdir -p "$package_path"/{bin,config,manifests,docs}

    # Move binary
    mv "$PROJECT_ROOT/operator/$binary_name" "$package_path/bin/"

    # Copy operator-specific files
    if [ -f "$PROJECT_ROOT/operator/robustmq.yaml" ]; then
        cp "$PROJECT_ROOT/operator/robustmq.yaml" "$package_path/manifests/"
    fi

    if [ -f "$PROJECT_ROOT/operator/sample-robustmq.yaml" ]; then
        cp "$PROJECT_ROOT/operator/sample-robustmq.yaml" "$package_path/manifests/"
    fi

    if [ -f "$PROJECT_ROOT/operator/README.md" ]; then
        cp "$PROJECT_ROOT/operator/README.md" "$package_path/docs/"
    fi

    # Create version file
    echo "$VERSION" > "$package_path/config/version.txt"

    # Create package info
    cat > "$package_path/package-info.txt" << EOF
Package: robustmq-operator
Version: $VERSION
Platform: $platform
Target: $go_target
Build Type: $BUILD_TYPE
Build Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
Binary: $binary_name
EOF

    # Set permissions
    chmod 755 "$package_path/bin/$binary_name"

    # Create tarball
    log_info "Creating tarball for $platform..."
    (cd "$OUTPUT_DIR" && tar -czf "${output_name}.tar.gz" "$(basename "$package_path")")

    # Calculate checksum
    (cd "$OUTPUT_DIR" && sha256sum "${output_name}.tar.gz" > "${output_name}.tar.gz.sha256")

    # Clean up directory
    rm -rf "$package_path"

    log_success "Operator component built successfully: ${output_name}.tar.gz"
}

build_platform() {
    local platform="$1"

    log_step "Building for platform: $platform"

    # Validate platform
    if [[ "$COMPONENT" == "server" || "$COMPONENT" == "all" ]] && [[ -z "$(get_rust_target "$platform")" ]]; then
        log_error "Platform $platform is not supported for server component"
        return 1
    fi

    if [[ "$COMPONENT" == "operator" || "$COMPONENT" == "all" ]] && [[ -z "$(get_go_target "$platform")" ]]; then
        log_error "Platform $platform is not supported for operator component"
        return 1
    fi

    # Build components
    if [ "$COMPONENT" = "server" ] || [ "$COMPONENT" = "all" ]; then
        if ! build_server_component "$platform"; then
            return 1
        fi
    fi

    if [ "$COMPONENT" = "operator" ] || [ "$COMPONENT" = "all" ]; then
        if ! build_operator_component "$platform"; then
            return 1
        fi
    fi
}

build_all_platforms() {
    local failed_platforms=()
    local successful_platforms=()

    log_step "Building for all platforms: ${ALL_PLATFORMS[*]}"

    for platform in "${ALL_PLATFORMS[@]}"; do
        log_info "Building platform: $platform"
        if build_platform "$platform"; then
            successful_platforms+=("$platform")
        else
            failed_platforms+=("$platform")
            log_error "Failed to build for platform: $platform"
        fi
    done

    # Summary
    echo
    log_step "Build Summary"
    if [ ${#successful_platforms[@]} -gt 0 ]; then
        log_success "Successfully built for: ${successful_platforms[*]}"
    fi

    if [ ${#failed_platforms[@]} -gt 0 ]; then
        log_error "Failed to build for: ${failed_platforms[*]}"
        return 1
    fi
}

cleanup_output_dir() {
    if [ "$CLEAN" = "true" ] && [ -d "$OUTPUT_DIR" ]; then
        log_info "Cleaning output directory: $OUTPUT_DIR"
        if [ "$DRY_RUN" = "true" ]; then
            log_debug "[DRY RUN] Would clean: $OUTPUT_DIR"
        else
            rm -rf "$OUTPUT_DIR"
        fi
    fi
}

show_build_info() {
    echo
    log_step "Build Configuration"
    log_info "Version: $VERSION"
    log_info "Component: $COMPONENT"
    log_info "Platform: $PLATFORM"
    log_info "Build Type: $BUILD_TYPE"
    log_info "Output Directory: $OUTPUT_DIR"
    log_info "Parallel Builds: $PARALLEL"
    if [ "$DRY_RUN" = "true" ]; then
        log_warning "DRY RUN MODE - No actual building will occur"
    fi
    echo
}

main() {
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
            -c|--component)
                COMPONENT="$2"
                shift 2
                ;;
            -p|--platform)
                PLATFORM="$2"
                shift 2
                ;;
            -a|--all-platforms)
                PLATFORM="all"
                shift
                ;;
            -t|--type)
                BUILD_TYPE="$2"
                shift 2
                ;;
            -o|--output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --clean)
                CLEAN="true"
                shift
                ;;
            --verbose)
                VERBOSE="true"
                shift
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            --no-parallel)
                PARALLEL="false"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Validate component
    case "$COMPONENT" in
        server|operator|all)
            ;;
        *)
            log_error "Invalid component: $COMPONENT"
            log_info "Valid components: server, operator, all"
            exit 1
            ;;
    esac

    # Validate build type
    case "$BUILD_TYPE" in
        release|debug)
            ;;
        *)
            log_error "Invalid build type: $BUILD_TYPE"
            log_info "Valid build types: release, debug"
            exit 1
            ;;
    esac

    # Detect current platform if auto
    if [ "$PLATFORM" = "auto" ]; then
        PLATFORM=$(detect_current_platform)
        log_debug "Detected current platform: $PLATFORM"
    fi

    # Show header
    if [ "$DRY_RUN" != "true" ]; then
        echo -e "${BOLD}${BLUE}🔨 RobustMQ Build Script${NC}"
    else
        echo -e "${BOLD}${YELLOW}🔍 RobustMQ Build Script (DRY RUN)${NC}"
    fi

    show_build_info

    # Check dependencies
    log_step "Checking dependencies..."
    check_dependencies

    # Cleanup if requested
    cleanup_output_dir

    # Create output directory
    if [ "$DRY_RUN" != "true" ]; then
        mkdir -p "$OUTPUT_DIR"
    fi

    # Build
    if [ "$PLATFORM" = "all" ]; then
        build_all_platforms
    else
        build_platform "$PLATFORM"
    fi

    # Show completion message
    if [ "$DRY_RUN" != "true" ]; then
        echo
        log_success "Build completed successfully!"
        log_info "Output directory: $OUTPUT_DIR"
        if [ -d "$OUTPUT_DIR" ]; then
            log_info "Generated packages:"
            find "$OUTPUT_DIR" -name "*.tar.gz" -exec basename {} \; | sed 's/^/  • /'
        fi
    else
        echo
        log_info "Dry run completed. No files were actually built."
    fi
}

# Run main function
main "$@"
