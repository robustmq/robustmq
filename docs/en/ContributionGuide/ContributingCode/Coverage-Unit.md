# Local Unit Coverage

This guide documents the reusable local coverage command from child issue
`#2079`. It produces evidence for the existing unit-test scope; it does not add
a coverage threshold, change production behavior, or replace the correctness
workflow.

## Prerequisites

Use the Rust version pinned in `rust-toolchain.toml` and install the tools proven
by the coverage toolchain task:

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.6.21 --locked
cargo install cargo-nextest --version 0.9.84 --locked
```

The repository's native build dependencies, including CMake, Protobuf, and
OpenSSL development libraries on Linux, are also required by the workspace.

## Run locally

From the repository root, run:

```bash
make coverage-unit
```

The equivalent direct command is:

```bash
./scripts/coverage-unit.sh
```

Use a different output directory when you want to preserve an earlier run:

```bash
./scripts/coverage-unit.sh --output-dir /tmp/robustmq-coverage
```

The runner keeps the existing `make test` scope exactly:

- all workspace tests are selected;
- `robustmq-test` is excluded;
- `grpc-clients` is excluded; and
- the `storage-adapter` `meta` test is filtered out.

It uses `cargo llvm-cov nextest --no-cfg-coverage`, disables incremental compilation, and lets
`cargo-llvm-cov` assign collision-resistant `%p-%10m` profile names for every
test process. Instrumented build output and profiles stay under the selected
output directory, so normal `target/` artifacts are not mixed with coverage
artifacts. Each run uses a clean instrumented target so stale profiles cannot
be mixed into the report.

`--no-cfg-coverage` keeps LLVM instrumentation enabled but does not expose the
`cfg(coverage)` condition to dependencies. This is required on stable Rust
because some dependencies use that condition to enable nightly-only features.

## Output contract

The default directory is `target/coverage/unit`:

| Path | Purpose |
| --- | --- |
| `coverage.txt` | Human-readable coverage report |
| `lcov.info` | LCOV report for later CI or external tooling |
| `coverage.json` | Machine-readable LLVM coverage export |
| `html/index.html` | Browsable HTML report |
| `coverage-summary.json` | Commit, scope, profile count, tool versions, and report paths |
| `summary.txt` | Concise terminal-friendly summary |
| `run.log` | Test and report-generation diagnostics |

Generated reports and instrumented profiles are local artifacts. They are not
committed to the repository. The output directory is ignored through the
existing `target/` rule.

## Validation and failures

The command stops after the test phase when tests fail. It then checks that the
coverage target contains non-empty `.profraw` files and refuses to generate a
report when profile data is missing, empty, or invalid. It also verifies that
the JSON report contains covered source files and that every advertised report
file is non-empty.

The runner records the current commit and exact Rust, Cargo, `cargo-llvm-cov`,
and `cargo-nextest` versions in `coverage-summary.json`. The LCOV and JSON files
can be compared by counting `SF:` records and JSON `data[0].files` entries;
the HTML report can be opened at `html/index.html` to inspect covered and
uncovered source files.

This command only measures in-process unit and library tests. MQTT broker
processes, MQTT integration scenarios, GitHub Actions artifacts, Codecov, and
coverage thresholds belong to later child issues.
