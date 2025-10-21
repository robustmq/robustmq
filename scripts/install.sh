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
# for your operating system and architecture. It downloads pre-built binaries from
# GitHub releases using the naming convention: robustmq-{VERSION}-{platform}.tar.gz
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
# Check if terminal supports colors
if [ -t 1 ] && [ "${TERM:-}" != "dumb" ] && [ "${NO_COLOR:-}" != "1" ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    # Disable colors if not in a proper terminal
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

# Configuration variables
OS_TYPE=""
ARCH_TYPE=""
VERSION="${VERSION:-latest}"
GITHUB_ORG="robustmq"
GITHUB_REPO="robustmq"
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
    echo -e "${BOLD}RobustMQ Installation Script${NC}"
    echo
    echo -e "${BOLD}USAGE:${NC}"
    echo "    $0 [OPTIONS]"
    echo
    echo -e "${BOLD}OPTIONS:${NC}"
    echo "    -h, --help              Show this help message"
    echo "    -v, --version VERSION   Install specific version (default: latest)"
    echo "    -d, --dir DIRECTORY     Installation directory (default: auto-detect)"
    echo "    -s, --silent            Silent installation"
    echo "    -f, --force             Force installation even if already exists"
    echo "    --dry-run               Show what would be installed without actually installing"
    echo
    echo -e "${BOLD}ENVIRONMENT VARIABLES:${NC}"
    echo "    VERSION                 Version to install"
    echo "    INSTALL_DIR            Installation directory"
    echo "    SILENT                 Silent mode (true/false)"
    echo "    FORCE                  Force installation (true/false)"
    echo "    DRY_RUN                Dry run mode (true/false)"
    echo
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    # Install latest version"
    echo "    $0"
    echo
    echo "    # Install specific version"
    echo "    VERSION=v0.1.0 $0"
    echo
    echo "    # Install to custom directory"
    echo "    INSTALL_DIR=/usr/local/bin $0"
    echo
    echo "    # Silent installation"
    echo "    SILENT=true $0"
    echo
    echo "For more information, visit: https://github.com/robustmq/robustmq"
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
        armv7l)
            ARCH_TYPE="armv7"
            ;;
        armv6l)
            ARCH_TYPE="armv6"
            ;;
        *)
            log_error "Unsupported CPU architecture: $arch_type"
            log_info "Supported architectures (based on build.sh): amd64, arm64, armv7"
            log_info "Your architecture '$arch_type' may not have pre-built binaries available"
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

check_platform_support() {
    local platform="${OS_TYPE}-${ARCH_TYPE}"

    # Check if this platform is supported based on build.sh logic
    case "$platform" in
        "linux-amd64"|"linux-arm64"|"darwin-amd64"|"darwin-arm64"|"windows-amd64")
            log_info "Platform $platform is supported"
            return 0
            ;;
        "linux-386"|"windows-386"|"freebsd-amd64"|"linux-armv7")
            log_warning "Platform $platform has limited support"
            return 0
            ;;
        *)
            log_error "Platform $platform is not supported"
            log_info "Supported platforms: linux-amd64, linux-arm64, darwin-amd64, darwin-arm64, windows-amd64"
            return 1
            ;;
    esac
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

