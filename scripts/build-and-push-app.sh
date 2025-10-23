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

# ============================================================================
# RobustMQ Application Image - Build & Push Helper
# ============================================================================
#
# Purpose:
#   Build the RobustMQ application Docker image (docker/robustmq/Dockerfile) and push
#   it to a container registry (GHCR or Docker Hub).
#
# Prerequisites:
#   - Docker installed and running
#   - Logged in to your target registry (GHCR or Docker Hub)
#       GHCR:  echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
#       Hub :  docker login
#
# Usage:
#   ./build-and-push-app.sh [--version VERSION] [--org ORG] [--name NAME] \
#                           [--registry ghcr|dockerhub] [--target STAGE] \
#                           [--platform linux/amd64] [--push-latest]
#
# Examples:
#   ./build-and-push-app.sh --version 0.2.0 --org socutes --registry ghcr
#   ./build-and-push-app.sh --version 0.2.0 --org myuser --registry dockerhub
#   ./build-and-push-app.sh --version 0.2.0 --org socutes --target mqtt-broker
#   ./build-and-push-app.sh --version 0.2.0 --org socutes --platform linux/amd64
#   ./build-and-push-app.sh --version 0.2.0 --org socutes --push-latest
#
# Notes:
#   - The Dockerfile supports multiple stages: meta-service, mqtt-broker,
#     journal-service, all-in-one (default runtime image is all-in-one).
#   - Use --target to build a specific stage, otherwise the final default stage
#     in Dockerfile is used.
# ============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log()      { echo -e "${BLUE}ℹ️  $*${NC}"; }
ok()       { echo -e "${GREEN}✅ $*${NC}"; }
warn()     { echo -e "${YELLOW}⚠️  $*${NC}"; }
err()      { echo -e "${RED}❌ $*${NC}" 1>&2; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Defaults
VERSION="latest"
VERSION_SET="false"   # track if user explicitly set --version
ORG=""
NAME="robustmq"
REGISTRY="ghcr"       # ghcr | dockerhub
TARGET_STAGE=""        # e.g. mqtt-broker
PLATFORM=""           # e.g. linux/amd64
PUSH_LATEST="false"

usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  --version VERSION         Image version tag (default: latest)
  --org ORG                 Registry org/username (required)
  --name NAME               Image name (default: robustmq)
  --registry REG            ghcr | dockerhub (default: ghcr)
  --target STAGE            Dockerfile target stage (optional)
  --platform PLATFORM       e.g. linux/amd64 (optional)
  --push-latest             Also push :latest tag
  -h, --help                Show this help

Examples:
  $0 --version 0.2.0 --org socutes --registry ghcr
  $0 --version 0.2.0 --org myuser --registry dockerhub --target mqtt-broker
  $0 --version 0.2.0 --org socutes --platform linux/amd64 --push-latest
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version) VERSION="$2"; VERSION_SET="true"; shift 2 ;;
        --org) ORG="$2"; shift 2 ;;
        --name) NAME="$2"; shift 2 ;;
        --registry) REGISTRY="$2"; shift 2 ;;
        --target) TARGET_STAGE="$2"; shift 2 ;;
        --platform) PLATFORM="$2"; shift 2 ;;
        --push-latest) PUSH_LATEST="true"; shift 1 ;;
        -h|--help) usage; exit 0 ;;
        *) err "Unknown argument: $1"; usage; exit 1 ;;
    esac
done

if [[ -z "$ORG" ]]; then
    err "--org is required"
    usage
    exit 1
fi

# Try to auto-detect version from Cargo.toml if user didn't provide one
detect_workspace_version() {
    local cargo_file="$PROJECT_ROOT/Cargo.toml"
    if [[ ! -f "$cargo_file" ]]; then
        return 1
    fi
    
    # Use simpler grep approach instead of complex awk
    local version
    version=$(grep -E '^version\s*=\s*"[^"]+"' "$cargo_file" | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [[ -n "$version" ]]; then
        echo "$version"
        return 0
    fi
    return 1
}

if [[ "$VERSION_SET" == "false" ]]; then
    if auto_v=$(detect_workspace_version); then
        VERSION="$auto_v"
        log "Auto-detected version from Cargo.toml: $VERSION"
    else
        warn "Could not detect version from Cargo.toml; using default: $VERSION"
    fi
fi

case "$REGISTRY" in
    ghcr)
        REG_PREFIX="ghcr.io/${ORG}"
        ;;
    dockerhub)
        REG_PREFIX="${ORG}"
        ;;
    *)
        err "Unsupported registry: $REGISTRY (use ghcr or dockerhub)"
        exit 1
        ;;
