# NATS Core: Features and Usage

## Protocol Basics

NATS Core uses a pure text protocol over TCP. Each command ends with `\r\n` (CRLF), fields are separated by whitespace. Commands are case-insensitive; Subject names are case-sensitive.

### Connection Handshake

```
Server → Client:  INFO {...}\r\n
Client → Server:  CONNECT {...}\r\n
Client → Server:  PING\r\n
Server → Client:  PONG\r\n
```

The server sends `INFO` immediately on accept. The client replies with `CONNECT` to complete authentication and capability negotiation. The client then sends `PING` and waits for `PONG` — once received, the connection is ready for Pub/Sub operations.

### Subject Naming Rules

A Subject is the NATS addressing unit:

- Composed of letters, digits, `.`, `-`, `_`
- Case-sensitive: `foo.bar` and `Foo.Bar` are different subjects
- `.` is the hierarchy separator: `orders.us.created`
- Cannot start or end with `.`
- Cannot contain spaces

### Wildcards

| Wildcard | Description | Example |
|----------|-------------|---------|
| `*` | Matches a single hierarchy level | `orders.*.created` matches `orders.us.created`, not `orders.us.east.created` |
| `>` | Matches one or more levels; only valid at the end | `orders.>` matches `orders.us`, `orders.us.created`, `orders.us.east.created` |

---

## Core Commands

### PUB — Publish a Message

```
PUB <subject> [reply-to] <#bytes>\r\n[payload]\r\n
```

| Parameter | Description |
|-----------|-------------|
| `subject` | Target subject |
| `reply-to` | Optional reply address (for Request-Reply pattern) |
| `#bytes` | Payload byte count |

Examples:

```bash
# Publish to orders.created
nats pub orders.created '{"order_id":"001","amount":100}'

# Publish with explicit reply address (manual request-reply)
nats pub orders.query '{"id":"001"}' --reply orders.response.tmp
```

### SUB — Subscribe

```
SUB <subject> [queue group] <sid>\r\n
```

| Parameter | Description |
|-----------|-------------|
| `subject` | Subject to subscribe to; wildcards supported |
| `queue group` | Optional queue group name for competing consumption |
| `sid` | Subscription ID, client-assigned identifier |

Examples:

```bash
# Subscribe to a single subject
nats sub orders.created

# Wildcard subscription
nats sub "orders.*"
nats sub "orders.>"

# Queue Group subscription (competing consumers)
nats sub orders.created --queue order-processors
```

### UNSUB — Unsubscribe

```
UNSUB <sid> [max-msgs]\r\n
```

| Parameter | Description |
|-----------|-------------|
| `sid` | Subscription ID to cancel |
| `max-msgs` | Optional: auto-unsubscribe after receiving N more messages |

### HPUB — Publish a Message With Headers

Requires server to support `headers: true` (declared in INFO).

```
HPUB <subject> [reply-to] <#header bytes> <#total bytes>\r\n
[headers]\r\n\r\n[payload]\r\n
```

Headers use HTTP/1 format:

```
NATS/1.0\r\n
Key1: Value1\r\n
Key2: Value2\r\n
\r\n
```

Example:

```bash
nats pub --header "Content-Type:application/json" \
         --header "X-Trace-ID:abc123" \
         orders.created '{"order_id":"001"}'
```

---

## Pub/Sub

The most fundamental communication pattern: publishers don't know who is subscribed, subscribers don't know who published.

```bash
# Terminal 1: Subscribe
nats sub "sensor.temperature.>"

# Terminal 2: Publish
nats pub sensor.temperature.room1 '{"value":22.5,"unit":"celsius"}'
nats pub sensor.temperature.room2 '{"value":24.1,"unit":"celsius"}'
```

**Characteristics:**
- All currently online subscribers receive every message
- Messages published with no active subscriber are dropped immediately (at-most-once)
- No setup required — publish and subscribe directly

---

## Request-Reply

Synchronous request-response pattern, implemented internally via a temporary reply-to subject.

