# 使用 NATS 客户端库接入 mq9

mq9 构建在 NATS 之上。任何语言的 NATS 客户端库都可以直接与 mq9 交互——无需 RobustMQ SDK。本页展示如何使用原生 NATS 客户端库执行所有 mq9 操作。

---

## 工作原理

mq9 在标准 NATS pub/sub 之上定义了一套 subject 命名约定：

- `nats pub` / `nc.publish()` → 发送消息（即发即忘）
- `nats req` / `nc.request()` → 请求-回复（CREATE、LIST、DELETE 操作）
- `nats sub` / `nc.subscribe()` → 订阅邮箱

完整 subject 参考见[协议设计](./Protocol.md)。以下所有示例均连接 `nats://localhost:4222`。

---

## Python — nats-py

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
    await asyncio.sleep(1)  # 等待先存储后推送完成
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
        print(f"  msg_id={m['msg_id']} priority={m['priority']} ts={m['ts']}")

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

    # 相同队列组名的所有 Worker 共享负载
    sub = await nc.subscribe("$mq9.AI.MAILBOX.MSG.task.queue.*",
                             queue="workers", cb=handler)
    await asyncio.sleep(30)
    await sub.unsubscribe()
    await nc.close()

asyncio.run(worker("w-1"))
```

---

## Go — nats.go

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

## JavaScript — nats.js

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

## Rust — async-nats

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
        .publish(
            format!("$mq9.AI.MAILBOX.MSG.{}", mail_id),
            task.into(),
        )
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

## Java — jnats

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

## C# — NATS.Net

```
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
