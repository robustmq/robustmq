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

set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${COVERAGE_OUTPUT_DIR:-${PROJECT_ROOT}/target/coverage/unit}"
PACKAGE_SPEC="${COVERAGE_PACKAGES:-${COVERAGE_PACKAGE:-}}"
PACKAGES=()

usage() {
    cat <<'EOF'
Usage: scripts/coverage-unit.sh [--output-dir DIRECTORY] [--package PACKAGE[,PACKAGE...]]

Run the repository's unit-test scope with LLVM coverage instrumentation and
write reports to DIRECTORY. The default is target/coverage/unit.

Without --package, the test scope is the same as `make test`: robustmq-test and
grpc-clients are excluded, and the storage-adapter meta test is filtered out.
With --package, only the selected package(s)' test targets are executed. Use a
comma-separated list for multiple packages, for example
`--package common-metrics,system-info`. Cargo may still compile and instrument
dependency crates; the reports are labeled with the selected scope.
EOF
}

while (($# > 0)); do
    case "$1" in
        --output-dir)
            if (($# < 2)); then
                echo "--output-dir requires a directory" >&2
                exit 2
            fi
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --package|-p)
            if (($# < 2)); then
                echo "--package requires a package name" >&2
                exit 2
            fi
            if [[ -n "$PACKAGE_SPEC" && "$PACKAGE_SPEC" != "$2" ]]; then
                echo "Conflicting package selectors: ${PACKAGE_SPEC} and $2" >&2
                exit 2
            fi
            PACKAGE_SPEC="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ -n "$PACKAGE_SPEC" ]]; then
    if [[ ! "$PACKAGE_SPEC" =~ ^[A-Za-z0-9_-]+(,[A-Za-z0-9_-]+)*$ ]]; then
        echo "Invalid package selector: ${PACKAGE_SPEC}" >&2
        exit 2
    fi
    IFS=',' read -r -a PACKAGES <<< "$PACKAGE_SPEC"
    if ((${#PACKAGES[@]} == 0)); then
        echo "At least one package name is required" >&2
        exit 2
    fi
    for ((package_index = 0; package_index < ${#PACKAGES[@]}; package_index++)); do
        package="${PACKAGES[$package_index]}"
        if [[ ! "$package" =~ ^[A-Za-z0-9_-]+$ ]]; then
            echo "Invalid package name: ${package}" >&2
            exit 2
        fi
        for ((previous_index = 0; previous_index < package_index; previous_index++)); do
            if [[ "${PACKAGES[$previous_index]}" == "$package" ]]; then
                echo "Duplicate package name: ${package}" >&2
                exit 2
            fi
        done
        if [[ "$package" == "robustmq-test" || "$package" == "grpc-clients" ]]; then
            echo "Package ${package} is integration-only and cannot use unit coverage." >&2
            echo "Use the corresponding integration workflow instead." >&2
            exit 2
        fi
    done
    if ! cargo metadata --no-deps --format-version 1 \
        | python3 -c 'import json, sys; selected = set(sys.argv[1].split(",")); metadata = json.load(sys.stdin); names = {item["name"] for item in metadata["packages"]}; missing = selected - names; print("Unknown workspace package(s): " + ", ".join(sorted(missing)), file=sys.stderr) if missing else None; raise SystemExit(bool(missing))' "$PACKAGE_SPEC"; then
        exit 2
    fi
fi

mkdir -p -- "$OUTPUT_DIR"
OUTPUT_DIR="$(cd -- "$OUTPUT_DIR" && pwd)"
case "$OUTPUT_DIR" in
    /|"${PROJECT_ROOT}"|"${PROJECT_ROOT}/")
        echo "Refusing to use a project root or filesystem root as the output directory: ${OUTPUT_DIR}" >&2
        exit 2
        ;;
esac

MARKER_FILE="$OUTPUT_DIR/.robustmq-coverage-output"
if [[ -e "$MARKER_FILE" ]]; then
    if [[ ! -f "$MARKER_FILE" ]] || [[ "$(cat "$MARKER_FILE")" != "RobustMQ local unit coverage output" ]]; then
        echo "Refusing to use output directory with an unrecognized marker: ${OUTPUT_DIR}" >&2
        exit 2
    fi
elif [[ -n "$(find "$OUTPUT_DIR" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
    echo "Output directory is not empty and is not marked as runner-owned: ${OUTPUT_DIR}" >&2
    echo "Choose an empty directory or the runner's existing output directory." >&2
    exit 2
else
    printf '%s\n' "RobustMQ local unit coverage output" > "$MARKER_FILE"
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required" >&2
    exit 1
fi
if ! cargo llvm-cov --version >/dev/null 2>&1; then
    echo "cargo-llvm-cov is required; install version 0.6.21 or a compatible version" >&2
    exit 1
fi
if ! cargo nextest --version >/dev/null 2>&1; then
    echo "cargo-nextest is required; install version 0.9.84 or a compatible version" >&2
    exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required to validate and write the report metadata" >&2
    exit 1
fi

rm -rf "$OUTPUT_DIR/html"
rm -f \
    "$OUTPUT_DIR/coverage.txt" \
    "$OUTPUT_DIR/lcov.info" \
    "$OUTPUT_DIR/coverage.json" \
    "$OUTPUT_DIR/coverage-summary.json" \
    "$OUTPUT_DIR/summary.txt" \
    "$OUTPUT_DIR/run.log"

LOG_FILE="$OUTPUT_DIR/run.log"
exec > >(tee "$LOG_FILE") 2>&1

CARGO_TARGET_DIR="$OUTPUT_DIR/cargo-target"
export CARGO_TARGET_DIR
export CARGO_INCREMENTAL=0
unset RUSTC_WRAPPER

# Cargo refuses to clean a target directory without its cache safety marker.
mkdir -p "$CARGO_TARGET_DIR/llvm-cov-target"
printf '%s\n' 'Signature: 8a477f597d28d172789f06886806ab4' > "$CARGO_TARGET_DIR/llvm-cov-target/CACHEDIR.TAG"

if [[ -n "$PACKAGE_SPEC" ]]; then
    SCOPE_KIND="package"
    SCOPE_LABEL="package:${PACKAGE_SPEC}"
    if ((${#PACKAGES[@]} > 1)); then
        SCOPE_KIND="packages"
        SCOPE_LABEL="packages:${PACKAGE_SPEC}"
    fi
    CARGO_SCOPE_ARGS=()
    for package in "${PACKAGES[@]}"; do
        CARGO_SCOPE_ARGS+=(--package "$package")
    done
    TEST_SCOPE="cargo llvm-cov nextest --no-cfg-coverage ${CARGO_SCOPE_ARGS[*]} --filter-expr !(test(meta) & package(storage-adapter)) --no-fail-fast --no-report"
else
    SCOPE_KIND="workspace"
    SCOPE_LABEL="workspace"
    TEST_SCOPE="cargo llvm-cov nextest --no-cfg-coverage --workspace --exclude=robustmq-test --exclude=grpc-clients --filter-expr !(test(meta) & package(storage-adapter)) --no-fail-fast --no-report"
    CARGO_SCOPE_ARGS=(--workspace --exclude=robustmq-test --exclude=grpc-clients)
fi
TEST_STATUS=0

echo "Running RobustMQ unit coverage"
echo "Project: ${PROJECT_ROOT}"
echo "Output: ${OUTPUT_DIR}"
echo "Scope: ${TEST_SCOPE}"
echo

set +e
cargo llvm-cov nextest \
    --no-cfg-coverage \
    "${CARGO_SCOPE_ARGS[@]}" \
    --filter-expr '!(test(meta) & package(storage-adapter))' \
    --no-fail-fast \
    --no-report
TEST_STATUS=$?
set -e

if ((TEST_STATUS != 0)); then
    echo "Unit tests failed with exit status ${TEST_STATUS}. See ${LOG_FILE}." >&2
    exit "$TEST_STATUS"
fi

PROFILE_ROOT="$CARGO_TARGET_DIR/llvm-cov-target"
if [[ ! -d "$PROFILE_ROOT" ]]; then
    echo "Coverage run produced no profile directory at ${PROFILE_ROOT}." >&2
    echo "The coverage result is invalid; refusing to generate a false 0% report." >&2
    exit 1
fi
PROFILE_COUNT="$(find "$PROFILE_ROOT" -type f -name '*.profraw' -size +0c -print 2>/dev/null | wc -l | tr -d ' ')"
EMPTY_PROFILE_COUNT="$(find "$PROFILE_ROOT" -type f -name '*.profraw' -size 0c -print 2>/dev/null | wc -l | tr -d ' ')"

if [[ "$PROFILE_COUNT" -eq 0 ]]; then
    echo "Coverage run produced no non-empty .profraw files under ${PROFILE_ROOT}." >&2
    echo "The coverage result is invalid; refusing to generate a false 0% report." >&2
    exit 1
fi
if [[ "$EMPTY_PROFILE_COUNT" -ne 0 ]]; then
    echo "Coverage run produced ${EMPTY_PROFILE_COUNT} empty .profraw file(s) under ${PROFILE_ROOT}." >&2
    echo "The coverage result is invalid; refusing to generate a report." >&2
    exit 1
fi

echo "Collected ${PROFILE_COUNT} non-empty LLVM profile file(s)."

echo "Generating text report..."
cargo llvm-cov report --text --output-path "$OUTPUT_DIR/coverage.txt" || {
    echo "Text coverage report failed; profile data may be invalid. See ${LOG_FILE}." >&2
    exit 1
}
echo "Generating LCOV report..."
cargo llvm-cov report --lcov --output-path "$OUTPUT_DIR/lcov.info" || {
    echo "LCOV coverage report failed; profile data may be invalid. See ${LOG_FILE}." >&2
    exit 1
}
echo "Generating JSON report..."
cargo llvm-cov report --json --output-path "$OUTPUT_DIR/coverage.json" || {
    echo "JSON coverage report failed; profile data may be invalid. See ${LOG_FILE}." >&2
    exit 1
}
echo "Generating HTML report..."
cargo llvm-cov report --html --output-dir "$OUTPUT_DIR" || {
    echo "HTML coverage report failed; profile data may be invalid. See ${LOG_FILE}." >&2
    exit 1
}

for report in "$OUTPUT_DIR/coverage.txt" "$OUTPUT_DIR/lcov.info" "$OUTPUT_DIR/coverage.json" "$OUTPUT_DIR/html/index.html"; do
    if [[ ! -s "$report" ]]; then
        echo "Coverage report is missing or empty: ${report}" >&2
        exit 1
    fi
done

COMMIT="$(git -C "$PROJECT_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
RUST_VERSION="$(rustc --version 2>/dev/null || echo unavailable)"
CARGO_VERSION="$(cargo --version 2>/dev/null || echo unavailable)"
LLVM_COV_VERSION="$(cargo llvm-cov --version 2>/dev/null || echo unavailable)"
NEXTEST_VERSION="$(cargo nextest --version 2>/dev/null || echo unavailable)"

TEST_RESULT="$(python3 - "$LOG_FILE" <<'PY'
import pathlib
import re
import sys

log = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
# CARGO_TERM_COLOR=always keeps CI logs readable but adds ANSI formatting.
log = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log)
match = re.search(
    r"Summary .*?\d+ tests run: (\d+) passed(?:, (\d+) failed)?(?:, (\d+) skipped)?",
    log,
)
if match is None:
    raise SystemExit("could not extract the nextest result summary from run.log")
passed, failed, skipped = match.groups()
print(f"{passed} passed, {failed or 0} failed, {skipped or 0} skipped")
PY
)"

python3 - "$OUTPUT_DIR/coverage.json" "$OUTPUT_DIR/coverage-summary.json" "$COMMIT" "$PROFILE_COUNT" "$TEST_SCOPE" "$RUST_VERSION" "$CARGO_VERSION" "$LLVM_COV_VERSION" "$NEXTEST_VERSION" "$OUTPUT_DIR/lcov.info" "$LOG_FILE" "$SCOPE_KIND" "$PACKAGE_SPEC" <<'PY'
import json
import pathlib
import re
import sys

coverage_path = pathlib.Path(sys.argv[1])
summary_path = pathlib.Path(sys.argv[2])

with coverage_path.open(encoding="utf-8") as handle:
    coverage = json.load(handle)

if not coverage.get("data") or not coverage["data"][0].get("files"):
    raise SystemExit("coverage JSON contains no covered source files")

json_sources = {entry["filename"] for entry in coverage["data"][0]["files"]}
lcov_lines = pathlib.Path(sys.argv[10]).read_text(encoding="utf-8").splitlines()
lcov_sources = {line.removeprefix("SF:").strip() for line in lcov_lines if line.startswith("SF:")}
if not lcov_sources:
    raise SystemExit("LCOV report contains no source files")
if json_sources != lcov_sources:
    raise SystemExit(
        f"LCOV/JSON source mismatch: {len(lcov_sources)} LCOV files, "
        f"{len(json_sources)} JSON files"
    )
if not any(
    int(line.split(",", 2)[1]) > 0
    for line in lcov_lines
    if line.startswith("DA:")
):
    raise SystemExit("LCOV report contains no executed source lines")

test_log = pathlib.Path(sys.argv[11]).read_text(encoding="utf-8", errors="replace")
test_log = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", test_log)
test_result = re.search(
    r"Summary .*?\d+ tests run: (\d+) passed(?:, (\d+) failed)?(?:, (\d+) skipped)?",
    test_log,
)
if test_result is None:
    raise SystemExit("could not extract the nextest result summary from run.log")
passed, failed, skipped = test_result.groups()

summary = {
    "schema_version": 1,
    "commit": sys.argv[3],
    "profile_count": int(sys.argv[4]),
    "test_scope": sys.argv[5],
    "scope": {
        "kind": sys.argv[12],
        "package": sys.argv[13] if sys.argv[13] and "," not in sys.argv[13] else None,
        "packages": sys.argv[13].split(",") if sys.argv[13] else [],
    },
    "test_result": {
        "passed": int(passed),
        "failed": int(failed or 0),
        "skipped": int(skipped or 0),
    },
    "toolchain": {
        "rustc": sys.argv[6],
        "cargo": sys.argv[7],
        "cargo_llvm_cov": sys.argv[8],
        "cargo_nextest": sys.argv[9],
    },
    "reports": {
        "text": "coverage.txt",
        "lcov": "lcov.info",
        "json": "coverage.json",
        "html": "html/index.html",
        "log": "run.log",
    },
}

with summary_path.open("w", encoding="utf-8") as handle:
    json.dump(summary, handle, indent=2)
    handle.write("\n")
PY

cat > "$OUTPUT_DIR/summary.txt" <<EOF
RobustMQ unit coverage
======================
Commit: ${COMMIT}
Profiles: ${PROFILE_COUNT}
Tests: ${TEST_RESULT}
Rust: ${RUST_VERSION}
Cargo: ${CARGO_VERSION}
Coverage tool: ${LLVM_COV_VERSION}
Nextest: ${NEXTEST_VERSION}
Scope: ${TEST_SCOPE}
Selection: ${SCOPE_LABEL}

Reports:
  Text: ${OUTPUT_DIR}/coverage.txt
  LCOV: ${OUTPUT_DIR}/lcov.info
  JSON: ${OUTPUT_DIR}/coverage.json
  Metadata: ${OUTPUT_DIR}/coverage-summary.json
  HTML: ${OUTPUT_DIR}/html/index.html
  Log: ${OUTPUT_DIR}/run.log
EOF

echo
echo "Coverage completed successfully."
cat "$OUTPUT_DIR/summary.txt"
