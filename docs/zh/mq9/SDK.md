# SDK 接入

## 概述

mq9 支持两种接入方式，核心区别在于抽象层次：

**RobustMQ SDK** 封装了完整的邮箱语义。你直接操作邮箱——`create()`、`send()`、`subscribe()`——不需要知道底层 subject 是什么，不需要了解 NATS 协议，不需要手动拼接字符串。就像用邮件客户端发邮件，不用关心 SMTP 协议一样。适合想快速集成、不想处理协议细节的场景。

**NATS 原生 SDK** 更底层。你直接操作 NATS subject，需要理解 mq9 的 subject 命名规范（`$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` 等），使用 `publish`/`request`/`subscribe` 等 NATS 原语。优点是零额外依赖，任何语言的 NATS 客户端开箱即用；代价是需要手动处理协议细节。适合已有 NATS 基础、或需要最小依赖的场景。

两种方式功能完全等价，按实际需求选择即可。

---

## RobustMQ SDK

### 安装

| 语言 | 包名 | 安装 |
|------|------|------|
| Python | `robustmq` (PyPI) | `pip install robustmq` |
| Go | `github.com/robustmq/robustmq-sdk/go` | `go get github.com/robustmq/robustmq-sdk/go` |
| JavaScript | `@robustmq/sdk` (npm) | `npm install @robustmq/sdk` |
| Java | `com.robustmq:robustmq` (Maven) | 见下方 Maven 片段 |
| Rust | `robustmq` (crates.io) | `cargo add robustmq` |
| C# | `RobustMQ` (NuGet) | `dotnet add package RobustMQ` |

### API 概览

所有语言 SDK 提供以下方法，语义一致：

| 方法 | 说明 | 返回 |
|------|------|------|
| `connect()` | 连接 NATS 服务器 | — |
| `create(ttl, public?, name?, desc?)` | 创建邮箱 | `Mailbox(mail_id, public, name, desc)` |
| `send(mail_id, payload, priority?)` | 发送消息（即发即忘） | — |
| `subscribe(mail_id, callback, priority?, queue_group?)` | 订阅邮箱 | `Subscription` |
| `list(mail_id)` | 列出消息元数据（不含消息体） | `list[MessageMeta(msg_id, priority, ts)]` |
| `delete(mail_id, msg_id)` | 删除消息 | — |
| `close()` | 断开连接 | — |

---

### Python

```python
from robustmq.mq9 import Client, Priority
```

```python
import asyncio
from robustmq.mq9 import Client, Priority

async def main():
    async with Client(server="nats://localhost:4222") as client:
        # 创建私有邮箱（TTL 1 小时）
        mailbox = await client.create(ttl=3600)
        print(f"邮箱: {mailbox.mail_id}")

        # 按不同优先级发送消息
        await client.send(mailbox.mail_id, {"task": "analyze", "doc": "abc123"}, priority=Priority.CRITICAL)
        await client.send(mailbox.mail_id, "urgent interrupt", priority=Priority.URGENT)
        await client.send(mailbox.mail_id, b"routine task", priority=Priority.NORMAL)

        # 订阅并处理收到的消息
        received = []
        async def handler(msg):
            received.append(msg)
            print(f"收到 [{msg.subject}]: {msg.data}")

        sub = await client.subscribe(mailbox.mail_id, handler)

        await asyncio.sleep(1)  # 等待投递
        await sub.unsubscribe()

        # 列出元数据
        metas = await client.list(mailbox.mail_id)
        for meta in metas:
            print(f"  msg_id={meta.msg_id} priority={meta.priority} ts={meta.ts}")

        # 删除消息
        if metas:
            await client.delete(mailbox.mail_id, metas[0].msg_id)

asyncio.run(main())
```

**公开邮箱 + 队列组** — 每条任务只投递给一个 Worker：

```python
# 创建公开任务队列
queue = await client.create(ttl=86400, public=True, name="task.queue", desc="Worker 队列")

# Worker 1（在单独的协程或进程中运行）
sub1 = await client.subscribe(queue.mail_id, worker_handler, queue_group="workers")
# Worker 2
sub2 = await client.subscribe(queue.mail_id, worker_handler, queue_group="workers")
# 每条任务只投递给一个 Worker
```

---

### Go

```go
import "github.com/robustmq/robustmq-sdk/go/mq9"
```