esac

IMAGE_LOCAL="${NAME}:${VERSION}"
IMAGE_REMOTE="${REG_PREFIX}/${NAME}:${VERSION}"

log "Project root: $PROJECT_ROOT"
log "Dockerfile:   docker/robustmq/Dockerfile"
log "Registry:     $REGISTRY"
log "Image name:   $IMAGE_REMOTE"
if [[ -n "$TARGET_STAGE" ]]; then log "Target stage:  $TARGET_STAGE"; fi
if [[ -n "$PLATFORM" ]]; then log "Platform:      $PLATFORM"; fi

cd "$PROJECT_ROOT"

# Auto-login to GitHub Container Registry if needed
if [[ "$REGISTRY" == "ghcr" ]]; then
    log "Checking GitHub Container Registry authentication..."
    
    # Check if already logged in
    if ! docker info | grep -q "ghcr.io"; then
        # Check for GITHUB_TOKEN environment variable
        if [ -z "$GITHUB_TOKEN" ]; then
            log_error "GITHUB_TOKEN environment variable is not set"
            log "Please set your GitHub token:"
            log "  export GITHUB_TOKEN=your_github_token"
            log "  # or add it to your ~/.bashrc or ~/.zshrc"
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
        
        log "Logging in to GHCR as $github_user..."
        
        # Login to GHCR
        if echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$github_user" --password-stdin; then
            log "Successfully logged in to GHCR"
        else
            log_error "Failed to login to GHCR"
            log "Please check your GITHUB_TOKEN and try again"
            exit 1
        fi
    else
        log "Already logged in to GHCR"
    fi
fi

# Build step
log "Building image... this could take a few minutes"

BUILD_ARGS=( -f docker/robustmq/Dockerfile -t "$IMAGE_LOCAL" . )
if [[ -n "$TARGET_STAGE" ]]; then
    BUILD_ARGS=( -f docker/robustmq/Dockerfile --target "$TARGET_STAGE" -t "$IMAGE_LOCAL" . )
fi

if [[ -n "$PLATFORM" ]]; then
    # Use buildx for cross-arch builds; load result into local docker
    if ! docker buildx inspect robustx >/dev/null 2>&1; then
        warn "Creating buildx builder 'robustx'"
        docker buildx create --use --name robustx >/dev/null 2>&1 || true
    fi
    DOCKER_BUILDKIT=1 docker buildx build --platform "$PLATFORM" "${BUILD_ARGS[@]::${#BUILD_ARGS[@]}-1}" --load .
else
    DOCKER_BUILDKIT=1 docker build "${BUILD_ARGS[@]}"
fi

ok "Build finished: $IMAGE_LOCAL"

# Tag & push
log "Tagging as $IMAGE_REMOTE"
docker tag "$IMAGE_LOCAL" "$IMAGE_REMOTE"

log "Pushing $IMAGE_REMOTE"
if docker push "$IMAGE_REMOTE"; then
    ok "Pushed $IMAGE_REMOTE"
else
    warn "Failed to push $IMAGE_REMOTE"
    if [[ "$REGISTRY" == "ghcr" ]]; then
        warn "Make sure you are logged in: echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin"
    else
        warn "Run: docker login"
    fi
    exit 1
fi

if [[ "$PUSH_LATEST" == "true" ]]; then
    LATEST_REMOTE="${REG_PREFIX}/${NAME}:latest"
    log "Tagging & pushing latest: $LATEST_REMOTE"
    docker tag "$IMAGE_LOCAL" "$LATEST_REMOTE"
    docker push "$LATEST_REMOTE"
    ok "Pushed $LATEST_REMOTE"
fi

ok "All done."


