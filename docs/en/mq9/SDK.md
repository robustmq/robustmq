# SDK Integration

## Overview

mq9 supports two integration paths. The core difference is the level of abstraction:

**RobustMQ SDK** encapsulates complete mailbox semantics. You work directly with mailboxes — `create()`, `send()`, `subscribe()` — without needing to know what the underlying subjects look like, without understanding the NATS protocol, without manually constructing strings. Like using an email client to send mail without caring about SMTP. Best for teams that want fast integration and don't want to deal with protocol details.

**Native NATS SDK** is lower-level. You work directly with NATS subjects and must understand mq9's subject naming convention (`$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` etc.), using NATS primitives like `publish`/`request`/`subscribe`. The advantage is zero extra dependencies — any NATS client in any language works out of the box. The tradeoff is that you handle protocol details manually. Best for teams already familiar with NATS, or where minimal dependencies matter.

Both are functionally equivalent. Choose based on your needs.

---

## RobustMQ SDK

### Installation

| Language | Package | Install |
|----------|---------|---------|
| Python | `robustmq` (PyPI) | `pip install robustmq` |
| Go | `github.com/robustmq/robustmq-sdk/go` | `go get github.com/robustmq/robustmq-sdk/go` |
| JavaScript | `@robustmq/sdk` (npm) | `npm install @robustmq/sdk` |
| Java | `com.robustmq:robustmq` (Maven) | see Maven snippet below |
| Rust | `robustmq` (crates.io) | `cargo add robustmq` |
| C# | `RobustMQ` (NuGet) | `dotnet add package RobustMQ` |

### API Overview

The following methods are available in all language SDKs with consistent semantics:

| Method | Description | Returns |
|--------|-------------|---------|
| `connect()` | Connect to NATS server | — |
| `create(ttl, public?, name?, desc?)` | Create mailbox | `Mailbox(mail_id, public, name, desc)` |
| `send(mail_id, payload, priority?)` | Send message (fire-and-forget) | — |
| `subscribe(mail_id, callback, priority?, queue_group?)` | Subscribe to mailbox | `Subscription` |
| `list(mail_id)` | List message metadata (no payload) | `list[MessageMeta(msg_id, priority, ts)]` |
| `delete(mail_id, msg_id)` | Delete a message | — |
| `close()` | Disconnect | — |

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
        # Create private mailbox (TTL 1 hour)
        mailbox = await client.create(ttl=3600)
        print(f"Mailbox: {mailbox.mail_id}")

        # Send messages at different priorities
        await client.send(mailbox.mail_id, {"task": "analyze", "doc": "abc123"}, priority=Priority.CRITICAL)
        await client.send(mailbox.mail_id, "urgent interrupt", priority=Priority.URGENT)
        await client.send(mailbox.mail_id, b"routine task", priority=Priority.NORMAL)

        # Subscribe and process incoming messages
        received = []
        async def handler(msg):
            received.append(msg)
            print(f"Received [{msg.subject}]: {msg.data}")

        sub = await client.subscribe(mailbox.mail_id, handler)

        await asyncio.sleep(1)  # wait for delivery
        await sub.unsubscribe()

        # List metadata
        metas = await client.list(mailbox.mail_id)
        for meta in metas:
            print(f"  msg_id={meta.msg_id} priority={meta.priority} ts={meta.ts}")

        # Delete a message
        if metas:
            await client.delete(mailbox.mail_id, metas[0].msg_id)

asyncio.run(main())
```

**Public mailbox with queue group** — each task is delivered to exactly one worker:

```python
# Public task queue with queue group
queue = await client.create(ttl=86400, public=True, name="task.queue", desc="Worker queue")

# Worker 1 (run in separate coroutine or process)
sub1 = await client.subscribe(queue.mail_id, worker_handler, queue_group="workers")
# Worker 2
sub2 = await client.subscribe(queue.mail_id, worker_handler, queue_group="workers")
# Each task delivered to exactly one worker
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

    // Create mailbox
    mailbox, err := client.Create(ctx, mq9.CreateOptions{TTL: 3600})
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Mailbox: %s\n", mailbox.MailID)

    // Send message
    if err := client.Send(ctx, mailbox.MailID, []byte(`{"task":"analyze"}`), mq9.PriorityNormal); err != nil {
        log.Fatal(err)
    }

    // Subscribe
    sub, err := client.Subscribe(ctx, mailbox.MailID, func(msg *mq9.Message) {
        fmt.Printf("Received: %s\n", msg.Data)
    }, mq9.SubscribeOptions{Priority: "*"})
    if err != nil {
        log.Fatal(err)
    }
    defer sub.Unsubscribe()

    // List
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

