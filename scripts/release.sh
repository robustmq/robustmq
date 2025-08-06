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

# RobustMQ Release Script
# 
# This script automates the GitHub release process for RobustMQ:
# 1. Extracts version from Cargo.toml
# 2. Creates GitHub release if it doesn't exist
# 3. Builds distribution packages using build.sh
# 4. Uploads tar.gz files to the GitHub release (SHA256 files are kept locally)
#
# Usage:
#   ./release.sh [OPTIONS]
#   
# Examples:
#   ./release.sh                        # Create release for current platform
#   ./release.sh --version v0.1.30     # Create release for specific version
#   ./release.sh --platform linux-amd64 # Build and upload specific platform only
#   ./release.sh --platform all        # Build for all supported platforms
#   ./release.sh --dry-run              # Show what would be done without executing

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
VERSION="${VERSION:-}"
PLATFORM="${PLATFORM:-auto}"  # auto, specific platform, or all
GITHUB_TOKEN="${GITHUB_TOKEN:-}"
GITHUB_REPO="${GITHUB_REPO:-robustmq/robustmq}"
DRY_RUN="${DRY_RUN:-false}"
FORCE="${FORCE:-false}"
VERBOSE="${VERBOSE:-false}"
SKIP_BUILD="${SKIP_BUILD:-false}"

# GitHub API base URL
GITHUB_API_URL="https://api.github.com"

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
    cat << EOF
${BOLD}${BLUE}RobustMQ Release Script${NC}

${BOLD}USAGE:${NC}
    $0 [OPTIONS]

${BOLD}OPTIONS:${NC}
    -h, --help                  Show this help message
    -v, --version VERSION       Release version (default: auto-detect from Cargo.toml)
    -p, --platform PLATFORM    Target platform to build (default: auto-detect current)
    -t, --token TOKEN          GitHub personal access token
    -r, --repo REPO            GitHub repository (default: robustmq/robustmq)
    --dry-run                  Show what would be done without executing
    --force                    Force recreate release if it exists
    --skip-build               Skip building packages (use existing ones)
    --verbose                  Enable verbose output

${BOLD}PLATFORMS:${NC}
    auto                      Auto-detect current platform (default)
    all                       Build for all supported platforms
    linux-amd64               Linux x86_64
    linux-arm64               Linux ARM64
    darwin-amd64              macOS x86_64
    darwin-arm64              macOS ARM64 (Apple Silicon)
    windows-amd64             Windows x86_64

${BOLD}ENVIRONMENT VARIABLES:${NC}
    GITHUB_TOKEN              GitHub personal access token
    GITHUB_REPO               GitHub repository (owner/repo format)
    VERSION                   Release version
    PLATFORM                  Target platform
    DRY_RUN                   Dry run mode (true/false)
    FORCE                     Force recreate release (true/false)
    VERBOSE                   Verbose output (true/false)
    SKIP_BUILD                Skip building packages (true/false)

${BOLD}EXAMPLES:${NC}
    # Create release for current platform (default)
    $0

    # Create release with specific version
    $0 --version v0.1.30

    # Build and upload specific platform only
    $0 --platform linux-amd64

    # Build for all platforms
    $0 --platform all

    # Dry run to see what would be done
    $0 --dry-run

    # Force recreate existing release
    $0 --force

    # Skip build and upload existing packages
    $0 --skip-build

${BOLD}PREREQUISITES:${NC}
    1. GitHub personal access token with repo permissions
    2. curl command available
    3. jq command available for JSON parsing
    4. Build dependencies (if not using --skip-build)

For more information, visit: ${GITHUB_REPO}
EOF
}

