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

# RobustMQ Installation Script
# 
# This script automatically downloads and installs the latest version of RobustMQ
# for your operating system and architecture.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
#   
# Options:
#   VERSION=v0.1.0 bash install.sh    # Install specific version
#   INSTALL_DIR=/usr/local/bin bash install.sh  # Install to specific directory
#   COMPONENT=server bash install.sh # Install specific component (server/operator)
#   SILENT=true bash install.sh      # Silent installation

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration variables
OS_TYPE=""
ARCH_TYPE=""
VERSION="${VERSION:-latest}"
GITHUB_ORG="robustmq"
GITHUB_REPO="robustmq"
COMPONENT="${COMPONENT:-server}"  # server, operator, or all
INSTALL_DIR="${INSTALL_DIR:-}"
SILENT="${SILENT:-false}"
FORCE="${FORCE:-false}"
DRY_RUN="${DRY_RUN:-false}"

# Helper functions
log_info() {
    if [ "$SILENT" != "true" ]; then
        echo -e "${BLUE}ℹ️  $1${NC}"
    fi
}

log_success() {
    if [ "$SILENT" != "true" ]; then
        echo -e "${GREEN}✅ $1${NC}"
    fi
}

log_warning() {
    if [ "$SILENT" != "true" ]; then
        echo -e "${YELLOW}⚠️  $1${NC}"
    fi
}

log_error() {
    echo -e "${RED}❌ Error: $1${NC}" >&2
}

log_step() {
    if [ "$SILENT" != "true" ]; then
        echo -e "${BOLD}🚀 $1${NC}"
    fi
}

show_help() {
    cat << EOF
${BOLD}RobustMQ Installation Script${NC}

${BOLD}USAGE:${NC}
    $0 [OPTIONS]

${BOLD}OPTIONS:${NC}
    -h, --help              Show this help message
    -v, --version VERSION   Install specific version (default: latest)
    -c, --component COMP    Component to install: server, operator, or all (default: server)
    -d, --dir DIRECTORY     Installation directory (default: auto-detect)
    -s, --silent            Silent installation
    -f, --force             Force installation even if already exists
    --dry-run               Show what would be installed without actually installing

${BOLD}ENVIRONMENT VARIABLES:${NC}
    VERSION                 Version to install
    COMPONENT              Component to install
    INSTALL_DIR            Installation directory
    SILENT                 Silent mode (true/false)
    FORCE                  Force installation (true/false)
    DRY_RUN                Dry run mode (true/false)

${BOLD}EXAMPLES:${NC}
    # Install latest server
    $0

    # Install specific version
    VERSION=v0.1.0 $0

    # Install operator component
    COMPONENT=operator $0

    # Install to custom directory
    INSTALL_DIR=/usr/local/bin $0

    # Silent installation
    SILENT=true $0

${BOLD}COMPONENTS:${NC}
    server                 RobustMQ server binary
    operator               RobustMQ Kubernetes operator
    all                    All components

For more information, visit: https://github.com/robustmq/robustmq
EOF
}

check_dependencies() {
    local missing_deps=()
    
    # Check for required tools
    if ! command -v tar >/dev/null 2>&1; then
        missing_deps+=("tar")
    fi
    
    # Check for download tools (wget or curl)
    if ! command -v wget >/dev/null 2>&1 && ! command -v curl >/dev/null 2>&1; then
        missing_deps+=("wget or curl")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again."
        exit 1
    fi
}

detect_os() {
    local os_type
    os_type="$(uname -s)"

    case "$os_type" in
        Darwin)
            OS_TYPE="darwin"
            ;;
        Linux)
            OS_TYPE="linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS_TYPE="windows"
            ;;
        FreeBSD)
            OS_TYPE="freebsd"
            ;;
        *)
            log_error "Unsupported OS type: $os_type"
            log_info "Supported OS types: Linux, macOS (Darwin), Windows, FreeBSD"
            exit 1
            ;;
    esac
}

detect_arch() {
    local arch_type
    arch_type="$(uname -m)"

    case "$arch_type" in
        arm64|aarch64)
            ARCH_TYPE="arm64"
            ;;
        x86_64|amd64)
            ARCH_TYPE="amd64"
            ;;
        i386|i686)
            ARCH_TYPE="386"
            ;;
        armv6l)
            ARCH_TYPE="armv6"
            ;;
        armv7l)
            ARCH_TYPE="armv7"
            ;;
        *)
            log_error "Unsupported CPU architecture: $arch_type"
            log_info "Supported architectures: amd64, arm64, 386, armv6, armv7"
            exit 1
            ;;
    esac
}