```go
package main

import (
    "context"
    "fmt"
    "log"
    "github.com/robustmq/robustmq-sdk/go/mq9"
)

func main() {
    client := mq9.NewMQ9Client("nats://localhost:4222")
    if err := client.Connect(); err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    ctx := context.Background()

    // 创建邮箱
    mailbox, err := client.Create(ctx, mq9.CreateOptions{TTL: 3600})
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("邮箱: %s\n", mailbox.MailID)

    // 发送消息
    if err := client.Send(ctx, mailbox.MailID, []byte(`{"task":"analyze"}`), mq9.PriorityNormal); err != nil {
        log.Fatal(err)
    }

    // 订阅
    sub, err := client.Subscribe(ctx, mailbox.MailID, func(msg *mq9.Message) {
        fmt.Printf("收到: %s\n", msg.Data)
    }, mq9.SubscribeOptions{Priority: "*"})
    if err != nil {
        log.Fatal(err)
    }
    defer sub.Unsubscribe()

    // 列出元数据
    metas, err := client.List(ctx, mailbox.MailID)
    if err != nil {
        log.Fatal(err)
    }
    for _, m := range metas {
        fmt.Printf("msg_id=%s priority=%s\n", m.MsgID, m.Priority)
    }
}
```

---

### JavaScript（TypeScript）

```typescript
import { MQ9Client, Priority } from '@robustmq/sdk/mq9';
```

```typescript
import { MQ9Client, Priority } from '@robustmq/sdk/mq9';

async function main() {
  const client = new MQ9Client({ server: 'nats://localhost:4222' });
  await client.connect();

  try {
    // 创建邮箱
    const mailbox = await client.create({ ttl: 3600 });
    console.log(`邮箱: ${mailbox.mailId}`);

    // 发送消息
    await client.send(mailbox.mailId, { task: 'analyze' }, Priority.CRITICAL);
    await client.send(mailbox.mailId, 'status update', Priority.NORMAL);

    // 订阅
    const sub = await client.subscribe(mailbox.mailId, (msg) => {
      console.log(`收到 [${msg.subject}]:`, msg.data);
    });

    await new Promise(resolve => setTimeout(resolve, 500));
    await sub.unsubscribe();

    // 列出并删除
    const metas = await client.list(mailbox.mailId);
    if (metas.length > 0) {
      await client.delete(mailbox.mailId, metas[0].msgId);
    }
  } finally {
    await client.close();
  }
}

main().catch(console.error);
```

---

### Java

```java
import com.robustmq.mq9.*;
import java.util.List;
import java.util.concurrent.CompletableFuture;

public class Example {
    public static void main(String[] args) throws Exception {
        MQ9Client client = new MQ9Client("nats://localhost:4222");
        client.connect();

        try {
            // 创建邮箱
            Mailbox mailbox = client.create(3600).get();
            System.out.println("邮箱: " + mailbox.getMailId());

            // 发送消息
            client.send(mailbox.getMailId(), "{\"task\":\"analyze\"}".getBytes(), Priority.NORMAL).get();

            // 订阅
            Subscription sub = client.subscribe(mailbox.getMailId(), msg -> {
                System.out.println("收到: " + new String(msg.getData()));
            }, SubscribeOptions.allPriorities()).get();

            Thread.sleep(500);
            sub.unsubscribe();

            // 列出
            List<MessageMeta> metas = client.list(mailbox.getMailId()).get();
            metas.forEach(m -> System.out.println("msg_id=" + m.getMsgId()));

            // 删除
            if (!metas.isEmpty()) {
                client.delete(mailbox.getMailId(), metas.get(0).getMsgId()).get();
            }
        } finally {
            client.close();
        }
    }
}
```

**Maven 依赖：**

```xml
<dependency>
  <groupId>com.robustmq</groupId>
  <artifactId>robustmq</artifactId>
  <version>0.3.5</version>
</dependency>
```

---

### Rust

```rust
use robustmq::mq9::{MQ9Client, Priority};
```

