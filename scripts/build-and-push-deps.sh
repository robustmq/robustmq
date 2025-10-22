#!/usr/bin/env bash
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

# ==============================================================================
# RobustMQ Dependency Base Image Builder
# ==============================================================================
#
# Purpose: Build and push the dependency base image to GitHub Container Registry
#
# Prerequisites:
#   - Docker installed and running
#   - Logged in to GHCR: echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
#   - Sufficient disk space (~20GB)
#
# Usage:
#   ./build-and-push.sh [TAG]
#
# Examples:
#   ./build-and-push.sh                    # Build and push as 'latest'
#   ./build-and-push.sh rust-1.90          # Build and push as 'rust-1.90'
#   ./build-and-push.sh 2025-10-20         # Build and push as '2025-10-20'
#
# ==============================================================================

set -euo pipefail

# Configuration
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Get GitHub username for package naming
get_github_user() {
    if [ -n "$GITHUB_USER" ]; then
        echo "$GITHUB_USER"
    elif [ -n "$GITHUB_TOKEN" ]; then
        curl -s -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user | grep '"login"' | cut -d'"' -f4 2>/dev/null || echo "github"
    else
        echo "github"
    fi
}

# Use user's GitHub username for package naming to avoid permission issues
readonly GITHUB_USERNAME=$(get_github_user)
readonly IMAGE_BASE="ghcr.io/${GITHUB_USERNAME}/robustmq/rust-deps"
readonly TAG="${1:-latest}"
readonly FULL_IMAGE="${IMAGE_BASE}:${TAG}"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $*${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $*${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $*${NC}"
}

log_error() {
    echo -e "${RED}❌ $*${NC}" >&2
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check disk space (need at least 20GB)
    local available_space
    # Use macOS-compatible df command
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: df -g gives output in 1K blocks, convert to GB
        available_space=$(df -g "$PROJECT_ROOT" | awk 'NR==2 {print int($4/1024/1024)}')
    else
        # Linux: use -BG for GB units
        available_space=$(df -BG "$PROJECT_ROOT" | awk 'NR==2 {print $4}' | sed 's/G//')
    fi
    if [ "$available_space" -lt 20 ]; then
        log_warning "Available disk space: ${available_space}GB (recommended: 20GB+)"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    log_success "All prerequisites met"
}

# Auto-login to GitHub Container Registry
auto_login_ghcr() {
    log_info "Checking GitHub Container Registry authentication..."
    
    # Check if already logged in
    if docker info | grep -q "ghcr.io"; then
        log_success "Already logged in to GHCR"
        return 0
    fi
    
    # Check for GITHUB_TOKEN environment variable
    if [ -z "$GITHUB_TOKEN" ]; then
        log_error "GITHUB_TOKEN environment variable is not set"
        log_info "Please set your GitHub token:"
        log_info "  export GITHUB_TOKEN=your_github_token"
        log_info "  # or add it to your ~/.bashrc or ~/.zshrc"
        exit 1
    fi
    
    # Get GitHub username from token or use current user
    local github_user
    if [ -n "$GITHUB_USER" ]; then
        github_user="$GITHUB_USER"
    else
        # Try to get username from GitHub API
        github_user=$(curl -s -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user | grep '"login"' | cut -d'"' -f4 2>/dev/null || echo "")
        if [ -z "$github_user" ]; then
            log_warning "Could not determine GitHub username, using 'github'"
            github_user="github"
        fi
    fi
    
    log_info "Logging in to GHCR as $github_user..."
    
    # Login to GHCR
    if echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$github_user" --password-stdin; then
        log_success "Successfully logged in to GHCR"
    else
        log_error "Failed to login to GHCR"
        log_info "Please check your GITHUB_TOKEN and try again"
        exit 1
    fi
    
    # Check if user has permission to push to the repository
    log_info "Checking repository permissions..."
    log_info "Using package: ${IMAGE_BASE}"
    
    # Since we're using the user's own GitHub username, they should have permission
    log_info "Package will be created under your GitHub account: $github_user"
    log_info "Package URL: https://github.com/$github_user/robustmq/pkgs/container/rust-deps"
}

# Display build information
show_build_info() {
    echo ""
    echo "=========================================="
    echo "  RobustMQ Dependency Image Builder"
    echo "=========================================="
    echo "  Image:    ${FULL_IMAGE}"
    echo "  Context:  ${PROJECT_ROOT}"
    echo "  Platform: linux/amd64"
    echo "=========================================="
    echo ""
}

# Build the image with retry mechanism
build_image() {
    log_info "Building Docker image..."
    log_info "This may take 20-40 minutes on first build..."
    
    cd "$PROJECT_ROOT"
    
    local start_time
    start_time=$(date +%s)
    
    # Build with retry mechanism
    local max_retries=3
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        log_info "Building attempt $((retry_count + 1))/$max_retries..."
        
        if DOCKER_BUILDKIT=1 docker build \
            --file docker/deps/Dockerfile.deps \
            --tag "${FULL_IMAGE}" \
            --tag "${IMAGE_BASE}:latest" \
            --build-arg BUILDKIT_INLINE_CACHE=1 \
            --progress=plain \
            .; then
            local end_time
            end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "Build completed in ${duration} seconds ($((duration / 60)) minutes)"
            return 0
        else
            log_warning "Build attempt $((retry_count + 1)) failed"
            retry_count=$((retry_count + 1))
            if [ $retry_count -lt $max_retries ]; then
                log_info "Retrying in 10 seconds..."
                sleep 10
            fi
        fi
    done
    
    log_error "Build failed after $max_retries attempts"
    exit 1
}

# Show image information
show_image_info() {
    log_info "Image information:"
    
    local image_size
    image_size=$(docker images "${FULL_IMAGE}" --format "{{.Size}}")
    echo "  Size: ${image_size}"
    
    local image_id
    image_id=$(docker images "${FULL_IMAGE}" --format "{{.ID}}")
    echo "  ID:   ${image_id}"
    
    echo ""
}

# Test the image
test_image() {
    log_info "Testing image..."
    
    # Quick smoke test
    if docker run --rm "${FULL_IMAGE}" bash -c "cargo --version && cargo nextest --version"; then
        log_success "Image test passed"
    else
        log_error "Image test failed"
        exit 1
    fi
}

# Push to registry
push_image() {
    log_info "Pushing to GitHub Container Registry..."
    
    # Push versioned tag
    if docker push "${FULL_IMAGE}"; then
        log_success "Pushed ${FULL_IMAGE}"
    else
        log_error "Failed to push ${FULL_IMAGE}"
        log_warning "Make sure you're logged in: echo \$GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin"
        exit 1
    fi
    
    # Also push 'latest' if building a specific version
    if [ "$TAG" != "latest" ]; then
        if docker push "${IMAGE_BASE}:latest"; then
            log_success "Pushed ${IMAGE_BASE}:latest"
        else
            log_warning "Failed to push latest tag (non-fatal)"
        fi
    fi
}

# Show usage instructions
show_usage() {
    cat <<EOF

${GREEN}🎉 Successfully built and pushed dependency image!${NC}

${BLUE}📋 Next Steps:${NC}

1️⃣  Update GitHub Actions workflows to use your image:
   container:
     image: ${FULL_IMAGE}
     credentials:
       username: \${{ github.actor }}
       password: \${{ secrets.GITHUB_TOKEN }}

2️⃣  Or use the original image if you have access:
   container:
     image: ghcr.io/socutes/robustmq/rust-deps:latest

3️⃣  Verify in CI that workflows use the new image

4️⃣  Monitor CI performance improvement

${BLUE}📊 When to Rebuild:${NC}

✅ Cargo.lock has 20+ dependency changes
✅ Rust version upgrades (e.g., 1.90 → 1.91)
✅ CI build time exceeds 8 minutes
❌ Don't rebuild for minor code changes

${BLUE}🔖 Version Tags:${NC}

Your Image:  ${FULL_IMAGE}
Latest:      ${IMAGE_BASE}:latest

${BLUE}💡 Tips:${NC}
- Your image is stored under your GitHub account
- You can share this image with your team
- Add this to your calendar for monthly builds!

EOF
}

# Pre-build check
pre_build_check() {
    log_info "Running pre-build checks..."
    if ! ./scripts/pre-build-check.sh; then
        log_error "Pre-build check failed"
        exit 1
    fi
    log_success "Pre-build checks passed"
}

# Main execution
main() {
    show_build_info
    check_prerequisites
    auto_login_ghcr
    pre_build_check
    build_image
    show_image_info
    test_image
    push_image
    show_usage
}

# Handle Ctrl+C gracefully
trap 'log_error "Build interrupted by user"; exit 130' INT

# Run main function
main "$@"

