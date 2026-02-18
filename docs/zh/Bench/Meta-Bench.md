# Meta Bench 使用文档

本文聚焦 `robust-bench meta` 的使用方式，当前支持 Meta Service 的 `placement_create_session`、`placement_list_session` 和 `placement_delete_session` 三个场景。

## 1. 子命令总览

```bash
robust-bench meta placement-create-session ...
robust-bench meta placement-list-session ...
robust-bench meta placement-delete-session ...
```

## 2. placement-create-session

### 用途

压测 Meta Service `CreateSession` 接口的写入吞吐能力、延迟分布与错误率。

### 常用参数

- `--host`：Meta 服务地址，默认 `127.0.0.1`
- `--port`：Meta 服务端口，默认 `1228`
- `--count`：总请求数
- `--concurrency`：并发请求数
- `--timeout-ms`：单请求超时毫秒数
- `--session-expiry-secs`：构造会话时的 `session_expiry_interval`
- `--client-id-prefix`：压测请求 `client_id` 前缀
- `--output`：`table|json`

### 示例

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

### 用途

压测 Meta Service `ListSession` 接口的读取吞吐能力、延迟分布与错误率。每次请求通过 `client_id` 查询对应 Session，适用于评估元数据读取性能。

### 常用参数

- `--host`：Meta 服务地址，默认 `127.0.0.1`
- `--port`：Meta 服务端口，默认 `1228`
- `--count`：总请求数
- `--concurrency`：并发请求数
- `--timeout-ms`：单请求超时毫秒数
- `--client-id-prefix`：查询的 `client_id` 前缀（需与写入时一致）
- `--output`：`table|json`

### 示例

```bash
robust-bench meta placement-list-session \
  --host 127.0.0.1 \
  --port 1228 \
  --count 100000 \
  --concurrency 1000 \
  --timeout-ms 3000 \
  --output table
```

> **注意**：Bench 启动时会自动创建 1 条 Session 作为 setup 数据，然后对同一个 `client_id` 发起 N 次 `ListSession` 请求，纯测读取吞吐。报告中 `received` 字段表示总共返回的 Session 记录数。

## 4. placement-delete-session

### 用途

压测 Meta Service `DeleteSession` 接口的写入吞吐能力、延迟分布与错误率。适用于评估 Raft 写入路径在删除操作下的极限吞吐，以及分析 MQTT 断连流程中元数据删除的性能瓶颈。

压测时固定使用同一个 `client_id`，首次请求删除已存在的 Session，后续请求对不存在的 Session 发起删除——两种情况均经过完整的 Raft 写入路径，可真实反映 Meta Service 的删除写入能力。

### 常用参数

- `--host`：Meta 服务地址，默认 `127.0.0.1`
- `--port`：Meta 服务端口，默认 `1228`
- `--count`：总请求数
- `--concurrency`：并发请求数
- `--timeout-ms`：单请求超时毫秒数
- `--client-id`：固定使用的 `client_id`，默认 `bench-delete-session-client`
- `--output`：`table|json`

### 示例

```bash
robust-bench meta placement-delete-session \
  --host 127.0.0.1 \
  --port 1228 \
  --count 100000 \
  --concurrency 1000 \
  --timeout-ms 5000 \
  --client-id my-bench-client \
  --output table
```

> **注意**：Bench 启动时会自动创建 1 条 Session 作为 setup 数据，随后所有并发请求均对同一 `client_id` 发起 `DeleteSession`。测量目标是删除操作的 Raft 写入吞吐，而非幂等行为。

## 5. 输出说明

运行中实时输出（每秒）：

- `ops/s`
- `total`
- `success/failed/timeout`
- `p95/p99 latency`

结束后汇总输出：

- `avg_ops_per_sec`、`peak_ops_per_sec`
- `success_rate/error_rate/timeout_rate`
- `latency min/avg/p50/p95/p99/max`
- `Error Distribution`

## 6. 建议

- 先用小并发验证连通性，再逐步提高并发寻找拐点。
- 固定 `count`、`concurrency`、`timeout-ms` 做横向对比。
- 关注 `p95/p99` 与 `timeout_rate`，优先定位尾延迟和超时问题。
- 读写对比：先跑 `placement-create-session` 再跑 `placement-list-session`，对比写入与读取的吞吐和延迟差异。
- 删除写入对比：跑 `placement-delete-session` 与 `placement-create-session` 对比同一 Raft 写入路径在不同操作下的吞吐差异。