```rust
use robustmq::mq9::{MQ9Client, Priority, SubscribeOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MQ9Client::connect("nats://localhost:4222").await?;

    // 创建邮箱
    let mailbox = client.create(3600).await?;
    println!("邮箱: {}", mailbox.mail_id);

    // 发送消息
    client.send(&mailbox.mail_id, b"hello from rust", Priority::Normal).await?;

    // 订阅
    let mut sub = client.subscribe(&mailbox.mail_id, SubscribeOptions::all_priorities()).await?;
    if let Some(msg) = sub.next().await {
        println!("收到: {:?}", msg.data);
    }
    sub.unsubscribe().await?;

    // 列出
    let metas = client.list(&mailbox.mail_id).await?;
    for m in &metas {
        println!("msg_id={} priority={}", m.msg_id, m.priority);
    }

    // 删除
    if let Some(first) = metas.first() {
        client.delete(&mailbox.mail_id, &first.msg_id).await?;
    }

    client.close().await?;
    Ok(())
}
```

**Cargo.toml：**

```toml
[dependencies]
robustmq = "0.3.5"
tokio = { version = "1", features = ["full"] }
```

---

### C# (RobustMQ SDK)

```csharp
using RobustMQ.Mq9;
```

```csharp
using RobustMQ.Mq9;

var client = new MQ9Client("nats://localhost:4222");
await client.ConnectAsync();

try
{
    // 创建邮箱
    var mailbox = await client.CreateAsync(ttl: 3600);
    Console.WriteLine($"邮箱: {mailbox.MailId}");

    // 发送消息
    await client.SendAsync(mailbox.MailId, System.Text.Encoding.UTF8.GetBytes("{\"task\":\"analyze\"}"), Priority.Normal);

    // 订阅
    var sub = await client.SubscribeAsync(mailbox.MailId, async msg =>
    {
        Console.WriteLine($"收到: {System.Text.Encoding.UTF8.GetString(msg.Data)}");
    });

    await Task.Delay(500);
    await sub.UnsubscribeAsync();

    // 列出
    var metas = await client.ListAsync(mailbox.MailId);
    foreach (var meta in metas)
        Console.WriteLine($"msg_id={meta.MsgId} priority={meta.Priority}");

    // 删除
    if (metas.Count > 0)
        await client.DeleteAsync(mailbox.MailId, metas[0].MsgId);
}
finally
{
    await client.CloseAsync();
}
```

**NuGet：**

```bash
dotnet add package RobustMQ --version 0.3.5
```

---

## NATS SDK

任何语言的 NATS 客户端库都可以直接与 mq9 交互——无需 RobustMQ SDK。mq9 在标准 NATS pub/sub 之上定义了一套 subject 命名约定：

- `nats pub` / `nc.publish()` → 发送消息（即发即忘）
- `nats req` / `nc.request()` → 请求-回复（CREATE、LIST、DELETE 操作）
- `nats sub` / `nc.subscribe()` → 订阅邮箱

完整 subject 参考见[协议设计](./Protocol.md)。以下所有示例均连接 `nats://localhost:4222`。

---

### Python — nats-py

```bash
pip install nats-py
```

```python
import asyncio
import json
import nats

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # --- 创建私有邮箱 ---
    reply = await nc.request(
        "$mq9.AI.MAILBOX.CREATE",
        json.dumps({"ttl": 3600}).encode(),
        timeout=5
    )
    mail_id = json.loads(reply.data)["mail_id"]
    print(f"邮箱创建成功: {mail_id}")

    # --- 按不同优先级发送消息 ---
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.critical",
                     json.dumps({"type": "abort", "task_id": "t-001"}).encode())
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.urgent",
                     json.dumps({"type": "interrupt", "task_id": "t-002"}).encode())
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}",
                     json.dumps({"type": "task", "payload": "process dataset A"}).encode())

    # --- 订阅所有优先级 ---
    received = []

    async def handler(msg):
        data = json.loads(msg.data)
        print(f"收到 [{msg.subject}]: {data}")
        received.append(data)

    sub = await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)
    await asyncio.sleep(1)
    await sub.unsubscribe()

    # --- 列出消息元数据 ---
    reply = await nc.request(
        f"$mq9.AI.MAILBOX.LIST.{mail_id}",
        b"{}",
        timeout=5
    )
    meta = json.loads(reply.data)
    print("邮箱中的消息:")
    for m in meta.get("messages", []):
        print(f"  msg_id={m['msg_id']} priority={m['priority']} ts={m['create_time']}")

    # --- 删除消息 ---
    if meta.get("messages"):
        msg_id = meta["messages"][0]["msg_id"]
        await nc.request(
            f"$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}",
            b"{}",
            timeout=5
        )
        print(f"已删除消息: {msg_id}")

    # --- 创建公开邮箱 ---
    reply = await nc.request(
        "$mq9.AI.MAILBOX.CREATE",
        json.dumps({"ttl": 86400, "public": True, "name": "task.queue", "desc": "Worker 队列"}).encode(),
        timeout=5
    )
    print(f"公开邮箱: {json.loads(reply.data)['mail_id']}")

    await nc.close()

asyncio.run(main())
```

