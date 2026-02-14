# Meta Bench Guide

This document focuses on `robust-bench meta`. The first built-in scenario targets Meta Service `placement_create_session`.

## 1. Subcommand

```bash
robust-bench meta placement-create-session ...
```

## 2. placement-create-session

### Purpose

Benchmark Meta Service `CreateSession` throughput, latency distribution, and error rate.

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

## 3. Output

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

## 4. Recommendations

- Start with low concurrency to validate connectivity, then increase step by step.
- Keep `count`, `concurrency`, and `timeout-ms` fixed for fair comparison.
- Focus on `p95/p99` and `timeout_rate` when identifying bottlenecks.
