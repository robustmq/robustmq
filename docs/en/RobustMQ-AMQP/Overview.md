# Overview

RobustMQ AMQP is an **AMQP 0-9-1 protocol compatibility layer** built on top of the RobustMQ unified kernel — it is not a standalone RabbitMQ distribution, but a protocol implementation that lets the standard AMQP 0-9-1 client ecosystem connect directly to RobustMQ. Official RabbitMQ clients (Java `amqp-client`, `pika`, `amqplib`, etc.) can connect directly, on the default port `5672`.

## Design choice: reuse the shared consume-group model instead of building a separate one

The core design choice behind RobustMQ AMQP is to **reuse RobustMQ's existing shared consume-group infrastructure** rather than implementing a dedicated queue engine just for AMQP. AMQP's "one queue, many competing consumers" model is, at its core, the same problem as MQTT/NATS shared subscriptions: a group of consumers competing to consume the same data, coordinated by one node. As a result:

- **A queue is a shared consume group**: each AMQP queue internally maps to a shared consume group; consumers registered via `Basic.Consume` are members of that group.
- **The group leader is elected by meta-service**: message pulling and delivery for a queue is driven by the leader node the cluster elected based on load; a `Basic.Get` on a non-leader node is forwarded to the leader via a single gRPC round trip.
- **Exchange / Queue / Binding metadata is persisted in Raft**: declared exchanges, queues, and bindings are cluster-wide metadata replicated through meta-service's Raft layer — they survive restarts and are visible from any node.
- **Message storage is the File Segment engine**: AMQP shares the same underlying storage as Kafka and MQTT (append-only segments, offset index); each AMQP queue maps to an internal topic/shard.

See [System Architecture](./SystemArchitecture.md) for details.

## Capability Overview

| Capability | Status | Notes |
|---|---|---|
| Connection / Channel handshake | ✅ | SASL PLAIN authentication, `Tune`/`TuneOk` negotiation |
| Exchange management | ✅ | Four types — `direct` / `fanout` / `topic` / `headers`; declare / delete / bind / unbind |
| Queue management | ✅ | declare (incl. passive) / delete (if-unused / if-empty) / bind / unbind / purge |
| Publish & routing | ✅ | `Basic.Publish`; default-exchange direct-by-name routing plus the four exchange-type matchers; `mandatory` returns |
| Pull consumption (`Basic.Get`) | ✅ | Single-message pull, auto-forwarded across nodes to the queue leader |
| Push consumption (`Basic.Consume`) | ✅ | Competing-consumer model, driven by the shared consume group's leader push |
| Message acknowledgement | ✅ | `Basic.Ack` / `Nack` / `Reject` / `Recover` (requeue / discard) |
| Publisher confirms | ✅ | After `Confirm.Select`, every publish gets an ack/nack matching its durability outcome |
| QoS prefetch | 🟡 | Enforced when the consumer is co-located with the queue leader; best-effort across nodes |
| Channel.Flow | 🟡 | Only takes effect for push delivery on the local node |
| Transactions (Tx class) | ❌ | Handshake works; `Tx.Commit`/`Rollback` don't implement real transactional semantics |
| Dead-letter queues / message TTL | ❌ | Not supported |

> See the [Protocol Compatibility Matrix](./Protocol.md) for per-method support status, and [Compatibility & Limitations](./Compatibility-and-Limitations.md) for the full supported / partial / unsupported list with root causes.

## Quick Start

After starting a single node, use the official RabbitMQ Java client to declare a queue, publish, and consume:

```java
ConnectionFactory factory = new ConnectionFactory();
factory.setHost("localhost");
factory.setPort(5672);
try (Connection connection = factory.newConnection();
     Channel channel = connection.createChannel()) {
    channel.queueDeclare("quickstart", true, false, false, null);
    channel.basicPublish("", "quickstart", null, "hello".getBytes());
    GetResponse resp = channel.basicGet("quickstart", true);
    System.out.println(new String(resp.getBody()));
}
```

See [Quick Start](./QuickStart.md) for the full walkthrough, including multi-language client examples and verifying the shared queue group.

## Documentation Map

| Document | Content |
|---|---|
| [Core Concepts](./AMQPCoreConcepts.md) | Connection / Channel / Exchange / Queue / Binding / shared consume group |
| [System Architecture](./SystemArchitecture.md) | Layered architecture, request flow, key differences from native RabbitMQ |
| [Protocol Compatibility Matrix](./Protocol.md) | Per-method support status and notes |
| [Quick Start](./QuickStart.md) | Single-node startup, minimal Java client example |
| [Compatibility & Limitations](./Compatibility-and-Limitations.md) | Supported / partial / unsupported list with root causes |
