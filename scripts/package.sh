#!/usr/bin/env bash
# Package RobustMQ source code (src, config, scripts, Cargo files) into a tar.gz archive,
# then upload it to the remote server.
# Usage: ./scripts/package.sh [output_dir]
# Default output dir is the project root.

REMOTE_HOST="root@117.72.92.117"
REMOTE_DIR="/root/robustmq-push"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

OUTPUT_DIR="${1:-$PROJECT_ROOT}"
VERSION=$(git -C "$PROJECT_ROOT" describe --tags --always --dirty 2>/dev/null || echo "dev")
TIMESTAMP=$(date +%Y%m%d%H%M%S)
ARCHIVE="$OUTPUT_DIR/robustmq-${VERSION}-${TIMESTAMP}.tar.gz"

git -C "$PROJECT_ROOT" ls-files -- src/ Cargo.toml Cargo.lock config/ scripts/ \
    | tar czf "$ARCHIVE" -C "$PROJECT_ROOT" -T -

echo "Packaged: $ARCHIVE ($(du -sh "$ARCHIVE" | cut -f1))"

echo "Uploading to ${REMOTE_HOST}:${REMOTE_DIR} ..."
scp "$ARCHIVE" "${REMOTE_HOST}:${REMOTE_DIR}"
echo "Upload complete: ${REMOTE_HOST}:${REMOTE_DIR}/$(basename "$ARCHIVE")"
rm "$ARCHIVE"
echo "Local archive deleted."