determine_install_dir() {
    if [ -n "$INSTALL_DIR" ]; then
        return
    fi

    # Try to determine the best installation directory
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -w "$HOME/.local/bin" ] || [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        # Ensure the directory exists (only if not in dry-run mode)
        if [ "$DRY_RUN" != "true" ]; then
            mkdir -p "$INSTALL_DIR"
        fi
    elif [ -w "$HOME/bin" ] || [ -d "$HOME/bin" ]; then
        INSTALL_DIR="$HOME/bin"
        if [ "$DRY_RUN" != "true" ]; then
            mkdir -p "$INSTALL_DIR"
        fi
    else
        INSTALL_DIR="$HOME/.robustmq/bin"
        if [ "$DRY_RUN" != "true" ]; then
            mkdir -p "$INSTALL_DIR"
        fi
        log_warning "Installing to $INSTALL_DIR (you may need to add this to your PATH)"
    fi
}

get_latest_version() {
    log_info "Fetching latest version information..."
    
    local api_url="https://api.github.com/repos/${GITHUB_ORG}/${GITHUB_REPO}/releases/latest"
    local version=""
    
    if command -v curl >/dev/null 2>&1; then
        version=$(curl -s "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        version=$(wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    fi
    
    if [ -z "$version" ]; then
        log_error "Failed to fetch the latest version"
        log_info "Please specify a version manually using: VERSION=v0.1.0 $0"
        exit 1
    fi
    
    VERSION="$version"
}

download_file() {
    local url="$1"
    local output="$2"
    
    log_info "Downloading from: $url"
    
    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL -o "$output" "$url"; then
            log_error "Failed to download using curl"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q -O "$output" "$url"; then
            log_error "Failed to download using wget"
            return 1
        fi
    else
        log_error "No download tool available (curl or wget)"
        return 1
    fi
}

verify_checksum() {
    local file="$1"
    local checksum_url="${2}.sha256"
    local checksum_file="${file}.sha256"
    
    # Download checksum file if available
    if download_file "$checksum_url" "$checksum_file" 2>/dev/null; then
        log_info "Verifying checksum..."
        
        if command -v sha256sum >/dev/null 2>&1; then
            if ! echo "$(cat "$checksum_file")  $file" | sha256sum -c --quiet; then
                log_error "Checksum verification failed"
                rm -f "$checksum_file"
                return 1
            fi
        elif command -v shasum >/dev/null 2>&1; then
            if ! echo "$(cat "$checksum_file")  $file" | shasum -a 256 -c --quiet; then
                log_error "Checksum verification failed"
                rm -f "$checksum_file"
                return 1
            fi
        else
            log_warning "No checksum tool available, skipping verification"
        fi
        
        rm -f "$checksum_file"
        log_success "Checksum verification passed"
    else
        log_warning "Checksum file not available, skipping verification"
    fi
}

install_component() {
    local component="$1"
    local binary_name=""
    
    case "$component" in
        server)
            binary_name="robustmq"
            ;;
        operator)
            binary_name="robustmq-operator"
            ;;
        *)
            log_error "Unknown component: $component"
            return 1
            ;;
    esac
    
    local package_name="${binary_name}-${OS_TYPE}-${ARCH_TYPE}"
    local archive_name="${package_name}.tar.gz"
    local download_url="https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download/${VERSION}/${archive_name}"
    local temp_dir
    temp_dir=$(mktemp -d)
    local temp_file="${temp_dir}/${archive_name}"
    local final_path="${INSTALL_DIR}/${binary_name}"
    
    # Check if already installed
    if [ -f "$final_path" ] && [ "$FORCE" != "true" ]; then
        local existing_version
        existing_version=$("$final_path" --version 2>/dev/null | head -n1 || echo "unknown")
        log_warning "${binary_name} is already installed at $final_path (${existing_version})"
        log_info "Use FORCE=true to overwrite the existing installation"
        return 0
    fi
    
    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] Would install ${binary_name} ${VERSION} to ${final_path}"
        return 0
    fi
    
    log_step "Installing ${binary_name} ${VERSION}..."
    
    # Download the archive
    if ! download_file "$download_url" "$temp_file"; then
        log_error "Failed to download ${binary_name}"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Verify checksum if available
    verify_checksum "$temp_file" "$download_url" || true
    
    # Extract the archive
    log_info "Extracting ${binary_name}..."
    if ! tar -xzf "$temp_file" -C "$temp_dir"; then
        log_error "Failed to extract ${archive_name}"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Find the binary in the extracted files
    local extracted_binary
    extracted_binary=$(find "$temp_dir" -name "$binary_name" -type f | head -n1)
    
    if [ -z "$extracted_binary" ]; then
        log_error "Binary ${binary_name} not found in the archive"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Install the binary
    log_info "Installing ${binary_name} to ${final_path}..."
    if ! mv "$extracted_binary" "$final_path"; then
        log_error "Failed to install ${binary_name} to ${final_path}"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Make it executable
    chmod +x "$final_path"
    
    # Clean up
    rm -rf "$temp_dir"
    
    # Verify installation
    if "$final_path" --version >/dev/null 2>&1; then
        log_success "${binary_name} ${VERSION} installed successfully to ${final_path}"
    else
        log_success "${binary_name} ${VERSION} installed to ${final_path}"
    fi
}