**队列组（竞争消费）：**

```python
import asyncio, nats, json

async def worker(worker_id: str):
    nc = await nats.connect("nats://localhost:4222")

    async def handler(msg):
        data = json.loads(msg.data)
        print(f"Worker {worker_id} 收到任务: {data}")

    sub = await nc.subscribe("$mq9.AI.MAILBOX.MSG.task.queue.*",
                             queue="workers", cb=handler)
    await asyncio.sleep(30)
    await sub.unsubscribe()
    await nc.close()

asyncio.run(worker("w-1"))
```

---

### Go — nats.go

```bash
go get github.com/nats-io/nats.go
```

```go
package main

import (
    "encoding/json"
    "fmt"
    "log"
    "time"

    "github.com/nats-io/nats.go"
)

func main() {
    nc, err := nats.Connect("nats://localhost:4222")
    if err != nil {
        log.Fatal(err)
    }
    defer nc.Close()

    // --- 创建私有邮箱 ---
    payload, _ := json.Marshal(map[string]interface{}{"ttl": 3600})
    msg, err := nc.Request("$mq9.AI.MAILBOX.CREATE", payload, 5*time.Second)
    if err != nil {
        log.Fatal(err)
    }
    var createResp map[string]string
    json.Unmarshal(msg.Data, &createResp)
    mailID := createResp["mail_id"]
    fmt.Printf("邮箱创建成功: %s\n", mailID)

    // --- 发送消息 ---
    taskPayload, _ := json.Marshal(map[string]string{"type": "task", "payload": "process dataset A"})
    nc.Publish("$mq9.AI.MAILBOX.MSG."+mailID, taskPayload)

    abortPayload, _ := json.Marshal(map[string]string{"type": "abort", "task_id": "t-001"})
    nc.Publish("$mq9.AI.MAILBOX.MSG."+mailID+".critical", abortPayload)

    // --- 订阅 ---
    sub, err := nc.Subscribe("$mq9.AI.MAILBOX.MSG."+mailID+".*", func(msg *nats.Msg) {
        fmt.Printf("收到 [%s]: %s\n", msg.Subject, string(msg.Data))
    })
    if err != nil {
        log.Fatal(err)
    }
    time.Sleep(500 * time.Millisecond)
    sub.Unsubscribe()

    // --- 列出消息元数据 ---
    listMsg, err := nc.Request("$mq9.AI.MAILBOX.LIST."+mailID, []byte("{}"), 5*time.Second)
    if err != nil {
        log.Fatal(err)
    }
    var listResp map[string][]map[string]interface{}
    json.Unmarshal(listMsg.Data, &listResp)
    for _, m := range listResp["messages"] {
        fmt.Printf("  msg_id=%s priority=%s\n", m["msg_id"], m["priority"])
    }

    // --- 队列组（竞争消费） ---
    qsub, _ := nc.QueueSubscribe("$mq9.AI.MAILBOX.MSG.task.queue.*", "workers",
        func(msg *nats.Msg) {
            fmt.Printf("Worker 收到: %s\n", string(msg.Data))
        })
    defer qsub.Unsubscribe()
}
```

---

### JavaScript — nats.js

```bash
npm install nats
```

