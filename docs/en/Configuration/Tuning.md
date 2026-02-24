# Performance Tuning Guide

> This guide is intended for users who have completed a basic deployment and want to improve RobustMQ's stability and throughput in production environments, through both OS-level and parameter-level tuning.

---

## 1. OS-Level Tuning

Complete the following operating system configuration before starting RobustMQ. These settings are critical for high-concurrency scenarios.

### 1.1 File Descriptor Limits (ulimit)

**Why it matters**

Every MQTT connection, gRPC Channel, and RocksDB file consumes one file descriptor (fd). The system default `ulimit -n` is typically 1024, which will immediately trigger `Too many open files` errors in tens-of-thousands-of-connections scenarios.

**Check current limits**

```bash
ulimit -n                        # Soft limit for the current shell
cat /proc/sys/fs/file-max        # System-level maximum
```

**Temporary adjustment (current session)**

```bash
ulimit -n 1000000
```

**Permanent configuration (recommended)**

Edit `/etc/security/limits.conf` and add:

```
*    soft    nofile    1000000
*    hard    nofile    1000000
root soft    nofile    1000000
root hard    nofile    1000000
```

Also set in `/etc/sysctl.conf`:

```
fs.file-max = 1000000
```

Run `sysctl -p` to apply.

**Automatic handling by the RobustMQ startup script**

`bin/robust-server` automatically runs `ulimit -n` before starting the process — no configuration needed:

1. First attempts to set `20000000`
2. If that exceeds the system hard limit, falls back to the hard limit value
3. If the hard limit cannot be determined, falls back to `1000000`

This adjustment only affects the current process and its children. No root privilege is required as long as the value does not exceed the system hard limit. If the startup log shows `⚠️ Could not adjust file descriptor limit`, the system hard limit itself is too low and needs to be raised via `/etc/security/limits.conf` as described above.

---

### 1.2 TCP Kernel Parameters

MQTT workloads are dominated by long-lived persistent connections. The tuning focus is on **accept queue depth** and **socket buffer sizes**, rather than TIME_WAIT (which primarily affects short-lived HTTP connections).

```bash
# /etc/sysctl.conf

# ---- Connection accept queue ----
# Maximum queued (half-open + fully-established) connections per listen socket.
# During client reconnection storms, a small queue causes SYN drops.
net.core.somaxconn = 131072
net.ipv4.tcp_max_syn_backlog = 131072

# ---- NIC receive queue ----
# Temporary buffer when a CPU core cannot process NIC interrupts fast enough.
# Increase further for 10GbE+ cards.
net.core.netdev_max_backlog = 262144

# ---- TCP socket buffers ----
# MQTT messages are small, but total buffer consumption is high with many connections.
# min / default / max (bytes)
net.core.rmem_max = 16777216        # Max per-socket read buffer (16 MB)
net.core.wmem_max = 16777216        # Max per-socket write buffer (16 MB)
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# ---- TIME_WAIT management (mainly affects gRPC short connections, less impact on MQTT) ----
net.ipv4.tcp_tw_reuse = 1           # Allow TIME_WAIT reuse for outbound connections
net.ipv4.tcp_fin_timeout = 15       # Shorten FIN_WAIT2 timeout (seconds)
net.ipv4.tcp_max_tw_buckets = 1000000  # TIME_WAIT bucket limit; excess sockets are destroyed

# ---- Local port range (affects outbound gRPC connection count between nodes) ----
net.ipv4.ip_local_port_range = 1024 65535

# ---- Connection tracking (if iptables/NAT is enabled, raise this too) ----
# net.netfilter.nf_conntrack_max = 2000000
```

Run `sysctl -p` to apply.

**Parameter impact summary:**

| Parameter | Affected Scenario | Symptom When Too Small |
|-----------|------------------|----------------------|
| `somaxconn` / `tcp_max_syn_backlog` | Client reconnection storms, mass connections at startup | `Connection refused` or connection timeouts |
| `netdev_max_backlog` | High PPS on 10GbE+ NICs | `dropped` messages in kernel logs |
| `tcp_rmem` / `tcp_wmem` | High-throughput subscriptions (large payloads or high-frequency pushes) | Throughput plateaus while CPU is low |
| `tcp_max_tw_buckets` | Frequent gRPC connection rebuilds between nodes | `time wait bucket table overflow` in kernel logs |

> **Note:** If `iptables` firewall or NAT is enabled on the server, you must also raise `nf_conntrack_max`. If the connection tracking table overflows, new connections will be silently dropped — the symptoms are similar to fd exhaustion but the root cause is different.

---

## 2. Parameter Tuning

### 2.1 Tokio Runtime Thread Count

RobustMQ uses three independent Tokio runtimes internally, each serving a distinct workload:

| Runtime | Responsibilities | Default | Tuning Direction |
|---------|-----------------|---------|-----------------|
| `server-runtime` | gRPC, HTTP Admin, Prometheus | `max(4, CPU/2)` | I/O-bound; fewer threads are sufficient |
| `meta-runtime` | Raft state machines, RocksDB | `max(4, CPU/2)` | Raft is largely serial; more threads have diminishing returns |
| `broker-runtime` | MQTT hot path, message delivery | `CPU cores` | Most critical; prioritize sufficient threads |

**How to determine if adjustment is needed**

Check the `tokio_runtime_busy_ratio` metric in Grafana:

- **< 50%**: Thread pool has headroom; no adjustment needed
- **50%–80%**: Normal operating range
- **> 80% (sustained)**: Runtime is saturated; increase thread count or optimize business logic

