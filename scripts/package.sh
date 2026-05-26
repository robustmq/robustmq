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

# Collect all files that differ from origin/<branch> — covering three cases:
#   1. Committed locally but not yet pushed  (HEAD vs origin)
#   2. Modified in working tree but not committed  (working tree vs HEAD)
#   3. Untracked new files
COMMITTED_FILES=$(git -C "$PROJECT_ROOT" diff --name-only --diff-filter=ACM "origin/${LOCAL_BRANCH}" HEAD 2>/dev/null || true)
WORKDIR_FILES=$(git -C "$PROJECT_ROOT" diff --name-only --diff-filter=ACM HEAD 2>/dev/null || true)
UNTRACKED_FILES=$(git -C "$PROJECT_ROOT" ls-files --others --exclude-standard -- src/ Cargo.toml Cargo.lock config/ scripts/ docs/ bin/ 2>/dev/null || true)

# DELETED_FILES: only files that truly no longer exist on disk but are on origin.
DELETED_FILES=""
while IFS= read -r f; do
  [ -n "$f" ] && [ ! -e "$PROJECT_ROOT/$f" ] && DELETED_FILES="${DELETED_FILES}${f}"$'\n'
done < <(git -C "$PROJECT_ROOT" diff --name-only --diff-filter=D "origin/${LOCAL_BRANCH}" HEAD 2>/dev/null || true)
DELETED_FILES="${DELETED_FILES%$'\n'}"

# Combine all three sources, deduplicate, exclude .tar.gz
ALL_FILES=$(printf '%s\n%s\n%s' "$COMMITTED_FILES" "$WORKDIR_FILES" "$UNTRACKED_FILES" \
  | grep -v '\.tar\.gz$' | grep -v '^$' | sort -u)

# Debug: show what each source found
echo "--- File sources ---"
echo "[committed vs origin] $(echo "$COMMITTED_FILES" | grep -c . || echo 0) files"
echo "$COMMITTED_FILES" | grep -v '^$' | sed 's/^/  + /' || true
echo "[workdir vs HEAD]     $(echo "$WORKDIR_FILES" | grep -c . || echo 0) files"
echo "$WORKDIR_FILES" | grep -v '^$' | sed 's/^/  ~ /' || true
echo "[untracked]           $(echo "$UNTRACKED_FILES" | grep -c . || echo 0) files"
echo "$UNTRACKED_FILES" | grep -v '^$' | sed 's/^/  ? /' || true
echo "--------------------"

if [ -z "$ALL_FILES" ]; then
  echo "No changed files to package."
  SKIP_ARCHIVE=1
else
  SKIP_ARCHIVE=0
  echo "Packaging $(echo "$ALL_FILES" | wc -l | tr -d ' ') files:"
  echo "$ALL_FILES" | sed 's/^/  /'
  echo "$ALL_FILES" | tr '\n' '\0' \
    | COPYFILE_DISABLE=1 tar czf "$ARCHIVE" -C "$PROJECT_ROOT" --null -T -
  echo "Packaged: $ARCHIVE ($(du -sh "$ARCHIVE" | cut -f1))"
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
  if [ -f \"${ARCHIVE_NAME}\" ]; then
    tar xzf ${ARCHIVE_NAME} --warning=no-unknown-keyword && rm ${ARCHIVE_NAME}
  fi
  # Remove any stale .tar.gz files from the repo root
  find ${REMOTE_DIR} -maxdepth 1 -name '*.tar.gz' -delete
${REMOTE_DELETE_CMDS}
  git add -A
  git diff --cached --quiet || git commit -m 'dev'
  PUSH_RETRY=0
  MAX_PUSH_RETRIES=3
  until git push origin ${LOCAL_BRANCH} 2>&1 | tee /tmp/push_output.txt; do
    PUSH_OUTPUT=\$(cat /tmp/push_output.txt)
    # Abort immediately on auth/permission errors — retrying won't help
    if echo \"\${PUSH_OUTPUT}\" | grep -qiE 'refusing|403|permission|scope|authentication|not allowed'; then
      echo \"Push permanently rejected (permission/auth error), aborting.\"
      cat /tmp/push_output.txt
      exit 1
    fi
    PUSH_RETRY=\$((PUSH_RETRY + 1))
    if [ \${PUSH_RETRY} -ge \${MAX_PUSH_RETRIES} ]; then
      echo \"Push failed after \${MAX_PUSH_RETRIES} retries, giving up.\"
      exit 1
    fi
    echo \"Push failed, retrying (\${PUSH_RETRY}/\${MAX_PUSH_RETRIES})...\"
    sleep 3
  done
  echo \"Push succeeded after \${PUSH_RETRY} retries.\"
  echo \"Done.\"
"
echo "Remote extraction complete."

# Clean up any leftover .tar.gz files in the local project root
find "$PROJECT_ROOT" -maxdepth 1 -name '*.tar.gz' -delete
echo "Local .tar.gz files cleaned up."

# Local commit: only stage the files that were successfully packaged,
# so unpackaged files remain visible as unstaged for easy comparison.
if [ "${SKIP_ARCHIVE}" -eq 0 ] && [ -n "$ALL_FILES" ]; then
  echo "Committing packaged files locally..."
  while IFS= read -r f; do
    [ -n "$f" ] && git -C "$PROJECT_ROOT" add "$f" 2>/dev/null || true
  done <<< "$ALL_FILES"
  # Also stage any explicitly deleted files
  if [ -n "$DELETED_FILES" ]; then
    while IFS= read -r f; do
      [ -n "$f" ] && git -C "$PROJECT_ROOT" rm --cached "$f" 2>/dev/null || true
    done <<< "$DELETED_FILES"
  fi
  git -C "$PROJECT_ROOT" diff --cached --quiet || \
    git -C "$PROJECT_ROOT" commit -m 'dev'
  echo "Local commit done. Unpackaged files remain unstaged."
fi
