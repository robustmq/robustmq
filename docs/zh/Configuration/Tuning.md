# 性能调优指南

> 本文档面向已完成基本部署的用户，介绍如何通过系统调优和参数调优提升 RobustMQ 在生产环境中的稳定性与吞吐量。

---

## 一、系统级调优

在启动 RobustMQ 之前，建议先完成以下操作系统层面的配置，这些设置对于高并发场景至关重要。

### 1.1 文件描述符限制（ulimit）

**为什么重要**

每个 MQTT 连接、gRPC Channel、RocksDB 文件都会占用一个文件描述符（fd）。系统默认的 `ulimit -n` 通常为 1024，在万级连接场景下会直接触发 `Too many open files` 错误。

**检查当前限制**

```bash
ulimit -n          # 当前 shell 的软限制
cat /proc/sys/fs/file-max   # 系统级最大值
```

**临时调整（当前会话）**

```bash
ulimit -n 1000000
```

**永久生效（推荐）**

编辑 `/etc/security/limits.conf`，添加以下内容：

```
*    soft    nofile    1000000
*    hard    nofile    1000000
root soft    nofile    1000000
root hard    nofile    1000000
```

同时在 `/etc/sysctl.conf` 中设置：

```
fs.file-max = 1000000
```

执行 `sysctl -p` 使其生效。

**RobustMQ 启动脚本自动处理**

`bin/robust-server` 在启动进程前会自动执行 `ulimit -n` 调整，无需任何配置：

1. 优先尝试设置为 `20000000`
2. 若超出系统硬限制，则回退到系统硬限制值
3. 若硬限制也无法获取，则回退到 `1000000`

这一调整仅对当前进程及其子进程生效，不需要 root 权限（只要不超过系统硬限制）。如果启动日志中出现 `⚠️ Could not adjust file descriptor limit`，说明系统硬限制本身过低，需要按上面的方法修改 `/etc/security/limits.conf`。

---

### 1.2 TCP 内核参数

MQTT 的连接模式以长连接为主，内核参数的调优重点是**接受队列深度**和**收发缓冲区**，而非 TIME_WAIT（短连接才严重）。

```bash
# /etc/sysctl.conf

# ---- 连接接受队列 ----
# 每个 listen socket 的最大半连接 + 全连接排队数
# MQTT 客户端重连风暴时，队列太小会丢 SYN 包
net.core.somaxconn = 131072
net.ipv4.tcp_max_syn_backlog = 131072

# ---- 网卡收包队列 ----
# 单核网卡中断处理来不及时的临时缓冲，万兆网卡建议更大
net.core.netdev_max_backlog = 262144

# ---- TCP 收发缓冲区 ----
# MQTT 消息通常较小，但连接数多时总缓冲占用大
# min / default / max（字节）
net.core.rmem_max = 16777216        # socket 级最大读缓冲（16 MB）
net.core.wmem_max = 16777216        # socket 级最大写缓冲（16 MB）
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# ---- TIME_WAIT 管理（主要影响 gRPC 短连接，对 MQTT 长连接影响小）----
net.ipv4.tcp_tw_reuse = 1           # 允许 TIME_WAIT socket 复用（仅出向连接）
net.ipv4.tcp_fin_timeout = 15       # 缩短 FIN_WAIT2 超时（秒）
net.ipv4.tcp_max_tw_buckets = 1000000  # TIME_WAIT 桶上限，超出直接销毁

# ---- 本地端口范围（影响节点间 gRPC 出向连接数）----
net.ipv4.ip_local_port_range = 1024 65535

# ---- 连接跟踪（若开启了 iptables / NAT，需同步调大）----
# net.netfilter.nf_conntrack_max = 2000000
```

执行 `sysctl -p` 使其生效。

**各参数影响说明：**

| 参数 | 场景影响 | 过小的症状 |
|------|---------|-----------|
| `somaxconn` / `tcp_max_syn_backlog` | 客户端重连风暴、启动时大批连接涌入 | `Connection refused` 或连接超时 |
| `netdev_max_backlog` | 万兆网卡高 PPS 场景 | 内核日志出现 `dropped` |
| `tcp_rmem` / `tcp_wmem` | 高吞吐订阅（大 Payload 或高频推送） | 吞吐上不去，CPU 却不高 |
| `tcp_max_tw_buckets` | 节点间 gRPC 频繁重建连接 | 内核日志 `time wait bucket table overflow` |

> **注意：** 若服务器启用了 `iptables` 防火墙或 NAT，还需同时调大 `nf_conntrack_max`，否则连接跟踪表溢出会导致新连接被丢弃（症状与 fd 耗尽类似，但原因不同）。

---

## 二、参数调优

### 2.1 Tokio 运行时线程数

RobustMQ 内部使用三个独立的 Tokio 运行时，对应不同的工作负载：

| 运行时 | 职责 | 默认值 | 调优方向 |
|--------|------|--------|---------|
| `server-runtime` | gRPC、HTTP Admin、Prometheus | `max(4, CPU/2)` | I/O 密集，不需要太多线程 |
| `meta-runtime` | Raft 状态机、RocksDB | `max(4, CPU/2)` | Raft 本质串行，增线程收益有限 |
| `broker-runtime` | MQTT 热路径、消息投递 | `CPU核数` | 最关键，优先保证足够线程 |

**如何判断需要调整**

在 Grafana 中查看 `tokio_runtime_busy_ratio` 指标：

- **< 50%**：线程充裕，无需调整
- **50%~80%**：正常范围
- **> 80%（持续）**：运行时饱和，需增加线程或优化业务逻辑

**配置方法**

