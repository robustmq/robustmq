# Meta Bench Guide

This document focuses on `robust-bench meta`. Two built-in scenarios target Meta Service: `placement_create_session` (write) and `placement_list_session` (read).

## 1. Subcommands

```bash
robust-bench meta placement-create-session ...
robust-bench meta placement-list-session ...
```

## 2. placement-create-session

### Purpose

Benchmark Meta Service `CreateSession` write throughput, latency distribution, and error rate.

### Key Options

- `--host`: Meta service host, default `127.0.0.1`
- `--port`: Meta service port, default `1228`
- `--count`: total number of requests
- `--concurrency`: concurrent request count
- `--timeout-ms`: per-request timeout in milliseconds
- `--session-expiry-secs`: `session_expiry_interval` for generated sessions
- `--client-id-prefix`: `client_id` prefix for generated requests
- `--output`: `table|json`

### Example

```bash
robust-bench meta placement-create-session \
  --host 127.0.0.1 \
  --port 1228 \
  --count 100000 \
  --concurrency 1000 \
  --timeout-ms 3000 \
  --output table
```

## 3. placement-list-session

### Purpose

Benchmark Meta Service `ListSession` read throughput, latency distribution, and error rate. Each request queries sessions by `client_id`, making it ideal for evaluating metadata read performance.

### Key Options

- `--host`: Meta service host, default `127.0.0.1`
- `--port`: Meta service port, default `1228`
- `--count`: total number of requests
- `--concurrency`: concurrent request count
- `--timeout-ms`: per-request timeout in milliseconds
- `--client-id-prefix`: `client_id` prefix for queries (must match the write phase)
- `--output`: `table|json`

### Example

```bash
robust-bench meta placement-list-session \
  --host 127.0.0.1 \
  --port 1228 \
  --count 100000 \
  --concurrency 1000 \
  --timeout-ms 3000 \
  --output table
```

> **Note**: The bench automatically creates 1 session as setup data, then issues N `ListSession` requests against the same `client_id` to purely measure read throughput. The `received` field in the report indicates the total number of session records returned.

## 4. Output

Real-time output (every second):

- `ops/s`
- `total`
- `success/failed/timeout`
- `p95/p99 latency`

Final summary:

- `avg_ops_per_sec`, `peak_ops_per_sec`
- `success_rate/error_rate/timeout_rate`
- `latency min/avg/p50/p95/p99/max`
- `Error Distribution`

## 5. Recommendations

- Start with low concurrency to validate connectivity, then increase step by step.
- Keep `count`, `concurrency`, and `timeout-ms` fixed for fair comparison.
- Focus on `p95/p99` and `timeout_rate` when identifying bottlenecks.
- Read vs Write: run `placement-create-session` first, then `placement-list-session` to compare write and read throughput and latency.
