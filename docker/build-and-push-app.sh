#!/usr/bin/env bash
# ============================================================================
# RobustMQ Application Image - Build & Push Helper
# ============================================================================
#
# Purpose:
#   Build the RobustMQ application Docker image (docker/Dockerfile) and push
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
    # Extract version from [workspace.package]
    local version_line
    version_line=$(awk '
        BEGIN { in_section=0 }
        /^\[workspace\.package\]/ { in_section=1; next }
        /^\[/ { if (in_section==1) exit; in_section=0 }
        in_section==1 && /^version\s*=\s*"[^"]+"/ { print; exit }
    ' "$cargo_file") || true

    if [[ -n "$version_line" ]]; then
        echo "$version_line" | sed -E 's/.*"([^"]+)".*/\1/'
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
log "Dockerfile:   docker/Dockerfile"
log "Registry:     $REGISTRY"
log "Image name:   $IMAGE_REMOTE"
if [[ -n "$TARGET_STAGE" ]]; then log "Target stage:  $TARGET_STAGE"; fi
if [[ -n "$PLATFORM" ]]; then log "Platform:      $PLATFORM"; fi

cd "$PROJECT_ROOT"

# Build step
log "Building image... this could take a few minutes"

BUILD_ARGS=( -f docker/Dockerfile -t "$IMAGE_LOCAL" . )
if [[ -n "$TARGET_STAGE" ]]; then
    BUILD_ARGS=( -f docker/Dockerfile --target "$TARGET_STAGE" -t "$IMAGE_LOCAL" . )
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