```bash
# Server side: listen for requests and reply
nats reply orders.query '{"status":"ok","result":{"id":"001"}}'

# Client side: send request and wait for reply (default 2s timeout)
nats request orders.query '{"id":"001"}'
```

**How it works:**
1. Client publishes with an auto-generated temporary reply-to subject (e.g. `_INBOX.abc123`)
2. Server receives the message and publishes the response to the reply-to subject
3. Client waits for a message on the reply-to subject

**Java example:**

```java
Connection nc = Nats.connect("nats://localhost:4222");

// Server side (handle requests)
Dispatcher d = nc.createDispatcher((msg) -> {
    String request = new String(msg.getData());
    String response = processRequest(request);
    nc.publish(msg.getReplyTo(), response.getBytes());
});
d.subscribe("orders.query");

// Client side (send request)
Message reply = nc.request("orders.query",
    "{\"id\":\"001\"}".getBytes(),
    Duration.ofSeconds(2));
System.out.println("Response: " + new String(reply.getData()));
```

---

## Queue Groups (Competing Consumers)

Multiple subscribers in the same Queue Group receive messages in round-robin: each message is delivered to exactly one subscriber in the group, enabling load balancing.

```bash
# Start multiple workers in the same Queue Group
nats sub orders.created --queue order-processors  # Worker 1
nats sub orders.created --queue order-processors  # Worker 2
nats sub orders.created --queue order-processors  # Worker 3

# Publish messages (only one worker receives each)
nats pub orders.created '{"order_id":"001"}'
nats pub orders.created '{"order_id":"002"}'
nats pub orders.created '{"order_id":"003"}'
```

**Characteristics:**
- Within a Queue Group, each message is delivered to exactly one subscriber
- Workers can be added or removed at any time; NATS adjusts automatically
- Different Queue Groups are independent — each group receives all messages

**Java example:**

```java
// Three workers competing via Queue Group
for (int i = 1; i <= 3; i++) {
    final int id = i;
    Dispatcher worker = nc.createDispatcher((msg) -> {
        System.out.println("[Worker-" + id + "] processing: " + new String(msg.getData()));
    });
    worker.subscribe("orders.created", "order-processors");
}
```

---

## Connection and Authentication

### Connection Options

```bash
# Connect to a specific server
nats sub "test.>" --server nats://localhost:4222

# Username/password authentication
nats sub "test.>" --server nats://user:password@localhost:4222

# Token authentication
nats sub "test.>" --server nats://mytoken@localhost:4222

# TLS
nats sub "test.>" --server nats://localhost:4222 --tlscert client.crt --tlskey client.key
```

### CONNECT Command Fields

The client sends a `CONNECT` JSON payload on connection. Common fields:

| Field | Type | Description |
|-------|------|-------------|
| `verbose` | bool | Return `+OK` acknowledgment for every command |
| `pedantic` | bool | Enable strict mode (validate subject names, etc.) |
| `tls_required` | bool | Require TLS connection |
| `name` | string | Client name, useful for debugging |
| `lang` | string | Client language: `go`, `java`, `python`, etc. |
| `version` | string | Client version |
| `user` | string | Username (username/password auth) |
| `pass` | string | Password |
| `auth_token` | string | Token authentication |
| `headers` | bool | Whether the client supports message headers |

---

## Keepalive and Heartbeat

NATS maintains connection liveness via PING/PONG. The server periodically sends `PING`; the client must respond with `PONG`. Clients can also send `PING` proactively to check connection health.

```
Server → Client: PING\r\n
Client → Server: PONG\r\n
```

If the client fails to respond within the configured timeout, the server closes the connection. Client SDKs handle PING/PONG automatically — no manual management required.

---

## Error Handling

Server error format:

```
-ERR '<error message>'\r\n
```

Common errors:

| Error | Description |
|-------|-------------|
| `'Unknown Protocol Operation'` | Unrecognized command received |
| `'Attempted To Connect To Route Port'` | Client connected to the cluster routing port |
| `'Authorization Violation'` | Authentication failed |
| `'Authorization Timeout'` | Authentication timed out |
| `'Invalid Client Protocol'` | Protocol version incompatible |
| `'Maximum Control Line Exceeded'` | Control line exceeds maximum length |
| `'Parser Error'` | Protocol parse error |
| `'Secure Connection - TLS Required'` | TLS connection required |
| `'Stale Connection'` | Connection expired (PING/PONG timeout) |
| `'Maximum Connections Exceeded'` | Server connection limit reached |
| `'Slow Consumer'` | Consumer processing too slowly; buffer overflowed |
| `'Maximum Payload Violation'` | Payload exceeds maximum size |
| `'Invalid Subject'` | Subject format is illegal |
| `'Permissions Violation'` | Insufficient permissions to publish or subscribe |

---

## SDK Quick Start

RobustMQ is compatible with the standard NATS protocol. Use any official NATS client SDK to connect directly — no modifications needed.

### Go

```go
import "github.com/nats-io/nats.go"

nc, _ := nats.Connect("nats://localhost:4222")

// Publish
nc.Publish("orders.created", []byte(`{"order_id":"001"}`))

// Subscribe
nc.Subscribe("orders.>", func(m *nats.Msg) {
    fmt.Printf("Received: %s\n", m.Data)
})

// Queue Group
nc.QueueSubscribe("orders.created", "processors", func(m *nats.Msg) {
    fmt.Printf("Worker received: %s\n", m.Data)
})

// Request-Reply
msg, _ := nc.Request("orders.query", []byte(`{"id":"001"}`), 2*time.Second)
fmt.Printf("Reply: %s\n", msg.Data)
```

### Python

```python
import asyncio
import nats

async def main():
    nc = await nats.connect("nats://localhost:4222")

    # Publish
    await nc.publish("orders.created", b'{"order_id":"001"}')

    # Subscribe
    async def handler(msg):
        print(f"Received: {msg.data.decode()}")
    await nc.subscribe("orders.>", cb=handler)

    # Queue Group
    await nc.subscribe("orders.created", queue="processors", cb=handler)

    # Request-Reply
    reply = await nc.request("orders.query", b'{"id":"001"}', timeout=2)
    print(f"Reply: {reply.data.decode()}")

asyncio.run(main())
```

### Java

```java
// Dependency: io.nats:jnats:2.20.5
Connection nc = Nats.connect("nats://localhost:4222");

// Publish
nc.publish("orders.created", "{\"order_id\":\"001\"}".getBytes());

// Subscribe
Dispatcher d = nc.createDispatcher((msg) -> {
    System.out.println("Received: " + new String(msg.getData()));
});
d.subscribe("orders.>");

// Queue Group
d.subscribe("orders.created", "processors");

// Request-Reply
Message reply = nc.request("orders.query",
    "{\"id\":\"001\"}".getBytes(), Duration.ofSeconds(2));
System.out.println("Reply: " + new String(reply.getData()));
```

### JavaScript (Node.js)

```javascript
import { connect, StringCodec } from "nats";

const nc = await connect({ servers: "nats://localhost:4222" });
const sc = StringCodec();

// Publish
nc.publish("orders.created", sc.encode('{"order_id":"001"}'));

// Subscribe
const sub = nc.subscribe("orders.>");
(async () => {
    for await (const msg of sub) {
        console.log(`Received: ${sc.decode(msg.data)}`);
    }
})();

// Request-Reply
const reply = await nc.request("orders.query",
    sc.encode('{"id":"001"}'), { timeout: 2000 });
console.log(`Reply: ${sc.decode(reply.data)}`);
```

---

## Relationship to mq9

NATS Core is the underlying protocol for mq9. mq9 builds on NATS Core pub/sub/req-reply, adding persistence, priority queues, and TTL management via the `$mq9.AI.*` subject namespace — purpose-built for AI Agent asynchronous communication.

Both can be used together: regular NATS pub/sub for real-time scenarios, mq9 subjects for Agent communication that requires offline delivery guarantees.

See [mq9 Protocol](/en/mq9/Protocol) for details.