extract_version_from_cargo() {
    local cargo_file="$PROJECT_ROOT/Cargo.toml"
    
    if [ ! -f "$cargo_file" ]; then
        log_error "Cargo.toml not found at $cargo_file"
        return 1
    fi
    
    local version=$(grep '^version = ' "$cargo_file" | head -1 | sed 's/version = "\(.*\)"/\1/')
    
    if [ -z "$version" ]; then
        log_error "Could not extract version from Cargo.toml"
        return 1
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
    
    if ! command -v git >/dev/null 2>&1; then
        missing_deps+=("git")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again."
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

check_github_token() {
    if [ -z "$GITHUB_TOKEN" ]; then
        log_error "GitHub token is required. Set GITHUB_TOKEN environment variable or use --token option."
        log_info "You can create a personal access token at: https://github.com/settings/tokens"
        log_info "Required permissions: repo (Full control of private repositories)"
        return 1
    fi
}

github_api_request() {
    local method="$1"
    local endpoint="$2"
    local data="${3:-}"
    
    log_debug "GitHub API request: $method $endpoint"
    
    if [ "$DRY_RUN" = "true" ]; then
        log_debug "[DRY RUN] Would make GitHub API request: $method $endpoint"
        echo '{"dry_run": true}'
        return 0
    fi
    
    local response
    local http_code
    
    if [ -n "$data" ]; then
        log_debug "Request data: $data"
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
    
    log_debug "HTTP status code: $http_code"
    
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo "$response_body"
    else
        log_error "GitHub API request failed with HTTP $http_code"
        log_error "Response: $response_body"
        echo "$response_body"
        return 1
    fi
}

check_release_exists() {
    local version="$1"
    local response
    
    log_debug "Checking if release $version exists"
    
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
    
    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] Would create GitHub release for $version"
        return 0
    fi
    
    # Validate JSON first
    local release_data=$(cat << EOF
{
  "tag_name": "$version",
  "target_commitish": "main",
  "name": "$version",
  "body": "## What's Changed\\n\\n*This section will be automatically populated by GitHub's release notes generator.*\\n\\n## Installation\\n\\n### Linux/macOS\\n\`\`\`bash\\nwget https://github.com/$GITHUB_REPO/releases/download/$version/robustmq-$version-linux-amd64.tar.gz\\ntar -xzf robustmq-$version-linux-amd64.tar.gz\\ncd robustmq-$version-linux-amd64\\n./bin/robust-server start\\n\`\`\`\\n\\n### Windows\\nDownload the Windows package and extract it to run the server.\\n\\n### Docker (Coming Soon)\\n\`\`\`bash\\ndocker run -p 1883:1883 -p 9092:9092 robustmq/robustmq:$version\\n\`\`\`\\n\\n## Documentation\\n\\n- [📖 Documentation](https://robustmq.com/)\\n- [🚀 Quick Start Guide](https://github.com/$GITHUB_REPO/docs)\\n- [🔧 MQTT Guide](https://github.com/$GITHUB_REPO/docs)\\n\\nFor more information, visit: https://github.com/$GITHUB_REPO",
  "draft": false,
  "prerelease": false,
  "generate_release_notes": true
}
EOF
)
    
    # Validate JSON syntax
    if ! echo "$release_data" | jq . >/dev/null 2>&1; then
        log_error "Invalid JSON in release data"
        log_error "JSON: $release_data"
        return 1
    fi
    
    log_debug "Making GitHub API request to create release..."
    response=$(github_api_request "POST" "/releases" "$release_data")
    local api_exit_code=$?
    
    if [ $api_exit_code -ne 0 ]; then
        log_error "GitHub API request failed with exit code $api_exit_code"
        return 1
    fi
    
    if echo "$response" | jq -e '.id' >/dev/null 2>&1; then
        local release_id=$(echo "$response" | jq -r '.id')
        local release_url=$(echo "$response" | jq -r '.html_url // empty')
        log_success "GitHub release created successfully"
        log_info "Release ID: $release_id"
        if [ -n "$release_url" ]; then
            log_info "Release URL: $release_url"
        fi
        echo "$release_id"
    else
        log_error "Failed to create GitHub release"
        log_error "Response: $response"
        
        # Try to extract error message
        if echo "$response" | jq -e '.message' >/dev/null 2>&1; then
            local error_msg=$(echo "$response" | jq -r '.message')
            log_error "GitHub API error: $error_msg"
        fi
        
        if echo "$response" | jq -e '.errors' >/dev/null 2>&1; then
            local errors=$(echo "$response" | jq -r '.errors[]?.message // empty' | tr '\n' '; ')
            log_error "Validation errors: $errors"
        fi
        
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

upload_release_asset() {
    local release_id="$1"
    local file_path="$2"
    local filename="$(basename "$file_path")"
    
    if [ ! -f "$file_path" ]; then
        log_error "File not found: $file_path"
        return 1
    fi
    
    log_info "Uploading $filename to release..."
    
    if [ "$DRY_RUN" = "true" ]; then
        log_info "[DRY RUN] Would upload $filename"
        return 0
    fi
    
    local upload_url="https://uploads.github.com/repos/$GITHUB_REPO/releases/$release_id/assets?name=$filename"
    
    local response=$(curl -s \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Content-Type: application/gzip" \
        --data-binary "@$file_path" \
        "$upload_url")
    
    if echo "$response" | jq -e '.id' >/dev/null 2>&1; then
        log_success "Successfully uploaded $filename"
    else
        log_error "Failed to upload $filename"
        log_error "Response: $response"
        return 1
    fi
}

build_packages() {
    local version="$1"
    
    log_step "Building packages for version $version"
    
    local build_cmd="$SCRIPT_DIR/build.sh --version $version"
    
    if [ "$PLATFORM" != "all" ]; then
        build_cmd="$build_cmd --platform $PLATFORM"
    else
        build_cmd="$build_cmd --all-platforms"
    fi
    
    if [ "$DRY_RUN" = "true" ]; then
        build_cmd="$build_cmd --dry-run"
    fi
    
    if [ "$VERBOSE" = "true" ]; then
        build_cmd="$build_cmd --verbose"
    fi
    
    log_info "Running: $build_cmd"
    
    if ! $build_cmd; then
        log_error "Failed to build packages"
        return 1
    fi
    
    log_success "Packages built successfully"
}

upload_packages() {
    local version="$1"
    local release_id="$2"
    local build_dir="$PROJECT_ROOT/build"
    
    log_step "Uploading packages to GitHub release"
    
    if [ ! -d "$build_dir" ]; then
        log_error "Build directory not found: $build_dir"
        return 1
    fi
    
    local uploaded_count=0
    local failed_count=0
    
    for tarball in "$build_dir"/robustmq-$version-*.tar.gz; do
        if [ -f "$tarball" ]; then
            if upload_release_asset "$release_id" "$tarball"; then
                ((uploaded_count++))
            else
                ((failed_count++))
            fi
        fi
    done
    
    # Note: SHA256 checksum files are generated by build.sh but not uploaded to GitHub Release
    # They are available locally for manual verification if needed
    
    if [ $uploaded_count -gt 0 ]; then
        log_success "Uploaded $uploaded_count files successfully"
    fi
    
    if [ $failed_count -gt 0 ]; then
        log_error "Failed to upload $failed_count files"
        return 1
    fi
    
    if [ $uploaded_count -eq 0 ]; then
        log_warning "No packages found to upload in $build_dir"
        log_info "Expected pattern: robustmq-$version-*.tar.gz"
        return 1
    fi
}

show_release_info() {
    echo
    log_step "Release Configuration"
    log_info "Version: $VERSION"
    log_info "Platform: $PLATFORM"
    log_info "GitHub Repository: $GITHUB_REPO"
    log_info "Skip Build: $SKIP_BUILD"
    if [ "$DRY_RUN" = "true" ]; then
        log_warning "DRY RUN MODE - No actual changes will be made"
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
            -p|--platform)
                PLATFORM="$2"
                shift 2
                ;;
            -t|--token)
                GITHUB_TOKEN="$2"
                shift 2
                ;;
            -r|--repo)
                GITHUB_REPO="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            --force)
                FORCE="true"
                shift
                ;;
            --skip-build)
                SKIP_BUILD="true"
                shift
                ;;
            --verbose)
                VERBOSE="true"
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
        log_debug "Extracted version from Cargo.toml: $VERSION"
    fi
    
    # Ensure version has 'v' prefix
    if [[ "$VERSION" != v* ]]; then
        VERSION="v$VERSION"
    fi
    
    # Detect current platform if auto
    if [ "$PLATFORM" = "auto" ]; then
        PLATFORM=$(detect_current_platform)
        if [ $? -ne 0 ]; then
            exit 1
        fi
        log_debug "Detected current platform: $PLATFORM"
    fi
    
    # Show header
    if [ "$DRY_RUN" != "true" ]; then
        echo -e "${BOLD}${BLUE}🚀 RobustMQ Release Script${NC}"
    else
        echo -e "${BOLD}${YELLOW}🔍 RobustMQ Release Script (DRY RUN)${NC}"
    fi
    
    show_release_info
    
    # Check dependencies and authentication
    log_step "Checking dependencies..."
    check_dependencies
    check_github_token
    
    # Check if release already exists
    log_step "Checking if release exists..."
    local release_exists=$(check_release_exists "$VERSION")
    local release_id=""
    
    if [ "$release_exists" = "true" ]; then
        if [ "$FORCE" = "true" ]; then
            log_warning "Release $VERSION exists but --force specified, continuing..."
        else
            log_info "Release $VERSION already exists, skipping creation"
        fi
        release_id=$(get_release_id "$VERSION")
        if [ $? -ne 0 ]; then
            exit 1
        fi
    else
        # Create new release
        release_id=$(create_github_release "$VERSION")
        if [ $? -ne 0 ]; then
            exit 1
        fi
    fi
    
    # Build packages if not skipping
    if [ "$SKIP_BUILD" != "true" ]; then
        build_packages "$VERSION"
        if [ $? -ne 0 ]; then
            exit 1
        fi
    else
        log_info "Skipping package build as requested"
    fi
    
    # Upload packages to release
    upload_packages "$VERSION" "$release_id"
    if [ $? -ne 0 ]; then
        exit 1
    fi
    
    # Show completion message
    if [ "$DRY_RUN" != "true" ]; then
        echo
        log_success "Release $VERSION completed successfully!"
        log_info "GitHub Release: https://github.com/$GITHUB_REPO/releases/tag/$VERSION"
    else
        echo
        log_info "Dry run completed. No actual changes were made."
    fi
}

# Run main function
main "$@"