**Configuration**

```toml
[runtime]
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
# 0 = auto (recommended), non-zero overrides manually
# server_worker_threads = 0
# meta_worker_threads = 0
# broker_worker_threads = 0
```

**Typical configurations**

| Machine | server | meta | broker |
|---------|--------|------|--------|
| 4-core | 4 (auto) | 4 (auto) | 4 (auto) |
| 8-core | 4 (auto) | 4 (auto) | 8 (auto) |
| 16-core | 8 (auto) | 8 (auto) | 16 (auto) |
| 32-core, high broker load | 8 (auto) | 8 (auto) | `broker_worker_threads = 48` |

---

### 2.2 Network Handler Thread Count

```toml
[network]
accept_thread_num = 1     # Accept connection threads; 1–4 is usually sufficient
handler_thread_num = 64   # Number of handler coroutines
queue_size = 5000         # Request queue depth
```

**`handler_thread_num` tuning guide**

Each handler is a Tokio task. Even idle tasks contribute to the `alive_tasks` count. The default `64` suits most scenarios:

| Scenario | Recommended Value |
|----------|------------------|
| Low-concurrency testing | `16` |
| Regular production (10k connections) | `64` |
| High concurrency (100k connections) | `128` – `256` |

> Do not blindly increase `handler_thread_num` — too many idle tasks increases scheduling overhead and fd consumption.

---

### 2.3 gRPC Client Connection Pool

Nodes communicate via gRPC, and each Channel is one HTTP/2 TCP connection:

```toml
[grpc_client]
channels_per_address = 4
```

Each HTTP/2 connection theoretically supports ~200 concurrent streams. The default `4` supports ~800 concurrent RPCs.

| Scenario | Recommended Value |
|----------|------------------|
| Single-node / testing | `2` |
| Regular production | `4` |
| High-concurrency cluster | `8` – `16` |

> **Note:** Each Channel consumes one fd. Ensure `channels_per_address × node_count` stays well within the fd limit.

---

### 2.4 Raft Heartbeat Parameters

These parameters exist in code with defaults but are not written to `server.toml` (the defaults apply automatically). Only add them explicitly when you need to override:

```toml
[meta_runtime]
heartbeat_timeout_ms = 30000    # Node heartbeat timeout (time before a node is considered down)
heartbeat_check_time_ms = 1000  # Heartbeat check interval
raft_write_timeout_sec = 30     # Raft write timeout
```

These rarely need adjustment. The internal Raft `heartbeat_interval` is `10ms`, already tuned for low-latency scenarios.

---

### 2.5 RocksDB Configuration

```toml
[rocksdb]
data_path = "./data"
max_open_files = 10000
```

`max_open_files` is the maximum number of SST files RocksDB may keep open simultaneously. It must be less than the system fd limit:

| Data Volume | Recommended Value |
|-------------|------------------|
| Small (< 10 GB) | `10000` |
| Medium (10–100 GB) | `50000` |
| Large (> 100 GB) | `100000` (requires a corresponding `ulimit -n` increase) |

---

### 2.6 Message Queue and QoS In-Flight Window

```toml
[mqtt_protocol_config]
max_qos_flight_message = 2    # Per-connection QoS 1/2 in-flight window
receive_max = 65535           # Receive window advertised to clients
```

`max_qos_flight_message` controls the number of QoS 1/2 messages simultaneously in-flight to a single subscriber:

- **Too small**: Throughput is bottlenecked; slow subscribers in high-QPS scenarios
- **Too large**: Increased memory pressure; slow consumers cause backlogs

| Scenario | Recommended Value |
|----------|------------------|
| Low-frequency telemetry (< 10 msg/s/client) | `2` – `8` |
| High-frequency data streams (> 1000 msg/s/client) | `64` – `256` |

---

## 3. Metrics-Driven Tuning

Tuning should be guided by metrics, not guesswork. Key metrics and their significance:

| Metric | Meaning | Alert Threshold |
|--------|---------|----------------|
| `tokio_runtime_busy_ratio{runtime="broker"}` | broker-runtime thread busy ratio | > 80% for 5 minutes |
| `tokio_runtime_queue_depth{runtime="broker"}` | broker-runtime task queue backlog | > 1000 |
| `robustmq_process_fd` | Number of file descriptors open by the process | > 80% of ulimit |
| `robustmq_raft_apply_lag{state_machine="mqtt"}` | Raft log apply lag (unapplied entries) | > 1000 and growing |
| `robustmq_process_cpu_usage` | Process CPU usage (percent) | > 85% |
| `robustmq_process_memory` | Process memory usage | > 70% of physical RAM |

All of the above metrics are visible in real-time on the **RobustMQ MQTT Broker Dashboard** in Grafana. See the [Grafana Configuration Guide](../Observability/Grafana-Configuration-Guide.md) for details.

---

## 4. Tuning Checklist

Before going to production, verify each item:

- [ ] `ulimit -n` ≥ `1000000` (or estimate as `max_connection_num × 2`)
- [ ] TCP-related kernel parameters configured in `/etc/sysctl.conf`
- [ ] `[runtime]` three runtime thread counts kept at default `0` (auto), with manual overrides only when necessary
- [ ] `handler_thread_num` set to `64` (default) to avoid excessive idle tasks
- [ ] `channels_per_address` set according to cluster scale (regular: `4`, high-concurrency: `8–16`)
- [ ] `rocksdb.max_open_files` < 50% of `ulimit -n`
- [ ] Grafana Dashboard configured to monitor `busy_ratio`, `fd`, `raft_lag`, and other core metrics