```typescript
import { connect, StringCodec } from "nats";

const sc = StringCodec();

async function main() {
  const nc = await connect({ servers: "nats://localhost:4222" });

  // --- 创建私有邮箱 ---
  const createResp = await nc.request(
    "$mq9.AI.MAILBOX.CREATE",
    sc.encode(JSON.stringify({ ttl: 3600 }))
  );
  const { mail_id } = JSON.parse(sc.decode(createResp.data));
  console.log(`邮箱创建成功: ${mail_id}`);

  // --- 发送消息 ---
  nc.publish(
    `$mq9.AI.MAILBOX.MSG.${mail_id}.critical`,
    sc.encode(JSON.stringify({ type: "abort", task_id: "t-001" }))
  );
  nc.publish(
    `$mq9.AI.MAILBOX.MSG.${mail_id}`,
    sc.encode(JSON.stringify({ type: "task", payload: "process dataset A" }))
  );

  // --- 订阅 ---
  const sub = nc.subscribe(`$mq9.AI.MAILBOX.MSG.${mail_id}.*`);
  (async () => {
    for await (const msg of sub) {
      console.log(`收到 [${msg.subject}]:`, JSON.parse(sc.decode(msg.data)));
    }
  })();

  await new Promise((r) => setTimeout(r, 500));
  sub.unsubscribe();

  // --- 列出消息元数据 ---
  const listResp = await nc.request(
    `$mq9.AI.MAILBOX.LIST.${mail_id}`,
    sc.encode("{}")
  );
  const { messages } = JSON.parse(sc.decode(listResp.data));
  messages?.forEach((m: any) =>
    console.log(`  msg_id=${m.msg_id} priority=${m.priority}`)
  );

  // --- 队列组 ---
  const qsub = nc.subscribe(`$mq9.AI.MAILBOX.MSG.task.queue.*`, {
    queue: "workers",
  });
  (async () => {
    for await (const msg of qsub) {
      console.log(`Worker 收到:`, JSON.parse(sc.decode(msg.data)));
    }
  })();

  await nc.close();
}

main().catch(console.error);
```

---

### Rust — async-nats

```toml
# Cargo.toml
[dependencies]
async-nats = "0.36"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

```rust
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    // --- 创建私有邮箱 ---
    let payload = serde_json::json!({"ttl": 3600}).to_string().into_bytes();
    let response = client
        .request("$mq9.AI.MAILBOX.CREATE".to_string(), payload.into())
        .await?;
    let create_resp: serde_json::Value = serde_json::from_slice(&response.payload)?;
    let mail_id = create_resp["mail_id"].as_str().unwrap();
    println!("邮箱创建成功: {}", mail_id);

    // --- 发送消息 ---
    let task = serde_json::json!({"type": "task", "payload": "process dataset A"})
        .to_string()
        .into_bytes();
    client
        .publish(format!("$mq9.AI.MAILBOX.MSG.{}", mail_id), task.into())
        .await?;

    // --- 订阅 ---
    let mut subscriber = client
        .subscribe(format!("$mq9.AI.MAILBOX.MSG.{}.*", mail_id))
        .await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    while let Ok(Some(msg)) =
        tokio::time::timeout(Duration::from_millis(100), subscriber.next()).await
    {
        let data: serde_json::Value = serde_json::from_slice(&msg.payload)?;
        println!("收到 [{}]: {}", msg.subject, data);
    }
    subscriber.unsubscribe().await?;

    // --- 列出消息元数据 ---
    let list_resp = client
        .request(
            format!("$mq9.AI.MAILBOX.LIST.{}", mail_id),
            b"{}".as_ref().into(),
        )
        .await?;
    let list_data: serde_json::Value = serde_json::from_slice(&list_resp.payload)?;
    if let Some(messages) = list_data["messages"].as_array() {
        for m in messages {
            println!("  msg_id={} priority={}", m["msg_id"], m["priority"]);
        }
    }

    Ok(())
}
```

---

### Java — jnats

```xml
<!-- pom.xml -->
<dependency>
  <groupId>io.nats</groupId>
  <artifactId>jnats</artifactId>
  <version>2.17.6</version>
</dependency>
```

```java
import io.nats.client.*;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.time.Duration;
import java.util.Map;

public class Mq9Example {
    static final ObjectMapper mapper = new ObjectMapper();

