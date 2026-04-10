# 快速开始

本文带你在 5 分钟内连接 RobustMQ NATS，完成发布、订阅和 Request-Reply。

## 前置条件

- RobustMQ 已启动，NATS 端口默认 `4222`
- 安装 [NATS CLI](https://github.com/nats-io/natscli)（可选，用于验证）

## 启动 RobustMQ

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
robust-server start
```

## 验证连接（NATS CLI）

```bash
# 订阅（终端 1）
nats sub "hello.>" --server nats://localhost:4222

# 发布（终端 2）
nats pub "hello.world" "Hello RobustMQ!" --server nats://localhost:4222
```

终端 1 收到消息即表示连接正常。

---

## 各语言快速上手

选择你的语言，5 分钟跑通完整流程。

### Go

```bash
go get github.com/nats-io/nats.go
```

```go
package main

import (
    "fmt"
    "time"
    "github.com/nats-io/nats.go"
)

func main() {
    nc, _ := nats.Connect("nats://localhost:4222")
    defer nc.Close()

    // 订阅
    nc.Subscribe("hello.>", func(m *nats.Msg) {
        fmt.Printf("收到: %s\n", m.Data)
    })

    // 发布
    nc.Publish("hello.world", []byte("Hello RobustMQ!"))

    // Request-Reply
    msg, _ := nc.Request("hello.query", []byte("ping"), 2*time.Second)
    fmt.Printf("回复: %s\n", msg.Data)

    time.Sleep(time.Second)
}
```

→ 更多示例见 [nats.go 官方文档](https://github.com/nats-io/nats.go)

### Python

```bash
pip install nats-py
```

```python
import asyncio
import nats

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # 订阅
    async def handler(msg):
        print(f"收到: {msg.data.decode()}")
    await nc.subscribe("hello.>", cb=handler)

    # 发布
    await nc.publish("hello.world", b"Hello RobustMQ!")

    # Request-Reply
    reply = await nc.request("hello.query", b"ping", timeout=2)
    print(f"回复: {reply.data.decode()}")

    await asyncio.sleep(1)
    await nc.close()

asyncio.run(main())
```

→ 更多示例见 [nats.py 官方文档](https://github.com/nats-io/nats.py)

### JavaScript / Node.js

```bash
npm install nats
```

```javascript
import { connect, StringCodec } from "nats";

const nc = await connect({ servers: "nats://localhost:4222" });
const sc = StringCodec();

// 订阅
const sub = nc.subscribe("hello.>");
(async () => {
    for await (const msg of sub) {
        console.log(`收到: ${sc.decode(msg.data)}`);
    }
})();

// 发布
nc.publish("hello.world", sc.encode("Hello RobustMQ!"));

// Request-Reply
const reply = await nc.request("hello.query", sc.encode("ping"), { timeout: 2000 });
console.log(`回复: ${sc.decode(reply.data)}`);

await nc.close();
```

→ 更多示例见 [nats.js 官方文档](https://github.com/nats-io/nats.js)

### Java

```xml
<!-- Maven -->
<dependency>
    <groupId>io.nats</groupId>
    <artifactId>jnats</artifactId>
    <version>2.20.5</version>
</dependency>
```

```java
Connection nc = Nats.connect("nats://localhost:4222");

// 订阅
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("收到: " + new String(msg.getData()));
});
d.subscribe("hello.>");

// 发布
nc.publish("hello.world", "Hello RobustMQ!".getBytes());

// Request-Reply
Message reply = nc.request("hello.query", "ping".getBytes(), Duration.ofSeconds(2));
System.out.println("回复: " + new String(reply.getData()));

nc.close();
```

→ 更多示例见 [jnats 官方文档](https://github.com/nats-io/jnats)

### Rust

```toml
# Cargo.toml
[dependencies]
async-nats = "0.37"
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    // 订阅
    let mut sub = client.subscribe("hello.>").await?;

    // 发布
    client.publish("hello.world", "Hello RobustMQ!".into()).await?;

    // 接收
    if let Some(msg) = sub.next().await {
        println!("收到: {:?}", msg.payload);
    }

    Ok(())
}
```

→ 更多示例见 [nats.rs 官方文档](https://github.com/nats-io/nats.rs)

---

## 下一步

- [NATS Core 功能详解](./NatsCore.md) — 通配符、Queue Group、Header 等完整协议说明
- [SDK 接入](./SDK.md) — 各语言 SDK 及官方文档链接
- [JetStream](./JetStream.md) — 持久化消息流（开发中）
- [mq9](../mq9/Overview.md) — 基于 NATS 的 AI Agent 通信协议
