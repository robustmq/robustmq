# RobustMQ Bench CLI 使用文档

`robust-bench` 是 RobustMQ 的压测命令行工具，当前支持 MQTT 协议的连接、发布、订阅压测。

## 1. 命令结构

```bash
robust-bench mqtt <subcommand> [options]
```

其中 `<subcommand>` 支持：

- `conn`：连接压测
- `pub`：发布压测
- `sub`：订阅压测

## 2. 通用参数

- `--host`：Broker 地址，默认 `127.0.0.1`
- `--port`：Broker 端口，默认 `1883`
- `--count`：客户端数量，默认 `1000`
- `--interval-ms`：客户端启动间隔（毫秒），主要用于 `pub/sub`，默认 `0`
- `--duration-secs`：压测时长（秒），用于 `pub/sub`，默认 `60`
- `--qos`：QoS（`0|1|2`），默认 `0`
- `--username` / `--password`：用户名密码（推荐固定为 `admin` / `robustmq`）
- `--output`：输出格式，`table|json`，默认 `table`

## 3. 快速示例

### 3.1 连接压测

```bash
robust-bench mqtt conn \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 10000 \
  --concurrency 1000 \
  --mode create
```

或持连接模式：

```bash
robust-bench mqtt conn \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 10000 \
  --concurrency 1000 \
  --mode hold \
  --hold-secs 60
```

### 3.2 发布压测

```bash
robust-bench mqtt pub \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 1000 \
  --topic bench/%i \
  --payload-size 256 \
  --message-interval-ms 10 \
  --qos 1 \
  --duration-secs 120
```

### 3.3 订阅压测

```bash
robust-bench mqtt sub \
  --host 127.0.0.1 \
  --port 1883 \
  --username admin \
  --password robustmq \
  --count 1000 \
  --topic bench/# \
  --qos 1 \
  --duration-secs 120
```

## 4. 输出说明

工具包含两层输出：

1. 运行中实时输出（每秒）：
   - 每秒吞吐（ops/s）
   - 当前累计请求
   - success/failed/timeout/received
   - p95/p99 延迟
2. 结束后详细报告：
   - 总吞吐（total/avg/peak）
   - 成功率、错误率、超时率
   - 延迟分位（min/avg/p50/p95/p99/max）
   - 错误分布
   - 每秒时间线

## 5. 建议

- 首次压测先用较小 `count` 验证连通性，再逐步拉高。
- 发布压测建议至少同时启动一个订阅端验证链路完整性。
- 回归对比时建议固定 `count`、`concurrency`（conn）/`interval-ms`（pub/sub）、`duration-secs` 和 `topic` 模板。
