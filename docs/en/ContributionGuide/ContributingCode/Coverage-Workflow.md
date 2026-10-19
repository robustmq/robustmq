# Unit Coverage Workflow

The `Unit Coverage` GitHub Actions workflow publishes report-only coverage for
the repository's existing unit-test scope. It is separate from the normal unit,
MQTT integration, and code-style workflows, and it does not enforce a coverage
percentage threshold.

## Scope

The workflow runs:

```text
make coverage-unit
```

This is the same scope as `make test`: `robustmq-test` and `grpc-clients` are
excluded, and the `storage-adapter` `meta` test is filtered out. The workflow
checks out the exact commit under test and records the Rust, LLVM, coverage-tool,
and nextest versions.

Missing or invalid profiles, test failures, and report-generation failures fail
the workflow. A lower coverage percentage does not fail the workflow during
baseline collection.

## Artifacts

Every run attempts to publish these artifacts, including failed runs:

| Artifact | Contents |
| --- | --- |
| `coverage-unit-reports` | Text, LCOV, JSON, coverage metadata, toolchain metadata, and the exact test command |
| `coverage-unit-html` | Interactive HTML report |
| `coverage-unit-diagnostics` | Runner log and workflow diagnostics |

The instrumented Cargo target directory is not uploaded because it is large and
is only an intermediate build output.

## Actions summary

The job summary is intentionally report-only. It states the exact unit-test
scope, commit, runner outcome, test counts, profile count, and tool versions. If
the text report contains a `TOTAL` line, that line is shown under **Coverage
values**. The summary also names all downloadable artifacts so a maintainer can
inspect the complete HTML, LCOV, JSON, and diagnostic evidence.

This contract is the unit-coverage baseline for later MQTT smoke and integration
coverage workflows. Those workflows should use separate artifact names and
separate scope labels rather than combining unrelated coverage values.