install_robustmq() {
    # Convert platform format to match release naming convention
    local platform="${OS_TYPE}-${ARCH_TYPE}"

    # Package naming follows the pattern: robustmq-{VERSION}-{platform}.tar.gz
    local archive_name="robustmq-${VERSION}-${platform}.tar.gz"
    local download_url="https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download/${VERSION}/${archive_name}"
    local temp_dir
    temp_dir=$(mktemp -d)
    local temp_file="${temp_dir}/${archive_name}"

    # Define binaries to install
    local binaries_to_install=("broker-server" "cli-command" "cli-bench")

    # Check if already installed
    local already_installed=true
    for binary in "${binaries_to_install[@]}"; do
        local final_path="${INSTALL_DIR}/${binary}"
        if [ ! -f "$final_path" ] || [ "$FORCE" = "true" ]; then
            already_installed=false
            break
        fi
    done

    if [ "$already_installed" = "true" ] && [ "$FORCE" != "true" ]; then
        log_warning "RobustMQ is already installed"
        log_info "Use FORCE=true to overwrite the existing installation"
        return 0
    fi

    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] Would install RobustMQ ${VERSION} to ${INSTALL_DIR}"
        for binary in "${binaries_to_install[@]}"; do
            log_info "[DRY RUN] Would install ${binary}"
        done
        return 0
    fi

    log_step "Installing RobustMQ ${VERSION}..."

    # Download the archive
    log_info "Downloading ${archive_name}..."
    if ! download_file "$download_url" "$temp_file"; then
        log_error "Failed to download RobustMQ"
        log_info "Download URL: $download_url"
        log_info "Please check:"
        log_info "  1. Version '$VERSION' exists for platform '$platform'"
        log_info "  2. Internet connection is working"
        log_info "  3. GitHub releases page: https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases"
        rm -rf "$temp_dir"
        return 1
    fi

    # Verify checksum if available
    verify_checksum "$temp_file" "$download_url" || true

    # Extract the archive
    log_info "Extracting RobustMQ..."
    if ! tar -xzf "$temp_file" -C "$temp_dir"; then
        log_error "Failed to extract ${archive_name}"
        rm -rf "$temp_dir"
        return 1
    fi

    # Install each binary
    local installed_count=0
    for binary in "${binaries_to_install[@]}"; do
        local final_path="${INSTALL_DIR}/${binary}"
        
        # Find the binary in the extracted files
        # Based on build.sh logic, binaries are in libs/ directory
        local extracted_binary=""
        local search_paths=(
            "libs/${binary}"
            "bin/${binary}"
            "${binary}"
        )

        # Try different possible locations for the binary
        for search_path in "${search_paths[@]}"; do
            local full_path=$(find "$temp_dir" -path "*/${search_path}" -type f | head -n1)
            if [ -n "$full_path" ] && [ -f "$full_path" ]; then
                extracted_binary="$full_path"
                break
            fi
        done

        # If not found in expected paths, try generic search
        if [ -z "$extracted_binary" ]; then
            extracted_binary=$(find "$temp_dir" -name "$binary" -type f | head -n1)
        fi

        if [ -z "$extracted_binary" ]; then
            log_warning "Binary ${binary} not found in the archive, skipping..."
            continue
        fi

        # Install the binary
        log_info "Installing ${binary} to ${final_path}..."
        if ! cp "$extracted_binary" "$final_path"; then
            log_error "Failed to install ${binary} to ${final_path}"
            continue
        fi

        # Make it executable
        chmod +x "$final_path"
        installed_count=$((installed_count + 1))
        
        # Verify installation
        if "$final_path" --version >/dev/null 2>&1; then
            log_success "${binary} installed successfully"
        else
            log_success "${binary} installed to ${final_path}"
        fi
    done

    # Clean up
    rm -rf "$temp_dir"

    if [ $installed_count -eq 0 ]; then
        log_error "No binaries were installed"
        return 1
    fi

    log_success "RobustMQ ${VERSION} installed successfully ($installed_count binaries)"
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
    echo "  1. Run '${INSTALL_DIR}/broker-server --help' to see available options"
    echo "  2. Start the server: '${INSTALL_DIR}/broker-server start'"
    echo "  3. Use CLI tools: '${INSTALL_DIR}/cli-command' and '${INSTALL_DIR}/cli-bench'"
    echo "  4. Check the documentation: https://robustmq.com/docs"

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

    # Check platform support
    if ! check_platform_support; then
        exit 1
    fi

    # Get version if latest
    if [ "$VERSION" = "latest" ]; then
        get_latest_version
    fi

    log_info "Version: ${VERSION}"

    if [ "$DRY_RUN" = "true" ]; then
        log_info "Running in dry-run mode..."
    fi

    # Install RobustMQ
    install_robustmq

    # Add to PATH if needed
    if [ "$DRY_RUN" != "true" ] && [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        add_to_path
    fi

    show_completion_message
}

# Run main function
main "$@"
