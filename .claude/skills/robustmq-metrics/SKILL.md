---
name: robustmq-metrics
description: Designs and implements minimal, high-value metrics for RobustMQ services and dashboards. Use when the user asks to add metrics, improve observability, or update Grafana panels for core processing pipelines.
---

# RobustMQ Metrics

## Purpose

Add observability with minimal but complete coverage of a target pipeline:

- process count and process duration
- retry and terminal outcomes (if applicable)
- key failure points in read/process/write path
- liveness/health

Do not over-instrument. Prefer compact metrics that support operations and debugging.

## Metric Design Rules

1. **Cover chain, not everything**:
   - success/failure + duration
   - retry/terminal path (when strategy exists)
   - read/commit/write failures
   - up/down

2. **Always include count and latency** for processing path.

3. **Label policy**:
   - required: stable low-cardinality dimensions only
   - optional only when bounded enum (for example `result`, `strategy`, `protocol`)
   - service/entity name labels are optional and must pass cardinality review
   - forbidden high-cardinality labels: `topic`, `error_message`, payload-derived labels

4. **Naming**:
   - use module prefix (for example `mqtt_`, `raft_`, `handler_`)
   - counters end with `_total`
   - duration histogram uses `_ms`
   - liveness gauge uses `_up`

## Minimal Metric Set Template

Adapt this template to the target module:

- `<module>_messages_processed_success_total{...}`
- `<module>_messages_processed_failure_total{...}`
- `<module>_process_duration_ms{...}` (histogram)
- `<module>_retry_total{...,strategy}` (if retry exists)
- `<module>_terminal_total{...,result}` (discard/dlq/drop etc., if applicable)
- `<module>_critical_step_failure_total{...}` (read/write/commit/etc.)
- `<module>_up{...}` (gauge)

## Implementation Workflow

1. **Define metrics in `src/common/metrics`**
   - register counters/histogram/gauge
   - add record helper APIs
   - keep API signatures consistent with chosen low-cardinality labels

2. **Insert runtime instrumentation**
   - add metrics at success, failure, retry, terminal, and liveness points

3. **Fix call sites impacted by signature changes**
   - update all metric helper users
   - ensure compile passes across dependent crates

4. **Validation**
   - run targeted `cargo check` for impacted crates
   - run lints for touched files

## Grafana Decision Rule

After metrics are added, decide whether to update `grafana/robustmq-broker.json`:

- **Update dashboard** if metrics are operationally critical and stable.
- **Skip dashboard** only when user explicitly says not to update or metrics are temporary.

When updating dashboard:

- add/extend compact row
- prioritize low-cardinality dimensions and trend panels
- avoid panel explosion; 4-6 panels for first iteration

## Output Format

Return:

1. Metrics added/changed (names + labels)
2. Instrumentation points (files/functions)
3. Whether Grafana was updated and why
4. Validation commands and results

## Guardrails

- Do not add metrics without clear operational use.
- Do not introduce high-cardinality labels.
- Do not duplicate semantically equivalent metrics.
- Do not break existing metric names unless migration is requested.
