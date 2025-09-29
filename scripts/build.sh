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
BUILD_DOCKER="${BUILD_DOCKER:-false}"
OUTPUT_DIR="${OUTPUT_DIR:-${PROJECT_ROOT}/build}"

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
    echo "    --with-frontend         Build with frontend"
    echo "    --with-docker           Build Docker image"
    echo "    --clean                 Clean build directory before building"
    echo
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    # Build for current platform"
    echo "    $0"
    echo
    echo "    # Build with specific version"
    echo "    $0 --version v0.1.30"
    echo
    echo "    # Build with frontend"
    echo "    $0 --with-frontend"
    echo
    echo "    # Build Docker image"
    echo "    $0 --with-docker"
    echo
    echo -e "${BOLD}NOTES:${NC}"
    echo "    - Always builds for current platform only"
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
        *) 
            log_error "Unsupported platform: $platform"
            return 1
            ;;
    esac
}

check_dependencies() {
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "cargo not found. Please install Rust."
        return 1
    fi

    if ! command -v rustup >/dev/null 2>&1; then
        log_error "rustup not found. Please install Rust."
        return 1
    fi

    if [ "$BUILD_FRONTEND" = "true" ]; then
        if ! command -v pnpm >/dev/null 2>&1; then
            log_error "pnpm not found. Please install pnpm for frontend build."
            return 1
        fi
        
        if ! command -v git >/dev/null 2>&1; then
            log_error "git not found. Please install git for frontend repository cloning."
            return 1
        fi
    fi

    if [ "$BUILD_DOCKER" = "true" ]; then
        if ! command -v docker >/dev/null 2>&1; then
            log_error "docker command not found. Please install Docker."
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
    else
        log_info "Frontend directory exists, updating from remote"
        
        # Update existing repository
        cd "$frontend_dir"
        if ! git pull origin main; then
            log_warning "Failed to update frontend repository, continuing with existing code"
        fi
        cd "$PROJECT_ROOT"
    fi

    cd "$frontend_dir"
    
    if [ ! -f "package.json" ]; then
        log_error "package.json not found in frontend directory"
        return 1
    fi

    log_info "Installing frontend dependencies..."
    if ! pnpm install; then
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

    log_step "Building server for $platform"

    # Install target if not available
    if ! rustup target list --installed | grep -q "$rust_target"; then
        log_info "Installing Rust target: $rust_target"
        rustup target add "$rust_target"
    fi

    # Build server binaries
    log_info "Building server binaries..."
    
    local cargo_cmd="cargo build --release --target $rust_target"
    
    # Build main server
    if ! $cargo_cmd --bin broker-server; then
        log_error "Failed to build broker-server"
        return 1
    fi

    # Build CLI tools
    if ! $cargo_cmd --bin cli-command; then
        log_error "Failed to build cli-command"
        return 1
    fi

    if ! $cargo_cmd --bin cli-bench; then
        log_error "Failed to build cli-bench"
        return 1
    fi

    log_success "Server binaries built successfully"
}