add_to_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_info "Adding $INSTALL_DIR to PATH..."
        
        # Determine the shell profile file
        local profile_file=""
        if [ -n "${BASH_VERSION:-}" ]; then
            profile_file="$HOME/.bashrc"
        elif [ -n "${ZSH_VERSION:-}" ]; then
            profile_file="$HOME/.zshrc"
        elif [ -f "$HOME/.profile" ]; then
            profile_file="$HOME/.profile"
        fi
        
        if [ -n "$profile_file" ]; then
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$profile_file"
            log_info "Added $INSTALL_DIR to PATH in $profile_file"
            log_warning "Please restart your shell or run: source $profile_file"
        else
            log_warning "Please add $INSTALL_DIR to your PATH manually"
        fi
    fi
}

show_completion_message() {
    if [ "$SILENT" = "true" ]; then
        return
    fi
    
    echo
    log_success "Installation completed successfully!"
    echo
    echo -e "${BOLD}Next steps:${NC}"
    
    case "$COMPONENT" in
        server)
            echo "  1. Run '${INSTALL_DIR}/robustmq --help' to see available options"
            echo "  2. Start the server: '${INSTALL_DIR}/robustmq'"
            echo "  3. Check the documentation: https://robustmq.com/docs"
            ;;
        operator)
            echo "  1. Deploy to Kubernetes: 'kubectl apply -f operator/robustmq.yaml'"
            echo "  2. Create a RobustMQ instance: 'kubectl apply -f operator/sample-robustmq.yaml'"
            echo "  3. Check the operator documentation: https://robustmq.com/docs/operator"
            ;;
        all)
            echo "  1. For server: Run '${INSTALL_DIR}/robustmq --help'"
            echo "  2. For operator: Deploy to Kubernetes with 'kubectl apply -f operator/robustmq.yaml'"
            echo "  3. Check the documentation: https://robustmq.com/docs"
            ;;
    esac
    
    echo
    echo -e "${BLUE}Support:${NC}"
    echo "  • Documentation: https://robustmq.com/docs"
    echo "  • GitHub Issues: https://github.com/robustmq/robustmq/issues"
    echo "  • Community: https://github.com/robustmq/robustmq/discussions"
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
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            -s|--silent)
                SILENT="true"
                shift
                ;;
            -f|--force)
                FORCE="true"
                shift
                ;;
            --dry-run)
                DRY_RUN="true"
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
    
    if [ "$SILENT" != "true" ]; then
        echo -e "${BOLD}${BLUE}🚀 RobustMQ Installation Script${NC}"
        echo
    fi
    
    log_step "Checking system requirements..."
    check_dependencies
    detect_os
    detect_arch
    determine_install_dir
    
    log_info "Detected system: ${OS_TYPE}/${ARCH_TYPE}"
    log_info "Installation directory: ${INSTALL_DIR}"
    log_info "Component: ${COMPONENT}"
    
    # Get version if latest
    if [ "$VERSION" = "latest" ]; then
        get_latest_version
    fi
    
    log_info "Version: ${VERSION}"
    
    if [ "$DRY_RUN" = "true" ]; then
        log_info "Running in dry-run mode..."
    fi
    
    # Install components
    case "$COMPONENT" in
        server)
            install_component "server"
            ;;
        operator)
            install_component "operator"
            ;;
        all)
            install_component "server"
            install_component "operator"
            ;;
    esac
    
    # Add to PATH if needed
    if [ "$DRY_RUN" != "true" ] && [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        add_to_path
    fi
    
    show_completion_message
}

# Run main function
main "$@"
