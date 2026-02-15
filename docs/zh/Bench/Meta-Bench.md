# Meta Bench 使用文档

本文聚焦 `robust-bench meta` 的使用方式，当前首个场景为 Meta Service 的 `placement_create_session`。

## 1. 子命令总览

```bash
robust-bench meta placement-create-session ...
```

## 2. placement-create-session

### 用途

压测 Meta Service `CreateSession` 接口的吞吐能力、延迟分布与错误率。

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

## 3. 输出说明

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

## 4. 建议

- 先用小并发验证连通性，再逐步提高并发寻找拐点。
- 固定 `count`、`concurrency`、`timeout-ms` 做横向对比。
- 关注 `p95/p99` 与 `timeout_rate`，优先定位尾延迟和超时问题。