### JavaScript (TypeScript)

```typescript
import { MQ9Client, Priority } from '@robustmq/sdk/mq9';
```

```typescript
import { MQ9Client, Priority } from '@robustmq/sdk/mq9';

async function main() {
  const client = new MQ9Client({ server: 'nats://localhost:4222' });
  await client.connect();

  try {
    // Create mailbox
    const mailbox = await client.create({ ttl: 3600 });
    console.log(`Mailbox: ${mailbox.mailId}`);

    // Send messages
    await client.send(mailbox.mailId, { task: 'analyze' }, Priority.CRITICAL);
    await client.send(mailbox.mailId, 'status update', Priority.NORMAL);

    // Subscribe
    const sub = await client.subscribe(mailbox.mailId, (msg) => {
      console.log(`Received [${msg.subject}]:`, msg.data);
    });

    await new Promise(resolve => setTimeout(resolve, 500));
    await sub.unsubscribe();

    // List and delete
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
            // Create mailbox
            Mailbox mailbox = client.create(3600).get();
            System.out.println("Mailbox: " + mailbox.getMailId());

            // Send message
            client.send(mailbox.getMailId(), "{\"task\":\"analyze\"}".getBytes(), Priority.NORMAL).get();

            // Subscribe
            Subscription sub = client.subscribe(mailbox.getMailId(), msg -> {
                System.out.println("Received: " + new String(msg.getData()));
            }, SubscribeOptions.allPriorities()).get();

            Thread.sleep(500);
            sub.unsubscribe();

            // List
            List<MessageMeta> metas = client.list(mailbox.getMailId()).get();
            metas.forEach(m -> System.out.println("msg_id=" + m.getMsgId()));

            // Delete
            if (!metas.isEmpty()) {
                client.delete(mailbox.getMailId(), metas.get(0).getMsgId()).get();
            }
        } finally {
            client.close();
        }
    }
}
```

**Maven dependency:**

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

    // Create mailbox
    let mailbox = client.create(3600).await?;
    println!("Mailbox: {}", mailbox.mail_id);

    // Send message
    client.send(&mailbox.mail_id, b"hello from rust", Priority::Normal).await?;

    // Subscribe
    let mut sub = client.subscribe(&mailbox.mail_id, SubscribeOptions::all_priorities()).await?;
    if let Some(msg) = sub.next().await {
        println!("Received: {:?}", msg.data);
    }
    sub.unsubscribe().await?;

    // List
    let metas = client.list(&mailbox.mail_id).await?;
    for m in &metas {
        println!("msg_id={} priority={}", m.msg_id, m.priority);
    }

    // Delete
    if let Some(first) = metas.first() {
        client.delete(&mailbox.mail_id, &first.msg_id).await?;
    }

    client.close().await?;
    Ok(())
}
```

**Cargo.toml:**

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
    // Create mailbox
    var mailbox = await client.CreateAsync(ttl: 3600);
    Console.WriteLine($"Mailbox: {mailbox.MailId}");

    // Send message
    await client.SendAsync(mailbox.MailId, System.Text.Encoding.UTF8.GetBytes("{\"task\":\"analyze\"}"), Priority.Normal);

    // Subscribe
    var sub = await client.SubscribeAsync(mailbox.MailId, async msg =>
    {
        Console.WriteLine($"Received: {System.Text.Encoding.UTF8.GetString(msg.Data)}");
    });

    await Task.Delay(500);
    await sub.UnsubscribeAsync();

    // List
    var metas = await client.ListAsync(mailbox.MailId);
    foreach (var meta in metas)
        Console.WriteLine($"msg_id={meta.MsgId} priority={meta.Priority}");

    // Delete
    if (metas.Count > 0)
        await client.DeleteAsync(mailbox.MailId, metas[0].MsgId);
}
finally
{
    await client.CloseAsync();
}
```

