# Benchmark Report

This document records benchmark results for RobustMQ components, for regression comparison and performance tracking.

---

## 1. Meta Service

### 1.1 Single-Node Raft Write (placement-create-session)

#### Environment

| Item | Value |
|------|-------|
| Machine | MacBook Pro (Apple Silicon) |
| Deployment | Single node (Meta + Broker in one process) |
| Build | debug (`cargo run`) |
| Storage Engine | RocksDB |

#### Command

```bash
cargo run --package cmd --bin cli-bench meta placement-create-session \
  --host 127.0.0.1 \
  --port 1228 \
  --count 1000000 \
  --concurrency 5000 \
  --timeout-ms 3000 \
  --output table
```

#### Key Metrics

| Metric | Value |
|--------|-------|
| Total Requests | 1,000,000 |
| Success Rate | 100% (0 failed, 0 timeout) |
| Duration | 59 seconds |
| Avg QPS | 16,949 |
| Peak QPS | 22,879 |
| Avg Latency | 288.5 ms |
| P50 | 276.5 ms |
| P95 | 396.5 ms |
| P99 | 565.8 ms |
| Max Latency | 923.1 ms |

#### Latency Distribution

- P95 / P50 = 1.43x — compact distribution
- P99 / P50 = 2.05x — no severe tail latency
- Max / P99 = 1.63x — no extreme outliers

#### Throughput Timeline

```
sec | ops/s  | total     | success   | failed | timeout
  1 | 19157  |    19157  |    19157  |      0 |       0
  2 | 18519  |    37676  |    37676  |      0 |       0
  3 | 20657  |    58333  |    58333  |      0 |       0
  4 | 22879  |    81212  |    81212  |      0 |       0
  5 | 10047  |    91259  |    91259  |      0 |       0
  6 | 14954  |   106213  |   106213  |      0 |       0
  7 |  5296  |   111509  |   111509  |      0 |       0
  8 | 14414  |   125923  |   125923  |      0 |       0
  9 | 14895  |   140818  |   140818  |      0 |       0
 10 | 19355  |   160173  |   160173  |      0 |       0
 11 | 14659  |   174832  |   174832  |      0 |       0
 12 | 19602  |   194434  |   194434  |      0 |       0
 13 | 19761  |   214195  |   214195  |      0 |       0
 14 | 19482  |   233677  |   233677  |      0 |       0
 15 | 19802  |   253479  |   253479  |      0 |       0
 16 | 19832  |   273311  |   273311  |      0 |       0
 17 | 19573  |   292884  |   292884  |      0 |       0
 18 | 19678  |   312562  |   312562  |      0 |       0
 19 | 19729  |   332291  |   332291  |      0 |       0
 20 | 20098  |   352389  |   352389  |      0 |       0
 21 | 19679  |   372068  |   372068  |      0 |       0
 22 | 19668  |   391736  |   391736  |      0 |       0
 23 | 20019  |   411755  |   411755  |      0 |       0
 24 | 20032  |   431787  |   431787  |      0 |       0
 25 | 14735  |   446522  |   446522  |      0 |       0
 26 | 19887  |   466409  |   466409  |      0 |       0
 27 | 20070  |   486479  |   486479  |      0 |       0
 28 | 19740  |   506219  |   506219  |      0 |       0
 29 | 19685  |   525904  |   525904  |      0 |       0
 30 | 16554  |   542458  |   542458  |      0 |       0
 31 | 18112  |   560570  |   560570  |      0 |       0
 32 | 19745  |   580315  |   580315  |      0 |       0
 33 | 15044  |   595359  |   595359  |      0 |       0
 34 | 19833  |   615192  |   615192  |      0 |       0
 35 | 15224  |   630416  |   630416  |      0 |       0
 36 | 20189  |   650605  |   650605  |      0 |       0
 37 |  9735  |   660340  |   660340  |      0 |       0
 38 | 14566  |   674906  |   674906  |      0 |       0
 39 | 15277  |   690183  |   690183  |      0 |       0
 40 | 10731  |   700914  |   700914  |      0 |       0
 41 | 18800  |   719714  |   719714  |      0 |       0
 42 | 14732  |   734446  |   734446  |      0 |       0
 43 | 19721  |   754167  |   754167  |      0 |       0
 44 | 14581  |   768748  |   768748  |      0 |       0
 45 | 15069  |   783817  |   783817  |      0 |       0
 46 | 14746  |   798563  |   798563  |      0 |       0
 47 | 18174  |   816737  |   816737  |      0 |       0
 48 | 16045  |   832782  |   832782  |      0 |       0
 49 | 14892  |   847674  |   847674  |      0 |       0
 50 | 15022  |   862696  |   862696  |      0 |       0
 51 | 14609  |   877305  |   877305  |      0 |       0
 52 | 14695  |   892000  |   892000  |      0 |       0
 53 | 19546  |   911546  |   911546  |      0 |       0
 54 | 15179  |   926725  |   926725  |      0 |       0
 55 |  9668  |   936393  |   936393  |      0 |       0
 56 | 14922  |   951315  |   951315  |      0 |       0
 57 | 14696  |   966011  |   966011  |      0 |       0
 58 | 19623  |   985634  |   985634  |      0 |       0
 59 | 14366  | 1000000   | 1000000  |      0 |       0
```

