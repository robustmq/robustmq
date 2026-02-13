# RobustMQ Bench CLI Guide

`robust-bench` is the benchmark CLI for RobustMQ.  
At this stage, it focuses on MQTT benchmark workloads: connection, publish, and subscribe.

## 1. Command Shape

```bash
robust-bench mqtt <subcommand> [options]
```

Supported `<subcommand>`:

- `conn`: connection benchmark
- `pub`: publish benchmark
- `sub`: subscribe benchmark

## 2. Common Options

- `--host`: broker host, default `127.0.0.1`
- `--port`: broker port, default `1883`
- `--count`: number of clients, default `1000`
- `--interval-ms`: startup interval in milliseconds, default `0`
- `--duration-secs`: benchmark duration in seconds, default `60`
- `--qos`: QoS (`0|1|2`), default `0`
- `--username` / `--password`: optional credentials
- `--output`: `table|json`, default `table`

## 3. Quick Examples

### 3.1 Connection benchmark

```bash
robust-bench mqtt conn \
  --host 127.0.0.1 \
  --port 1883 \
  --count 10000 \
  --interval-ms 1 \
  --duration-secs 60
```

### 3.2 Publish benchmark

```bash
robust-bench mqtt pub \
  --host 127.0.0.1 \
  --port 1883 \
  --count 1000 \
  --topic bench/%i \
  --payload-size 256 \
  --message-interval-ms 10 \
  --qos 1 \
  --duration-secs 120
```

### 3.3 Subscribe benchmark

```bash
robust-bench mqtt sub \
  --host 127.0.0.1 \
  --port 1883 \
  --count 1000 \
  --topic bench/# \
  --qos 1 \
  --duration-secs 120
```

## 4. Output Details

The tool reports in two layers:

1. Real-time output (every second):
   - ops/s
   - cumulative ops
   - success/failed/timeout/received
   - p95/p99 latency
2. Final detailed report:
   - total/avg/peak throughput
   - success/error/timeout rates
   - latency percentiles (min/avg/p50/p95/p99/max)
   - error distribution
   - per-second timeline