create_package() {
    local version="$1"
    local platform="$2"
    local rust_target="$3"

    log_step "Creating package for $platform"

    local package_name="robustmq-$version-$platform"
    local package_dir="$OUTPUT_DIR/$package_name"
    local target_dir="$PROJECT_ROOT/target/$rust_target/release"

    # Create package directory structure
    mkdir -p "$package_dir"/{bin,config,docs}

    # Copy binaries
    local binaries=("broker-server" "cli-command" "cli-bench")
    for binary in "${binaries[@]}"; do
        local binary_path="$target_dir/$binary"
        if [[ "$platform" == windows-* ]]; then
            binary_path="${binary_path}.exe"
        fi
        
        if [ -f "$binary_path" ]; then
            cp "$binary_path" "$package_dir/bin/"
            log_info "Copied $binary"
        else
            log_warning "Binary not found: $binary_path"
        fi
    done

    # Copy configuration files
    if [ -d "$PROJECT_ROOT/config" ]; then
        cp -r "$PROJECT_ROOT/config"/* "$package_dir/config/" 2>/dev/null || true
    fi

    # Copy documentation
    if [ -f "$PROJECT_ROOT/README.md" ]; then
        cp "$PROJECT_ROOT/README.md" "$package_dir/docs/"
    fi
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        cp "$PROJECT_ROOT/LICENSE" "$package_dir/docs/"
    fi

    # Copy frontend if built
    if [ "$BUILD_FRONTEND" = "true" ]; then
        local frontend_dist="$PROJECT_ROOT/build/robustmq-copilot/packages/web-ui/dist"
        if [ -d "$frontend_dist" ]; then
            cp -r "$frontend_dist" "$package_dir/web/"
            log_info "Copied frontend files from $frontend_dist"
        else
            log_warning "Frontend dist directory not found at $frontend_dist"
        fi
    fi

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

build_docker_image() {
    if [ "$BUILD_DOCKER" != "true" ]; then
        return 0
    fi

    local version="$1"
    local dockerfile_path="$PROJECT_ROOT/docker/Dockerfile"

    log_step "Building Docker image"

    if [ ! -f "$dockerfile_path" ]; then
        log_error "Dockerfile not found at $dockerfile_path"
        return 1
    fi

    # Check if Docker daemon is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker daemon is not running. Please start Docker."
        return 1
    fi

    local image_name="robustmq/robustmq"
    local image_tag="$version"
    local full_image_name="$image_name:$image_tag"

    log_info "Building Docker image: $full_image_name"
    log_info "Using Dockerfile: $dockerfile_path"

    # Build the Docker image
    cd "$PROJECT_ROOT"
    
    if docker build -f "$dockerfile_path" -t "$full_image_name" .; then
        log_success "Docker image built successfully: $full_image_name"
        
        # Also tag as latest
        docker tag "$full_image_name" "$image_name:latest"
        log_success "Tagged as latest: $image_name:latest"
        
        # Show image info
        log_info "Docker image details:"
        docker images "$image_name" --format "table {{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.Size}}\t{{.CreatedAt}}" 2>/dev/null || docker images "$image_name"
    else
        log_error "Failed to build Docker image"
        return 1
    fi
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
            --with-frontend)
                BUILD_FRONTEND="true"
                shift
                ;;
            --with-docker)
                BUILD_DOCKER="true"
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

    # Show configuration
    echo -e "${BOLD}${BLUE}🚀 RobustMQ Build Script (Simplified)${NC}"
    echo
    log_info "Version: $VERSION"
    log_info "Platform: $platform"
    log_info "Rust Target: $rust_target"
    log_info "Build Frontend: $BUILD_FRONTEND"
    log_info "Build Docker: $BUILD_DOCKER"
    log_info "Output Directory: $OUTPUT_DIR"
    echo

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
    build_server "$VERSION" "$platform" "$rust_target"
    if [ $? -ne 0 ]; then
        exit 1
    fi

    # Create package (skip if only building Docker)
    if [ "$BUILD_DOCKER" != "true" ] || [ "$BUILD_FRONTEND" = "true" ]; then
        create_package "$VERSION" "$platform" "$rust_target"
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi

    # Build Docker image if requested
    if [ "$BUILD_DOCKER" = "true" ]; then
        build_docker_image "$VERSION"
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi

    # Show completion message
    echo
    log_success "Build completed successfully!"
    
    if [ "$BUILD_DOCKER" != "true" ] || [ "$BUILD_FRONTEND" = "true" ]; then
        log_info "Package created: $OUTPUT_DIR/robustmq-$VERSION-$platform.tar.gz"
    fi
    
    if [ "$BUILD_DOCKER" = "true" ]; then
        log_info "Docker image created: robustmq/robustmq:$VERSION"
    fi
}

# Run main function
main "$@"
