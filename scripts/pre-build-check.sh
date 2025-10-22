#!/bin/bash
# Pre-build check script to ensure base images are available

set -e

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

# Function to try pulling an image with retries
try_pull_image() {
    local image="$1"
    local max_retries=3
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        log_info "Attempting to pull $image (attempt $((retry_count + 1))/$max_retries)..."
        
        if docker pull "$image" >/dev/null 2>&1; then
            log_success "Successfully pulled $image"
            return 0
        else
            log_warning "Failed to pull $image, retrying..."
            retry_count=$((retry_count + 1))
            sleep 5
        fi
    done
    
    log_error "Failed to pull $image after $max_retries attempts"
    return 1
}

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    log_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

log_info "Checking base images availability..."

# List of required images
IMAGES=(
    "rust:1.90.0-bookworm"
)

# Try to pull each image
for image in "${IMAGES[@]}"; do
    if ! try_pull_image "$image"; then
        log_error "Cannot proceed without base image: $image"
        log_info "Please check your network connection and try again."
        log_info "You can also try: docker pull $image"
        exit 1
    fi
done

log_success "All base images are available. Proceeding with build..."