**NuGet:**

```bash
dotnet add package RobustMQ --version 0.3.5
```

---

---

---

## NATS SDK

Any NATS client library in any language can interact with mq9 directly — no RobustMQ SDK required. mq9 defines a set of subject naming conventions on top of standard NATS pub/sub:

- `nats pub` / `nc.publish()` → Send a message (fire-and-forget)
- `nats req` / `nc.request()` → Request with reply (CREATE, LIST, DELETE operations)
- `nats sub` / `nc.subscribe()` → Subscribe to a mailbox

The full subject reference is in [Protocol](./Protocol.md). All examples below connect to `nats://localhost:4222`.

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

    # --- Create a private mailbox ---
    reply = await nc.request(
        "$mq9.AI.MAILBOX.CREATE",
        json.dumps({"ttl": 3600}).encode(),
        timeout=5
    )
    mail_id = json.loads(reply.data)["mail_id"]
    print(f"Mailbox created: {mail_id}")

    # --- Send messages at different priorities ---
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.critical",
                     json.dumps({"type": "abort", "task_id": "t-001"}).encode())
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.urgent",
                     json.dumps({"type": "interrupt", "task_id": "t-002"}).encode())
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}",
                     json.dumps({"type": "task", "payload": "process dataset A"}).encode())

    # --- Subscribe to all priorities ---
    received = []

    async def handler(msg):
        data = json.loads(msg.data)
        print(f"Got [{msg.subject}]: {data}")
        received.append(data)

    sub = await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)
    await asyncio.sleep(1)
    await sub.unsubscribe()

    # --- List message metadata ---
    reply = await nc.request(
        f"$mq9.AI.MAILBOX.LIST.{mail_id}",
        b"{}",
        timeout=5
    )
    meta = json.loads(reply.data)
    print("Messages in mailbox:")
    for m in meta.get("messages", []):
        print(f"  msg_id={m['msg_id']} priority={m['priority']} ts={m['create_time']}")

    # --- Delete a message ---
    if meta.get("messages"):
        msg_id = meta["messages"][0]["msg_id"]
        await nc.request(
            f"$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}",
            b"{}",
            timeout=5
        )
        print(f"Deleted: {msg_id}")

    # --- Create a public mailbox ---
    reply = await nc.request(
        "$mq9.AI.MAILBOX.CREATE",
        json.dumps({"ttl": 86400, "public": True, "name": "task.queue", "desc": "Worker queue"}).encode(),
        timeout=5
    )
    print(f"Public mailbox: {json.loads(reply.data)['mail_id']}")

    await nc.close()

asyncio.run(main())
```

**Queue group (competing consumers):**

```python
import asyncio, nats, json

