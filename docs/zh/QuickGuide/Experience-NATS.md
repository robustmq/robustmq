# 体验 NATS Core

## 前提：启动 Broker

参考 [快速安装](Quick-Install.md) 完成安装，然后启动服务：

```bash
robust-server start
```

RobustMQ 启动后 NATS 默认监听端口 `4222`，无需额外配置。

---

## 安装 NATS CLI

```bash
# macOS
brew install nats-io/nats-tools/nats

# Linux / Windows
# 参考：https://docs.nats.io/using-nats/nats-tools/nats_cli
```

设置连接地址（可选，省去每次输入 `--server`）：

```bash
export NATS_URL=nats://localhost:4222
```

验证连接：

```bash
nats server ping
```

---

## Pub/Sub

最基础的发布订阅模式。

```bash
# 终端 1：订阅
nats sub "hello.>"

# 终端 2：发布
nats pub hello.world "Hello RobustMQ!"
nats pub hello.nats  "NATS is working!"
```

终端 1 会即时收到两条消息。`>` 是通配符，匹配 `hello.` 下的所有层级。

---

## Request-Reply

同步请求响应，适合 RPC 场景。

```bash
# 终端 1：启动服务端，收到请求后自动回复
nats reply orders.query '{"status":"ok"}'

# 终端 2：发起请求，等待回复
nats request orders.query '{"id":"001"}'
```

---

## Queue Group（竞争消费）

多个消费者加入同一个 Queue Group，每条消息只投递给其中一个，实现负载均衡。

```bash
# 终端 1、2、3：启动三个 Worker，加入同一 Queue Group
nats sub orders.created --queue order-processors   # Worker 1
nats sub orders.created --queue order-processors   # Worker 2
nats sub orders.created --queue order-processors   # Worker 3

# 终端 4：发送消息（每条只会被一个 Worker 收到）
nats pub orders.created '{"order_id":"001"}'
nats pub orders.created '{"order_id":"002"}'
nats pub orders.created '{"order_id":"003"}'
```

---

## 带 Header 的消息

```bash
nats pub --header "Content-Type:application/json" \
         --header "X-Trace-ID:abc123" \
         orders.created '{"order_id":"001"}'
```

---

## 下一步

- [NATS Core 完整文档](../nats/NatsCore.md) — 协议细节、通配符、鉴权、TLS 等
- [SDK 接入](../nats/SDK.md) — 各语言客户端库链接
- [体验 mq9](Experience-MQ9.md) — 基于 NATS 的 AI Agent 通信协议