    public static void main(String[] args) throws Exception {
        Options options = new Options.Builder().server("nats://localhost:4222").build();
        try (Connection nc = Nats.connect(options)) {

            // --- 创建私有邮箱 ---
            byte[] createPayload = mapper.writeValueAsBytes(Map.of("ttl", 3600));
            Message createResp = nc.request("$mq9.AI.MAILBOX.CREATE",
                    createPayload, Duration.ofSeconds(5));
            Map<?, ?> createData = mapper.readValue(createResp.getData(), Map.class);
            String mailId = (String) createData.get("mail_id");
            System.out.println("邮箱创建成功: " + mailId);

            // --- 发送消息 ---
            byte[] taskPayload = mapper.writeValueAsBytes(
                    Map.of("type", "task", "payload", "process dataset A"));
            nc.publish("$mq9.AI.MAILBOX.MSG." + mailId, taskPayload);

            // --- 订阅 ---
            Dispatcher dispatcher = nc.createDispatcher(msg -> {
                try {
                    Map<?, ?> data = mapper.readValue(msg.getData(), Map.class);
                    System.out.println("收到 [" + msg.getSubject() + "]: " + data);
                } catch (Exception e) { e.printStackTrace(); }
            });
            dispatcher.subscribe("$mq9.AI.MAILBOX.MSG." + mailId + ".*");
            Thread.sleep(500);

            // --- 列出消息元数据 ---
            Message listResp = nc.request("$mq9.AI.MAILBOX.LIST." + mailId,
                    "{}".getBytes(), Duration.ofSeconds(5));
            Map<?, ?> listData = mapper.readValue(listResp.getData(), Map.class);
            System.out.println("消息列表: " + listData.get("messages"));

            // --- 队列组 ---
            Dispatcher queueDispatcher = nc.createDispatcher(msg -> {
                System.out.println("Worker 收到: " + new String(msg.getData()));
            });
            queueDispatcher.subscribe("$mq9.AI.MAILBOX.MSG.task.queue.*", "workers");
        }
    }
}
```

---

### C# — NATS.Net

```bash
dotnet add package NATS.Net
```

```csharp
using NATS.Client.Core;
using System.Text;
using System.Text.Json;

await using var nats = new NatsConnection(new NatsOpts { Url = "nats://localhost:4222" });
await nats.ConnectAsync();

// --- 创建私有邮箱 ---
var createPayload = JsonSerializer.SerializeToUtf8Bytes(new { ttl = 3600 });
var createResp = await nats.RequestAsync<byte[], JsonElement>(
    "$mq9.AI.MAILBOX.CREATE", createPayload);
var mailId = createResp.Data.GetProperty("mail_id").GetString()!;
Console.WriteLine($"邮箱创建成功: {mailId}");

// --- 发送消息 ---
var taskPayload = JsonSerializer.SerializeToUtf8Bytes(
    new { type = "task", payload = "process dataset A" });
await nats.PublishAsync($"$mq9.AI.MAILBOX.MSG.{mailId}", taskPayload);

// --- 订阅 ---
await foreach (var msg in nats.SubscribeAsync<byte[]>($"$mq9.AI.MAILBOX.MSG.{mailId}.*"))
{
    var data = JsonSerializer.Deserialize<JsonElement>(msg.Data!);
    Console.WriteLine($"收到 [{msg.Subject}]: {data}");
    break;
}

// --- 列出消息元数据 ---
var listResp = await nats.RequestAsync<byte[], JsonElement>(
    $"$mq9.AI.MAILBOX.LIST.{mailId}", Encoding.UTF8.GetBytes("{}"));
var messages = listResp.Data.GetProperty("messages");
foreach (var m in messages.EnumerateArray())
    Console.WriteLine($"  msg_id={m.GetProperty("msg_id")} priority={m.GetProperty("priority")}");

// --- 删除消息 ---
if (messages.GetArrayLength() > 0)
{
    var msgId = messages[0].GetProperty("msg_id").GetString()!;
    await nats.RequestAsync<byte[], JsonElement>(
        $"$mq9.AI.MAILBOX.DELETE.{mailId}.{msgId}", Encoding.UTF8.GetBytes("{}"));
    Console.WriteLine($"已删除: {msgId}");
}
```

---

## Subject 快速参考

| 操作 | NATS 方法 | Subject |
|------|---------|---------|
| 创建私有邮箱 | `request` | `$mq9.AI.MAILBOX.CREATE` |
| 创建公开邮箱 | `request` | `$mq9.AI.MAILBOX.CREATE` |
| 发送消息 | `publish` | `$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` |
| 订阅（所有优先级） | `subscribe` | `$mq9.AI.MAILBOX.MSG.{mail_id}.*` |
| 订阅（单一优先级） | `subscribe` | `$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` |
| 队列组订阅 | `queueSubscribe` | `$mq9.AI.MAILBOX.MSG.{mail_id}.*` |
| 列出元数据 | `request` | `$mq9.AI.MAILBOX.LIST.{mail_id}` |
| 删除消息 | `request` | `$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}` |
| 发现公开邮箱 | `subscribe` | `$mq9.AI.PUBLIC.LIST` |

完整协议规范见[协议设计](./Protocol.md)。