#### Analysis

The timeline shows clear **periodic fluctuations**:

- **Peak**: 19k-22k ops/s (sec 1-4, 10, 12-24, 26-29, 32, 34, 36, 43, 53, 58)
- **Trough**: 5k-10k ops/s (sec 5, 7, 37, 40, 55)
- **Mid-range**: 14k-16k ops/s (remaining seconds)

This sawtooth pattern is caused by **Raft log persistence + RocksDB compaction**. When RocksDB runs background compaction, it briefly contends for disk I/O, slowing Raft apply and dropping QPS to 5k-10k. Once compaction finishes, QPS recovers to 19k+.

With 5000 concurrency and 288ms average latency, theoretical throughput ceiling = 5000 / 0.288 ≈ 17,361 ops/s, closely matching the actual 16,949. This indicates concurrency is the limiting factor, not the server.

> **Note**: This was built in debug mode (`cargo run`). Release mode typically improves performance by 2-5x.

### 1.2 Single-Node Raft Read (placement-list-session)

#### Environment

Same as 1.1 (run immediately after the write benchmark, with 1M sessions already in RocksDB).

#### Command

```bash
cargo run --package cmd --bin cli-bench meta placement-list-session \
  --host 127.0.0.1 \
  --port 1228 \
  --count 1000000 \
  --concurrency 5000 \
  --timeout-ms 3000 \
  --output table
```

> The bench automatically creates 1 session during setup, then issues 1M `ListSession` requests against the same `client_id`.

#### Key Metrics

| Metric | Value |
|--------|-------|
| Total Requests | 1,000,000 |
| Success Rate | 100% (0 failed, 0 timeout) |
| Records Returned | 1,000,000 (1 per request) |
| Duration | 11 seconds |
| Avg QPS | 90,909 |
| Peak QPS | 102,714 |
| Avg Latency | 45.6 ms |
| P50 | 43.0 ms |
| P95 | 82.3 ms |
| P99 | 102.3 ms |
| Max Latency | 198.7 ms |

#### Latency Distribution

- P95 / P50 = 1.91x — healthy distribution
- P99 / P50 = 2.38x — no severe tail latency
- Max / P99 = 1.94x — no extreme outliers

#### Throughput Timeline

```
sec | ops/s   | total     | success   | failed | timeout | received
  1 |  92529  |    92529  |    92529  |      0 |       0 |    92529
  2 | 100804  |   193333  |   193333  |      0 |       0 |   193333
  3 | 102714  |   296047  |   296047  |      0 |       0 |   296047
  4 |  95409  |   391456  |   391456  |      0 |       0 |   391456
  5 |  92831  |   484287  |   484287  |      0 |       0 |   484287
  6 |  89426  |   573713  |   573713  |      0 |       0 |   573713
  7 |  93254  |   666967  |   666967  |      0 |       0 |   666967
  8 |  90311  |   757278  |   757278  |      0 |       0 |   757278
  9 |  93433  |   850711  |   850711  |      0 |       0 |   850711
 10 |  99942  |   950653  |   950653  |      0 |       0 |   950653
 11 |  49347  |  1000000  |  1000000  |      0 |       0 |  1000000
```

#### Analysis

Read throughput is remarkably stable, sustaining **89k-103k ops/s** throughout, without the sawtooth fluctuations seen in the write benchmark. This is because:

- **Reads bypass Raft consensus**: `ListSession` reads directly from RocksDB, with no log replication or majority confirmation.
- **No WAL write overhead**: Read operations do not trigger RocksDB Write-Ahead Log writes and are unaffected by compaction.
- **Significantly lower latency**: Average 45.6ms vs 288.5ms for writes — read latency is only **1/6** of write latency.

#### Read vs Write Comparison

| Metric | Write (create-session) | Read (list-session) | Read/Write Ratio |
|--------|----------------------|---------------------|-----------------|
| Avg QPS | 16,949 | 90,909 | **5.4x** |
| Peak QPS | 22,879 | 102,714 | **4.5x** |
| Avg Latency | 288.5 ms | 45.6 ms | **6.3x faster** |
| P99 Latency | 565.8 ms | 102.3 ms | **5.5x faster** |
| Throughput Stability | Volatile (5k-22k) | Stable (89k-103k) | — |

> **Note**: This was built in debug mode (`cargo run`). Release mode typically improves performance by 2-5x.

---

## 2. MQTT Broker

> To be added.
