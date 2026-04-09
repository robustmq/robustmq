# SDK Integration

mq9 can be used with any NATS client directly — no SDK is required. The RobustMQ SDK provides typed wrappers with async patterns and consistent error handling across languages.

## Installation

| Language | Package | Install |
|----------|---------|---------|
| Python | `robustmq` (PyPI) | `pip install robustmq` |
| Go | `github.com/robustmq/robustmq-sdk/go` | `go get github.com/robustmq/robustmq-sdk/go` |
| JavaScript | `@robustmq/sdk` (npm) | `npm install @robustmq/sdk` |
| Java | `com.robustmq:robustmq` (Maven) | see Maven snippet below |
| Rust | `robustmq` (crates.io) | `cargo add robustmq` |
| C# | `RobustMQ` (NuGet) | `dotnet add package RobustMQ` |

## API Overview

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

## Python

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

## Go

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

## JavaScript (TypeScript)

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

## Java

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

## Rust

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

## C#

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

```
dotnet add package RobustMQ --version 0.3.5
```

---

## Using NATS Directly (No SDK)

Any standard NATS client works with mq9 out of the box. The following example uses the Python `nats-py` library to interact with mq9 subjects directly:

```python
import asyncio, nats, json

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # Create mailbox
    reply = await nc.request("$mq9.AI.MAILBOX.CREATE", json.dumps({"ttl": 3600}).encode())
    mail_id = json.loads(reply.data)["mail_id"]

    # Send
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.normal", b"hello")

    # Subscribe
    async def handler(msg):
        print(f"Got: {msg.data}")
    await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)

    await asyncio.sleep(1)
    await nc.close()

asyncio.run(main())
```

The same pattern applies to any NATS client in any language. See the [Protocol reference](./Protocol.md) for the full subject schema.
