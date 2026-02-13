# MQTT Bench Guide

This document focuses on `robust-bench mqtt` usage and benchmark practices.

## 1. Subcommands

```bash
robust-bench mqtt conn ...
robust-bench mqtt pub ...
robust-bench mqtt sub ...
```

## 2. `conn`: Connection Benchmark

### Purpose

Measure connection establishment capability and stable connection scale.

### Example

```bash
robust-bench mqtt conn \
  --host 10.0.0.10 \
  --port 1883 \
  --count 50000 \
  --interval-ms 1 \
  --duration-secs 90
```

## 3. `pub`: Publish Benchmark

### Purpose

Measure publish throughput, success rate, and publish latency.

### Key Options

- `--topic`: topic template, supports `%i`
- `--payload-size`: payload size in bytes
- `--message-interval-ms`: per-client publish interval
- `--qos`: QoS level

### Example

```bash
robust-bench mqtt pub \
  --host 10.0.0.10 \
  --port 1883 \
  --count 2000 \
  --topic load/%i \
  --payload-size 512 \
  --message-interval-ms 5 \
  --qos 0 \
  --duration-secs 120
```

## 4. `sub`: Subscribe Benchmark

### Purpose

Measure receive throughput, stability, and subscriber-side timeout behavior.

### Example

```bash
robust-bench mqtt sub \
  --host 10.0.0.10 \
  --port 1883 \
  --count 5000 \
  --topic "load/#" \
  --qos 1 \
  --duration-secs 120
```

## 5. How to Read the Result

- `avg_ops_per_sec`: average throughput
- `peak_ops_per_sec`: peak throughput
- `success_rate(%)`: reliability indicator
- `latency_p95/p99`: tail latency, critical for performance analysis
- `Error Distribution`: first place to locate failure reasons

## 6. Recommended Workflow

1. Run `conn` first to determine connection capacity.
2. Run `pub` + `sub` for data-plane performance.
3. Repeat each scenario three times and use median as baseline.
