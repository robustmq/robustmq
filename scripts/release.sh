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

# RobustMQ Release Script (Simplified)
#
# This script provides essential GitHub release functionality:
# 1. Create GitHub release and upload current platform package
# 2. Upload current platform package to existing release
#
# Usage:
#   ./release-simple.sh [OPTIONS]
#
# Examples:
#   ./release-simple.sh                     # Create release for current platform
#   ./release-simple.sh --version v0.1.30  # Create release for specific version
#   ./release-simple.sh --upload-only      # Upload to existing release

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
GITHUB_TOKEN="${GITHUB_TOKEN:-}"
GITHUB_REPO="${GITHUB_REPO:-robustmq/robustmq}"
UPLOAD_ONLY="${UPLOAD_ONLY:-false}"

# GitHub API base URL
GITHUB_API_URL="https://api.github.com"

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
    echo -e "${BOLD}${BLUE}RobustMQ Release Script (Simplified)${NC}"
    echo
    echo -e "${BOLD}USAGE:${NC}"
    echo "    $0 [OPTIONS]"
    echo
    echo -e "${BOLD}OPTIONS:${NC}"
    echo "    -h, --help                  Show this help message"
    echo "    -v, --version VERSION       Release version (default: auto-detect from Cargo.toml)"
    echo "    -t, --token TOKEN          GitHub personal access token"
    echo "    --upload-only              Only upload current platform to existing release"
    echo
    echo -e "${BOLD}ENVIRONMENT VARIABLES:${NC}"
    echo "    GITHUB_TOKEN              GitHub personal access token"
    echo "    VERSION                   Release version"
    echo "    UPLOAD_ONLY               Upload to existing release only (true/false)"
    echo
    echo -e "${BOLD}EXAMPLES:${NC}"
    echo "    # Create release for current platform"
    echo "    $0"
    echo
    echo "    # Create release with specific version"
    echo "    $0 --version v0.1.30"
    echo
    echo "    # Upload current platform to existing release"
    echo "    $0 --upload-only --version v0.1.30"
    echo
    echo -e "${BOLD}NOTES:${NC}"
    echo "    - Always builds and uploads current platform only"
    echo "    - Requires GitHub token with repo permissions"
    echo "    - Frontend is always built (no option to disable)"
}

extract_version_from_cargo() {
    local cargo_file="$PROJECT_ROOT/Cargo.toml"

    if [ ! -f "$cargo_file" ]; then
        log_error "Cargo.toml not found at $cargo_file"
        return 1
    fi

    # Try multiple methods to extract version
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

    # Validate version format
    if [ -z "$version" ]; then
        log_error "Could not extract version from Cargo.toml"
        log_error "Please ensure version is defined in [package] or [workspace.package] section"
        return 1
    fi

    # Validate semantic version format (basic check)
    if ! echo "$version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$'; then
        log_warning "Version '$version' may not follow semantic versioning format"
    fi

    # Add 'v' prefix if not present
    if [[ "$version" != v* ]]; then
        version="v$version"
    fi

    echo "$version"
}

check_dependencies() {
    local missing_deps=()

    if ! command -v curl >/dev/null 2>&1; then
        missing_deps+=("curl")
    fi

    if ! command -v jq >/dev/null 2>&1; then
        missing_deps+=("jq")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        return 1
    fi
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

check_github_token() {
    if [ -z "$GITHUB_TOKEN" ]; then
        log_error "GitHub token is required. Set GITHUB_TOKEN environment variable or use --token option."
        return 1
    fi
}

github_api_request() {
    local method="$1"
    local endpoint="$2"
    local data="${3:-}"

    local response
    local http_code

    if [ -n "$data" ]; then
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
            -X "$method" \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$GITHUB_API_URL/repos/$GITHUB_REPO$endpoint")
    else
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
            -X "$method" \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "$GITHUB_API_URL/repos/$GITHUB_REPO$endpoint")
    fi

    http_code=$(echo "$response" | grep -o "HTTPSTATUS:[0-9]*" | cut -d: -f2)
    response_body=$(echo "$response" | sed 's/HTTPSTATUS:[0-9]*$//')

    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo "$response_body"
    else
        log_error "GitHub API request failed with HTTP $http_code"
        echo "$response_body"
        return 1
    fi
}

check_release_exists() {
    local version="$1"
    local response

    response=$(github_api_request "GET" "/releases/tags/$version" 2>/dev/null || echo '{"message": "Not Found"}')

    if echo "$response" | jq -e '.id' >/dev/null 2>&1; then
        echo "true"
    else
        echo "false"
    fi
}

