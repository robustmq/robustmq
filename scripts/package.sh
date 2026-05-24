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

LOCAL_BRANCH=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD)

# Only pack files that changed (added/modified) relative to origin/<branch>,
# plus any untracked files. This avoids shipping the entire repo on every run.
CHANGED_FILES=$(git -C "$PROJECT_ROOT" diff --name-only --diff-filter=ACM "origin/${LOCAL_BRANCH}" 2>/dev/null || true)
UNTRACKED_FILES=$(git -C "$PROJECT_ROOT" ls-files --others --exclude-standard -- src/ Cargo.toml Cargo.lock config/ scripts/ docs/ 2>/dev/null || true)
DELETED_FILES=$(git -C "$PROJECT_ROOT" diff --name-only --diff-filter=D "origin/${LOCAL_BRANCH}" 2>/dev/null || true)

# Combine changed + untracked, exclude .tar.gz files
ALL_FILES=$(printf '%s\n%s' "$CHANGED_FILES" "$UNTRACKED_FILES" | grep -v '\.tar\.gz$' | grep -v '^$' | sort -u)

if [ -z "$ALL_FILES" ]; then
  echo "No changed files to package."
  SKIP_ARCHIVE=1
else
  SKIP_ARCHIVE=0
  printf '%s\0' $ALL_FILES \
    | COPYFILE_DISABLE=1 tar czf "$ARCHIVE" -C "$PROJECT_ROOT" --null -T -
  echo "Packaged: $ARCHIVE ($(du -sh "$ARCHIVE" | cut -f1)) — $(echo "$ALL_FILES" | wc -l | tr -d ' ') files"
fi

echo "Local branch: ${LOCAL_BRANCH}"

ARCHIVE_NAME="$(basename "$ARCHIVE")"
if [ "${SKIP_ARCHIVE}" -eq 0 ]; then
  echo "Uploading to ${REMOTE_HOST}:${REMOTE_DIR} ..."
  scp "$ARCHIVE" "${REMOTE_HOST}:${REMOTE_DIR}"
  echo "Upload complete: ${REMOTE_HOST}:${REMOTE_DIR}/${ARCHIVE_NAME}"
  rm "$ARCHIVE"
  echo "Local archive deleted."
fi

# Build a remote delete command for each locally-deleted file.
REMOTE_DELETE_CMDS=""
if [ -n "$DELETED_FILES" ]; then
  echo "Files deleted locally (will remove on remote):"
  while IFS= read -r f; do
    echo "  - $f"
    REMOTE_DELETE_CMDS="${REMOTE_DELETE_CMDS}  rm -f \"${REMOTE_DIR}/${f}\" && echo \"Deleted: ${f}\" || true"$'\n'
  done <<< "$DELETED_FILES"
fi

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
  if [ -f "${ARCHIVE_NAME}" ]; then
    tar xzf ${ARCHIVE_NAME} --warning=no-unknown-keyword && rm ${ARCHIVE_NAME}
  fi
  # Remove any stale .tar.gz files from the repo root
  find ${REMOTE_DIR} -maxdepth 1 -name '*.tar.gz' -delete
${REMOTE_DELETE_CMDS}
  git add -A
  git diff --cached --quiet || git commit -m 'dev'
  PUSH_RETRY=0
  until git push origin ${LOCAL_BRANCH}; do
    PUSH_RETRY=\$((PUSH_RETRY + 1))
    echo \"Push failed, retrying (\${PUSH_RETRY})...\"
    sleep 3
  done
  echo \"Push succeeded after \${PUSH_RETRY} retries.\"
  echo \"Done.\"
"
echo "Remote extraction complete."

# Clean up any leftover .tar.gz files in the local project root
find "$PROJECT_ROOT" -maxdepth 1 -name '*.tar.gz' -delete
echo "Local .tar.gz files cleaned up."