async def worker(worker_id: str):
    nc = await nats.connect("nats://localhost:4222")

    async def handler(msg):
        data = json.loads(msg.data)
        print(f"Worker {worker_id} got task: {data}")

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

    // --- Create a private mailbox ---
    payload, _ := json.Marshal(map[string]interface{}{"ttl": 3600})
    msg, err := nc.Request("$mq9.AI.MAILBOX.CREATE", payload, 5*time.Second)
    if err != nil {
        log.Fatal(err)
    }
    var createResp map[string]string
    json.Unmarshal(msg.Data, &createResp)
    mailID := createResp["mail_id"]
    fmt.Printf("Mailbox created: %s\n", mailID)

    // --- Send messages ---
    taskPayload, _ := json.Marshal(map[string]string{"type": "task", "payload": "process dataset A"})
    nc.Publish("$mq9.AI.MAILBOX.MSG."+mailID, taskPayload)

    abortPayload, _ := json.Marshal(map[string]string{"type": "abort", "task_id": "t-001"})
    nc.Publish("$mq9.AI.MAILBOX.MSG."+mailID+".critical", abortPayload)

    // --- Subscribe ---
    sub, err := nc.Subscribe("$mq9.AI.MAILBOX.MSG."+mailID+".*", func(msg *nats.Msg) {
        fmt.Printf("Got [%s]: %s\n", msg.Subject, string(msg.Data))
    })
    if err != nil {
        log.Fatal(err)
    }
    time.Sleep(500 * time.Millisecond)
    sub.Unsubscribe()

    // --- List message metadata ---
    listMsg, err := nc.Request("$mq9.AI.MAILBOX.LIST."+mailID, []byte("{}"), 5*time.Second)
    if err != nil {
        log.Fatal(err)
    }
    var listResp map[string][]map[string]interface{}
    json.Unmarshal(listMsg.Data, &listResp)
    for _, m := range listResp["messages"] {
        fmt.Printf("  msg_id=%s priority=%s\n", m["msg_id"], m["priority"])
    }

    // --- Queue group (competing consumers) ---
    qsub, _ := nc.QueueSubscribe("$mq9.AI.MAILBOX.MSG.task.queue.*", "workers",
        func(msg *nats.Msg) {
            fmt.Printf("Worker got: %s\n", string(msg.Data))
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

  // --- Create a private mailbox ---
  const createResp = await nc.request(
    "$mq9.AI.MAILBOX.CREATE",
    sc.encode(JSON.stringify({ ttl: 3600 }))
  );
  const { mail_id } = JSON.parse(sc.decode(createResp.data));
  console.log(`Mailbox created: ${mail_id}`);

  // --- Send messages ---
  nc.publish(
    `$mq9.AI.MAILBOX.MSG.${mail_id}.critical`,
    sc.encode(JSON.stringify({ type: "abort", task_id: "t-001" }))
  );
  nc.publish(
    `$mq9.AI.MAILBOX.MSG.${mail_id}`,
    sc.encode(JSON.stringify({ type: "task", payload: "process dataset A" }))
  );

  // --- Subscribe ---
  const sub = nc.subscribe(`$mq9.AI.MAILBOX.MSG.${mail_id}.*`);
  (async () => {
    for await (const msg of sub) {
      console.log(`Got [${msg.subject}]:`, JSON.parse(sc.decode(msg.data)));
    }
  })();

  await new Promise((r) => setTimeout(r, 500));
  sub.unsubscribe();

  // --- List message metadata ---
  const listResp = await nc.request(
    `$mq9.AI.MAILBOX.LIST.${mail_id}`,
    sc.encode("{}")
  );
  const { messages } = JSON.parse(sc.decode(listResp.data));
  messages?.forEach((m: any) =>
    console.log(`  msg_id=${m.msg_id} priority=${m.priority}`)
  );

  // --- Queue group ---
  const qsub = nc.subscribe(`$mq9.AI.MAILBOX.MSG.task.queue.*`, {
    queue: "workers",
  });
  (async () => {
    for await (const msg of qsub) {
      console.log(`Worker got:`, JSON.parse(sc.decode(msg.data)));
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

    // --- Create a private mailbox ---
    let payload = serde_json::json!({"ttl": 3600}).to_string().into_bytes();
    let response = client
        .request("$mq9.AI.MAILBOX.CREATE".to_string(), payload.into())
        .await?;
    let create_resp: serde_json::Value = serde_json::from_slice(&response.payload)?;
    let mail_id = create_resp["mail_id"].as_str().unwrap();
    println!("Mailbox created: {}", mail_id);

    // --- Send a message ---
    let task = serde_json::json!({"type": "task", "payload": "process dataset A"})
        .to_string()
        .into_bytes();
    client
        .publish(format!("$mq9.AI.MAILBOX.MSG.{}", mail_id), task.into())
        .await?;

    // --- Subscribe ---
    let mut subscriber = client
        .subscribe(format!("$mq9.AI.MAILBOX.MSG.{}.*", mail_id))
        .await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    while let Ok(Some(msg)) =
        tokio::time::timeout(Duration::from_millis(100), subscriber.next()).await
    {
        let data: serde_json::Value = serde_json::from_slice(&msg.payload)?;
        println!("Got [{}]: {}", msg.subject, data);
    }
    subscriber.unsubscribe().await?;

    // --- List message metadata ---
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

            // --- Create a private mailbox ---
            byte[] createPayload = mapper.writeValueAsBytes(Map.of("ttl", 3600));
            Message createResp = nc.request("$mq9.AI.MAILBOX.CREATE",
                    createPayload, Duration.ofSeconds(5));
            Map<?, ?> createData = mapper.readValue(createResp.getData(), Map.class);
            String mailId = (String) createData.get("mail_id");
            System.out.println("Mailbox created: " + mailId);

            // --- Send a message ---
            byte[] taskPayload = mapper.writeValueAsBytes(
                    Map.of("type", "task", "payload", "process dataset A"));
            nc.publish("$mq9.AI.MAILBOX.MSG." + mailId, taskPayload);

            // --- Subscribe ---
            Dispatcher dispatcher = nc.createDispatcher(msg -> {
                try {
                    Map<?, ?> data = mapper.readValue(msg.getData(), Map.class);
                    System.out.println("Got [" + msg.getSubject() + "]: " + data);
                } catch (Exception e) { e.printStackTrace(); }
            });
            dispatcher.subscribe("$mq9.AI.MAILBOX.MSG." + mailId + ".*");
            Thread.sleep(500);

            // --- List message metadata ---
            Message listResp = nc.request("$mq9.AI.MAILBOX.LIST." + mailId,
                    "{}".getBytes(), Duration.ofSeconds(5));
            Map<?, ?> listData = mapper.readValue(listResp.getData(), Map.class);
            System.out.println("Messages: " + listData.get("messages"));

            // --- Queue group ---
            Dispatcher queueDispatcher = nc.createDispatcher(msg -> {
                System.out.println("Worker got: " + new String(msg.getData()));
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

// --- Create a private mailbox ---
var createPayload = JsonSerializer.SerializeToUtf8Bytes(new { ttl = 3600 });
var createResp = await nats.RequestAsync<byte[], JsonElement>(
    "$mq9.AI.MAILBOX.CREATE", createPayload);
var mailId = createResp.Data.GetProperty("mail_id").GetString()!;
Console.WriteLine($"Mailbox created: {mailId}");

// --- Send a message ---
var taskPayload = JsonSerializer.SerializeToUtf8Bytes(
    new { type = "task", payload = "process dataset A" });
await nats.PublishAsync($"$mq9.AI.MAILBOX.MSG.{mailId}", taskPayload);

// --- Subscribe ---
await foreach (var msg in nats.SubscribeAsync<byte[]>($"$mq9.AI.MAILBOX.MSG.{mailId}.*"))
{
    var data = JsonSerializer.Deserialize<JsonElement>(msg.Data!);
    Console.WriteLine($"Got [{msg.Subject}]: {data}");
    break;
}

// --- List message metadata ---
var listResp = await nats.RequestAsync<byte[], JsonElement>(
    $"$mq9.AI.MAILBOX.LIST.{mailId}", Encoding.UTF8.GetBytes("{}"));
var messages = listResp.Data.GetProperty("messages");
foreach (var m in messages.EnumerateArray())
    Console.WriteLine($"  msg_id={m.GetProperty("msg_id")} priority={m.GetProperty("priority")}");

// --- Delete a message ---
if (messages.GetArrayLength() > 0)
{
    var msgId = messages[0].GetProperty("msg_id").GetString()!;
    await nats.RequestAsync<byte[], JsonElement>(
        $"$mq9.AI.MAILBOX.DELETE.{mailId}.{msgId}", Encoding.UTF8.GetBytes("{}"));
    Console.WriteLine($"Deleted: {msgId}");
}
```

---

## Subject Quick Reference

| Operation | NATS method | Subject |
|-----------|------------|---------|
| Create private mailbox | `request` | `$mq9.AI.MAILBOX.CREATE` |
| Create public mailbox | `request` | `$mq9.AI.MAILBOX.CREATE` |
| Send message | `publish` | `$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` |
| Subscribe (all priorities) | `subscribe` | `$mq9.AI.MAILBOX.MSG.{mail_id}.*` |
| Subscribe (one priority) | `subscribe` | `$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` |
| Queue group subscription | `queueSubscribe` | `$mq9.AI.MAILBOX.MSG.{mail_id}.*` |
| List metadata | `request` | `$mq9.AI.MAILBOX.LIST.{mail_id}` |
| Delete message | `request` | `$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}` |
| Discover public mailboxes | `subscribe` | `$mq9.AI.PUBLIC.LIST` |

Full protocol specification: [Protocol Design](./Protocol.md).
