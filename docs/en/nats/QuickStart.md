# Quick Start

Get connected to RobustMQ NATS in 5 minutes: publish, subscribe, and request-reply.

## Prerequisites

- RobustMQ running with NATS port `4222` (default)
- [NATS CLI](https://github.com/nats-io/natscli) installed (optional, for verification)

## Start RobustMQ

```bash
curl -fsSL https://raw.githubusercontent.com/robustmq/robustmq/main/scripts/install.sh | bash
robust-server start
```

## Verify the Connection (NATS CLI)

```bash
# Subscribe (terminal 1)
nats sub "hello.>" --server nats://localhost:4222

# Publish (terminal 2)
nats pub "hello.world" "Hello RobustMQ!" --server nats://localhost:4222
```

If terminal 1 receives the message, the connection is working.

---

## Quick Start by Language

Pick your language and run a complete flow in 5 minutes.

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

    // Subscribe
    nc.Subscribe("hello.>", func(m *nats.Msg) {
        fmt.Printf("Received: %s\n", m.Data)
    })

    // Publish
    nc.Publish("hello.world", []byte("Hello RobustMQ!"))

    // Request-Reply
    msg, _ := nc.Request("hello.query", []byte("ping"), 2*time.Second)
    fmt.Printf("Reply: %s\n", msg.Data)

    time.Sleep(time.Second)
}
```

→ More examples: [nats.go official docs](https://github.com/nats-io/nats.go)

### Python

```bash
pip install nats-py
```

```python
import asyncio
import nats

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # Subscribe
    async def handler(msg):
        print(f"Received: {msg.data.decode()}")
    await nc.subscribe("hello.>", cb=handler)

    # Publish
    await nc.publish("hello.world", b"Hello RobustMQ!")

    # Request-Reply
    reply = await nc.request("hello.query", b"ping", timeout=2)
    print(f"Reply: {reply.data.decode()}")

    await asyncio.sleep(1)
    await nc.close()

asyncio.run(main())
```

→ More examples: [nats.py official docs](https://github.com/nats-io/nats.py)

### JavaScript / Node.js

```bash
npm install nats
```

```javascript
import { connect, StringCodec } from "nats";

const nc = await connect({ servers: "nats://localhost:4222" });
const sc = StringCodec();

// Subscribe
const sub = nc.subscribe("hello.>");
(async () => {
    for await (const msg of sub) {
        console.log(`Received: ${sc.decode(msg.data)}`);
    }
})();

// Publish
nc.publish("hello.world", sc.encode("Hello RobustMQ!"));

// Request-Reply
const reply = await nc.request("hello.query", sc.encode("ping"), { timeout: 2000 });
console.log(`Reply: ${sc.decode(reply.data)}`);

await nc.close();
```

→ More examples: [nats.js official docs](https://github.com/nats-io/nats.js)

### Java

```xml
<dependency>
    <groupId>io.nats</groupId>
    <artifactId>jnats</artifactId>
    <version>2.20.5</version>
</dependency>
```

```java
Connection nc = Nats.connect("nats://localhost:4222");

// Subscribe
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("Received: " + new String(msg.getData()));
});
d.subscribe("hello.>");

// Publish
nc.publish("hello.world", "Hello RobustMQ!".getBytes());

// Request-Reply
Message reply = nc.request("hello.query", "ping".getBytes(), Duration.ofSeconds(2));
System.out.println("Reply: " + new String(reply.getData()));

nc.close();
```

→ More examples: [jnats official docs](https://github.com/nats-io/jnats)

### Rust

```toml
[dependencies]
async-nats = "0.37"
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Subscribe
    let mut sub = client.subscribe("hello.>").await?;

    // Publish
    client.publish("hello.world", "Hello RobustMQ!".into()).await?;

    // Receive
    if let Some(msg) = sub.next().await {
        println!("Received: {:?}", msg.payload);
    }

    Ok(())
}
```

→ More examples: [nats.rs official docs](https://github.com/nats-io/nats.rs)

---

## Next Steps

- [NATS Core](./NatsCore.md) — Wildcards, Queue Groups, Headers, and full protocol reference
- [SDK Integration](./SDK.md) — SDKs and official docs for all languages
- [JetStream](./JetStream.md) — Persistent message streams (in development)
- [mq9](../mq9/Overview.md) — AI Agent communication protocol built on NATS