```toml
[runtime]
tls_cert = "./config/certs/cert.pem"
tls_key = "./config/certs/key.pem"
# 0 = 自动（推荐），非 0 时手动指定线程数
# server_worker_threads = 0
# meta_worker_threads = 0
# broker_worker_threads = 0
```

**典型配置示例**

| 机器规格 | server | meta | broker |
|---------|--------|------|--------|
| 4 核 | 4（自动） | 4（自动） | 4（自动） |
| 8 核 | 4（自动） | 4（自动） | 8（自动） |
| 16 核 | 8（自动） | 8（自动） | 16（自动） |
| 32 核，broker 负载极高 | 8（自动） | 8（自动） | `broker_worker_threads = 48` |

---

### 2.2 网络处理线程数

```toml
[network]
accept_thread_num = 1     # 接受连接线程，通常 1-4 足够
handler_thread_num = 64   # 请求处理协程数
queue_size = 5000         # 请求队列深度
```

**`handler_thread_num` 调优建议**

每个 handler 协程是一个 Tokio task，空跑时也会占用 `alive_tasks` 计数。默认值 `64` 适合大多数场景：

| 场景 | 建议值 |
|------|--------|
| 低并发测试 | `16` |
| 常规生产（万级连接） | `64` |
| 高并发（十万级连接） | `128` ~ `256` |

> 不要盲目调大 `handler_thread_num`——过大会造成大量空闲 task，增加调度开销并占用 FD。

---

### 2.3 gRPC 客户端连接池

节点间通过 gRPC 通信，每个 Channel 是一条 HTTP/2 TCP 连接：

```toml
[grpc_client]
channels_per_address = 4
```

每条 HTTP/2 连接理论上支持约 200 个并发 Stream，默认 `4` 可支持 ~800 并发 RPC。

| 场景 | 建议值 |
|------|--------|
| 单节点 / 测试 | `2` |
| 常规生产 | `4` |
| 高并发集群 | `8` ~ `16` |

> **注意：** 每增加一个 Channel 会消耗一个 fd。`channels_per_address × 节点数` 需小于 fd 限制的合理范围。

---

### 2.4 Raft 心跳参数

以下参数在代码中存在默认值，`server.toml` 未写出（保持默认即可）。只有在需要调整时才显式写入配置文件：

```toml
[meta_runtime]
heartbeat_timeout_ms = 30000   # 节点心跳超时：超过此时间未收到心跳则判定节点宕机
heartbeat_check_time_ms = 1000 # 心跳检查间隔
raft_write_timeout_sec = 30    # Raft 写超时
```

通常无需修改这些值。Raft 内部心跳间隔（`heartbeat_interval`）目前为 `10ms`，已针对低延迟场景优化。

---

### 2.5 RocksDB 配置

```toml
[rocksdb]
data_path = "./data"
max_open_files = 10000
```

`max_open_files` 是 RocksDB 允许同时打开的 SST 文件数量上限。该值需小于系统 fd 限制：

| 数据量 | 建议值 |
|--------|--------|
| 小（< 10 GB） | `10000` |
| 中（10~100 GB） | `50000` |
| 大（> 100 GB） | `100000`（需相应提升 `ulimit -n`） |

---

### 2.6 消息队列与 QoS 飞行窗口

```toml
[mqtt_protocol_config]
max_qos_flight_message = 2    # 单连接 QoS 1/2 飞行窗口
receive_max = 65535           # 客户端申报的接收窗口
```

`max_qos_flight_message` 控制对单个订阅者同时在途的 QoS 1/2 消息数量：

- **值过小**：消息吞吐受限，高 QPS 场景下订阅者处理速度跟不上
- **值过大**：增加内存压力，慢消费者导致积压

| 场景 | 建议值 |
|------|--------|
| 低频遥测（<10 msg/s/client） | `2` ~ `8` |
| 高频数据流（>1000 msg/s/client） | `64` ~ `256` |

---

## 三、监控驱动调优

调优不应该靠猜测，而应该由指标驱动。以下是关键指标及其含义：

| 指标 | 含义 | 告警阈值 |
|------|------|---------|
| `tokio_runtime_busy_ratio{runtime="broker"}` | broker-runtime 线程繁忙比 | > 80% 持续 5 分钟 |
| `tokio_runtime_queue_depth{runtime="broker"}` | broker-runtime 任务队列积压 | > 1000 |
| `robustmq_process_fd` | 进程当前打开的 FD 数量 | > ulimit 的 80% |
| `robustmq_raft_apply_lag{state_machine="mqtt"}` | Raft 日志应用延迟（待追赶条目数） | > 1000 持续增长 |
| `robustmq_process_cpu_usage` | 进程 CPU 使用率（百分比） | > 85% |
| `robustmq_process_memory` | 进程内存使用量 | 超出机器物理内存 70% |

在 Grafana 中可以通过 **RobustMQ MQTT Broker Dashboard** 实时查看以上指标。详情参考 [Grafana 配置指南](../Observability/Grafana配置指南.md)。

---

## 四、调优 Checklist

部署上线前，建议逐项检查：

- [ ] `ulimit -n` ≥ `1000000`（或按 `max_connection_num × 2` 估算）
- [ ] `/etc/sysctl.conf` 已配置 TCP 相关参数
- [ ] `[runtime]` 三个运行时线程数保持默认 `0`（自动），按需手动覆盖
- [ ] `handler_thread_num` 设置为 `64`（默认），避免空跑 task 过多
- [ ] `channels_per_address` 根据集群规模设置（常规 `4`，高并发 `8~16`）
- [ ] `rocksdb.max_open_files` 小于 `ulimit -n` 的 50%
- [ ] Grafana Dashboard 已配置，可监控 `busy_ratio`、`fd`、`raft_lag` 等核心指标
