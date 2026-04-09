# Using mq9 with NATS Client Libraries

mq9 is built on top of NATS. Any NATS client library in any language can interact with mq9 directly — no RobustMQ SDK required. This page shows how to use native NATS client libraries to perform all mq9 operations.

---

## How It Works

mq9 defines a set of subject naming conventions on top of standard NATS pub/sub:

- `nats pub` / `nc.publish()` → Send a message (fire-and-forget)
- `nats req` / `nc.request()` → Request with reply (CREATE, LIST, DELETE operations)
- `nats sub` / `nc.subscribe()` → Subscribe to a mailbox

The full subject reference is in [Protocol](./Protocol.md). All examples below connect to `nats://localhost:4222`.

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
        print(f"Received [{msg.subject}]: {data}")
        received.append(data)

    sub = await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)
    await asyncio.sleep(1)  # wait for store-first delivery
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
        print(f"  msg_id={m['msg_id']} priority={m['priority']} ts={m['ts']}")

    # --- Delete a message ---
    if meta.get("messages"):
        msg_id = meta["messages"][0]["msg_id"]
        await nc.request(
            f"$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}",
            b"{}",
            timeout=5
        )
        print(f"Deleted message: {msg_id}")

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

**Queue group (competitive consumption):**

```python
import asyncio, nats, json

async def worker(worker_id: str):
    nc = await nats.connect("nats://localhost:4222")

    async def handler(msg):
        data = json.loads(msg.data)
        print(f"Worker {worker_id} got task: {data}")

    # All workers with the same queue group share load
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
        fmt.Printf("Received [%s]: %s\n", msg.Subject, string(msg.Data))
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

    // --- Queue group (competitive consumption) ---
    qsub, _ := nc.QueueSubscribe("$mq9.AI.MAILBOX.MSG.task.queue.*", "workers",
        func(msg *nats.Msg) {
            fmt.Printf("Worker got: %s\n", string(msg.Data))
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
      console.log(`Received [${msg.subject}]:`, JSON.parse(sc.decode(msg.data)));
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

    // --- Create a private mailbox ---
    let payload = serde_json::json!({"ttl": 3600}).to_string().into_bytes();
    let response = client
        .request("$mq9.AI.MAILBOX.CREATE".to_string(), payload.into())
        .await?;
    let create_resp: serde_json::Value = serde_json::from_slice(&response.payload)?;
    let mail_id = create_resp["mail_id"].as_str().unwrap();
    println!("Mailbox created: {}", mail_id);

    // --- Send messages ---
    let task = serde_json::json!({"type": "task", "payload": "process dataset A"})
        .to_string()
        .into_bytes();
    client
        .publish(
            format!("$mq9.AI.MAILBOX.MSG.{}", mail_id),
            task.into(),
        )
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
        println!("Received [{}]: {}", msg.subject, data);
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

    // --- Queue group ---
    let mut qsub = client
        .queue_subscribe(
            "$mq9.AI.MAILBOX.MSG.task.queue.*".to_string(),
            "workers".to_string(),
        )
        .await?;
    // Process messages from qsub in a separate task...

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
import io.nats.client.api.*;
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

            // --- Send messages ---
            byte[] taskPayload = mapper.writeValueAsBytes(
                    Map.of("type", "task", "payload", "process dataset A"));
            nc.publish("$mq9.AI.MAILBOX.MSG." + mailId, taskPayload);

            byte[] abortPayload = mapper.writeValueAsBytes(
                    Map.of("type", "abort", "task_id", "t-001"));
            nc.publish("$mq9.AI.MAILBOX.MSG." + mailId + ".critical", abortPayload);

            // --- Subscribe ---
            Dispatcher dispatcher = nc.createDispatcher(msg -> {
                try {
                    Map<?, ?> data = mapper.readValue(msg.getData(), Map.class);
                    System.out.println("Received [" + msg.getSubject() + "]: " + data);
                } catch (Exception e) {
                    e.printStackTrace();
                }
            });
            dispatcher.subscribe("$mq9.AI.MAILBOX.MSG." + mailId + ".*");
            Thread.sleep(500);
            dispatcher.unsubscribe("$mq9.AI.MAILBOX.MSG." + mailId + ".*");

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

// --- Create a private mailbox ---
var createPayload = JsonSerializer.SerializeToUtf8Bytes(new { ttl = 3600 });
var createResp = await nats.RequestAsync<byte[], JsonElement>(
    "$mq9.AI.MAILBOX.CREATE", createPayload);
var mailId = createResp.Data.GetProperty("mail_id").GetString()!;
Console.WriteLine($"Mailbox created: {mailId}");

// --- Send messages ---
var taskPayload = JsonSerializer.SerializeToUtf8Bytes(
    new { type = "task", payload = "process dataset A" });
await nats.PublishAsync($"$mq9.AI.MAILBOX.MSG.{mailId}", taskPayload);

var abortPayload = JsonSerializer.SerializeToUtf8Bytes(
    new { type = "abort", task_id = "t-001" });
await nats.PublishAsync($"$mq9.AI.MAILBOX.MSG.{mailId}.critical", abortPayload);

// --- Subscribe ---
await foreach (var msg in nats.SubscribeAsync<byte[]>($"$mq9.AI.MAILBOX.MSG.{mailId}.*"))
{
    var data = JsonSerializer.Deserialize<JsonElement>(msg.Data!);
    Console.WriteLine($"Received [{msg.Subject}]: {data}");
    break; // for demo, stop after first message
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

For full protocol details, see [Protocol](./Protocol.md).
