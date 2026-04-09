# SDK 接入

mq9 可以直接使用任何 NATS 客户端——无需额外 SDK。RobustMQ SDK 提供了类型化封装，在各语言中提供一致的异步模式和错误处理。

## 安装

| 语言 | 包名 | 安装 |
|------|------|------|
| Python | `robustmq` (PyPI) | `pip install robustmq` |
| Go | `github.com/robustmq/robustmq-sdk/go` | `go get github.com/robustmq/robustmq-sdk/go` |
| JavaScript | `@robustmq/sdk` (npm) | `npm install @robustmq/sdk` |
| Java | `com.robustmq:robustmq` (Maven) | 见下方 Maven 片段 |
| Rust | `robustmq` (crates.io) | `cargo add robustmq` |
| C# | `RobustMQ` (NuGet) | `dotnet add package RobustMQ` |

## API 概览

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

## Python

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

## JavaScript（TypeScript）

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

## Rust

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

```
dotnet add package RobustMQ --version 0.3.5
```

---

## 直接使用 NATS 客户端（不使用 SDK）

任何标准 NATS 客户端都能直接与 mq9 配合使用。以下示例使用 Python `nats-py` 库直接操作 mq9 subject：

```python
import asyncio, nats, json

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # 创建邮箱
    reply = await nc.request("$mq9.AI.MAILBOX.CREATE", json.dumps({"ttl": 3600}).encode())
    mail_id = json.loads(reply.data)["mail_id"]

    # 发送
    await nc.publish(f"$mq9.AI.MAILBOX.MSG.{mail_id}.normal", b"hello")

    # 订阅
    async def handler(msg):
        print(f"收到: {msg.data}")
    await nc.subscribe(f"$mq9.AI.MAILBOX.MSG.{mail_id}.*", cb=handler)

    await asyncio.sleep(1)
    await nc.close()

asyncio.run(main())
```

相同模式适用于任意语言的任意 NATS 客户端。完整 subject 规范请参见[协议设计](./Protocol.md)。
