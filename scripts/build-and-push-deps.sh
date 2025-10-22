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
readonly IMAGE_BASE="ghcr.io/socutes/robustmq/rust-deps"
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

# Build the image
build_image() {
    log_info "Building Docker image..."
    log_info "This may take 20-40 minutes on first build..."
    
    cd "$PROJECT_ROOT"
    
    local start_time
    start_time=$(date +%s)
    
    # Build with buildkit for better caching
    DOCKER_BUILDKIT=1 docker build \
        --file docker/Dockerfile.deps \
        --tag "${FULL_IMAGE}" \
        --tag "${IMAGE_BASE}:latest" \
        --build-arg BUILDKIT_INLINE_CACHE=1 \
        --progress=plain \
        .
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_success "Build completed in ${duration} seconds ($((duration / 60)) minutes)"
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

1️⃣  Update GitHub Actions workflows (if not already done):
   container:
     image: ${FULL_IMAGE}
     credentials:
       username: \${{ github.actor }}
       password: \${{ secrets.GITHUB_TOKEN }}

2️⃣  Verify in CI that workflows use the new image

3️⃣  Monitor CI performance improvement

${BLUE}📊 When to Rebuild:${NC}

✅ Cargo.lock has 20+ dependency changes
✅ Rust version upgrades (e.g., 1.90 → 1.91)
✅ CI build time exceeds 8 minutes
❌ Don't rebuild for minor code changes

${BLUE}🔖 Version Tags:${NC}

Current:  ${FULL_IMAGE}
Latest:   ${IMAGE_BASE}:latest

${BLUE}💡 Tip:${NC} Add this to your calendar for monthly builds!

EOF
}

# Main execution
main() {
    show_build_info
    check_prerequisites
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