create_github_release() {
    local version="$1"
    local response

    log_step "Creating GitHub release for version $version"

    local release_data=$(cat << EOF
{
  "tag_name": "$version",
  "target_commitish": "main",
  "name": "$version",
  "body": "Release $version",
  "draft": false,
  "prerelease": false
}
EOF
)

    response=$(github_api_request "POST" "/releases" "$release_data")

    if echo "$response" | jq -e '.id' >/dev/null 2>&1; then
        local release_id=$(echo "$response" | jq -r '.id')
        log_success "GitHub release created successfully (ID: $release_id)"
        echo "$release_id"
    else
        log_error "Failed to create GitHub release"
        return 1
    fi
}

get_release_id() {
    local version="$1"
    local response

    response=$(github_api_request "GET" "/releases/tags/$version")

    if echo "$response" | jq -e '.id' >/dev/null 2>&1; then
        echo "$response" | jq -r '.id'
    else
        log_error "Failed to get release ID for $version"
        return 1
    fi
}

build_package() {
    local version="$1"
    local platform="$2"

    log_step "Building package for $platform"

    local build_cmd="$SCRIPT_DIR/build.sh --version $version --platform $platform --with-frontend"

    if ! $build_cmd; then
        log_error "Failed to build package"
        return 1
    fi

    log_success "Package built successfully"
}

upload_package() {
    local version="$1"
    local release_id="$2"
    local platform="$3"
    local build_dir="$PROJECT_ROOT/build"

    log_step "Uploading package to GitHub release"

    local tarball="$build_dir/robustmq-$version-$platform.tar.gz"

    if [ ! -f "$tarball" ]; then
        log_error "Package not found: $tarball"
        return 1
    fi

    local filename="$(basename "$tarball")"
    local upload_url="https://uploads.github.com/repos/$GITHUB_REPO/releases/$release_id/assets?name=$filename"

    log_info "Uploading $filename..."

    local response=$(curl -s \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Content-Type: application/gzip" \
        --data-binary "@$tarball" \
        "$upload_url")

    if echo "$response" | jq -e '.id' >/dev/null 2>&1; then
        log_success "Successfully uploaded $filename"
    else
        log_error "Failed to upload $filename"
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
            -t|--token)
                GITHUB_TOKEN="$2"
                shift 2
                ;;
            --upload-only)
                UPLOAD_ONLY="true"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Extract version from Cargo.toml if not provided
    if [ -z "$VERSION" ]; then
        VERSION=$(extract_version_from_cargo)
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi

    # Ensure version has 'v' prefix
    if [[ "$VERSION" != v* ]]; then
        VERSION="v$VERSION"
    fi

    # Detect current platform
    local platform=$(detect_current_platform)
    if [ $? -ne 0 ]; then
        exit 1
    fi

    # Show configuration
    echo -e "${BOLD}${BLUE}🚀 RobustMQ Release Script (Simplified)${NC}"
    echo
    log_info "Version: $VERSION"
    log_info "Platform: $platform"
    log_info "Repository: $GITHUB_REPO"
    log_info "Upload Only: $UPLOAD_ONLY"
    echo

    # Check dependencies and authentication
    log_step "Checking dependencies..."
    check_dependencies
    check_github_token

    local release_id=""

    if [ "$UPLOAD_ONLY" = "true" ]; then
        # Upload-only mode: check if release exists
        log_step "Checking if release exists..."
        local release_exists=$(check_release_exists "$VERSION")
        
        if [ "$release_exists" != "true" ]; then
            log_error "Release $VERSION does not exist"
            log_error "Upload-only mode requires an existing release"
            exit 1
        fi
        
        release_id=$(get_release_id "$VERSION")
        log_success "Release $VERSION exists (ID: $release_id)"
    else
        # Normal mode: create release if it doesn't exist
        log_step "Checking if release exists..."
        local release_exists=$(check_release_exists "$VERSION")
        
        if [ "$release_exists" = "true" ]; then
            log_info "Release $VERSION already exists"
            release_id=$(get_release_id "$VERSION")
        else
            release_id=$(create_github_release "$VERSION")
            if [ $? -ne 0 ]; then
                exit 1
            fi
        fi
    fi

    # Build package
    build_package "$VERSION" "$platform"
    if [ $? -ne 0 ]; then
        exit 1
    fi

    # Upload package
    upload_package "$VERSION" "$release_id" "$platform"
    if [ $? -ne 0 ]; then
        exit 1
    fi

    # Show completion message
    echo
    log_success "Operation completed successfully!"
    log_info "GitHub Release: https://github.com/$GITHUB_REPO/releases/tag/$VERSION"
}

# Run main function
main "$@"
