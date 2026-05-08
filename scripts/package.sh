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

# Package RobustMQ source code (src, config, scripts, Cargo files) into a tar.gz archive,
# then upload it to the remote server.
# Usage: ./scripts/package.sh [output_dir]
# Default output dir is the project root.

REMOTE_HOST="root@117.72.92.117"
REMOTE_DIR="/root/robustmq"

set -euo pipefail
# todo
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
ARCHIVE_NAME="$(basename "$ARCHIVE")"
scp "$ARCHIVE" "${REMOTE_HOST}:${REMOTE_DIR}"
echo "Upload complete: ${REMOTE_HOST}:${REMOTE_DIR}/${ARCHIVE_NAME}"
rm "$ARCHIVE"
echo "Local archive deleted."

LOCAL_BRANCH=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD)
echo "Local branch: ${LOCAL_BRANCH}"

echo "Syncing remote branch ..."
ssh "${REMOTE_HOST}" "
  set -e
  cd ${REMOTE_DIR}
  REMOTE_BRANCH=\$(git rev-parse --abbrev-ref HEAD)
  echo \"Remote branch: \${REMOTE_BRANCH}\"
  if [ \"\${REMOTE_BRANCH}\" != \"${LOCAL_BRANCH}\" ]; then
    echo \"Switching remote branch to ${LOCAL_BRANCH} ...\"
    git fetch origin
    git checkout ${LOCAL_BRANCH} || git checkout -b ${LOCAL_BRANCH} origin/${LOCAL_BRANCH}
  fi
  git pull origin ${LOCAL_BRANCH}
  tar xzf ${ARCHIVE_NAME} 2>/dev/null && rm ${ARCHIVE_NAME}
  git add -A
  git diff --cached --quiet || git commit -m 'dev'
  git push origin ${LOCAL_BRANCH}
  echo \"Done.\"
"
echo "Remote extraction complete."
