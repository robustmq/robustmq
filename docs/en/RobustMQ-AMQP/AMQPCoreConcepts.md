# Core Concepts

This document explains the core concepts in RobustMQ AMQP and how each one is implemented on top of the RobustMQ unified kernel. If you're familiar with native RabbitMQ / AMQP 0-9-1, these concepts are identical; differences are called out explicitly.

## Connection and Channel

A **Connection** is a long-lived TCP connection carrying SASL authentication and vhost (tenant) selection. A **Channel** is a logical, multiplexed sub-connection over one TCP connection — almost every business method (Exchange/Queue/Basic) executes on some channel.

- Handshake: `Connection.Start` → `Start-Ok` (SASL PLAIN credentials) → `Tune`/`Tune-Ok` (negotiate channel-max, frame-max, heartbeat) → `Open`/`Open-Ok` (select vhost).
- AMQP's `virtual-host` maps to a RobustMQ **tenant**: the vhost name passed to `Connection.Open` is resolved to a tenant, with an empty string mapping to the default tenant.
- After `Channel.Open`, that channel's `Basic.Deliver`/`Basic.Get-Ok` delivery-tag starts at 1 and increases monotonically, never repeating for the channel's lifetime.

## Exchange

An exchange is the entry point for message routing — `Basic.Publish` always targets an exchange first, which then forwards the message to matching queues based on its type. RobustMQ supports all four standard types:

| Type | Routing rule |
|---|---|
| `direct` | Routes only when the routing-key **exactly equals** the binding-key |
| `fanout` | Ignores the routing-key, broadcasts to every bound queue |
| `topic` | Matches the routing-key against the binding pattern by `.`-separated segments (`*` matches one segment, `#` matches zero or more) |
| `headers` | Ignores the routing-key, matches on header key/value pairs declared at bind time (`x-match: all` / `any`) |

- The **default exchange** (empty-string name `""`) is implicit: every queue is automatically bound to the default exchange under its own name, so `Basic.Publish("", queue_name, ...)` delivers directly to the same-named queue with no explicit declare or bind needed.
- **Exchange-to-exchange bindings** (`Exchange.Bind`) support chained routing: an exchange can be bound to another exchange as a destination, and messages continue routing along the binding chain, with cycle protection built in.

## Queue and the Shared Consume Group

A queue is where messages ultimately land, and it's the **most important concept mapping** in RobustMQ AMQP: every queue internally maps to a [shared consume group](./SharedQueueGroup.md).

- `Queue.Declare` creates the queue's metadata (persisted in Raft) as well as the underlying storage shard that holds its messages.
- A queue doesn't need to pre-register a consume group — the first time a consumer calls `Basic.Consume` (or `Basic.Get`), RobustMQ creates the shared consume group on demand and elects the cluster node that will lead that queue.
- Multiple consumers (possibly connected to different nodes) calling `Basic.Consume` on the same queue are simply multiple members of that shared consume group — exactly AMQP's **competing consumers** semantics: each message is delivered to only one of them.

## Binding

A binding associates an exchange with a queue (or an exchange with another exchange), carrying a routing-key (plus header-match arguments for the `headers` type). The same queue can be bound to multiple exchanges, or multiple times to the same exchange with different routing-keys.

## Publishing and Delivery

- **Publish**: `Basic.Publish` carries the exchange and routing-key, immediately followed by a Content Header (properties) frame and a Content Body (payload) frame.
- **Pull consumption**: `Basic.Get` is a one-shot synchronous pull. If the queue's leader is on a different node, the request is forwarded internally via one gRPC round trip, transparent to the client.
- **Push consumption**: once registered via `Basic.Consume`, the queue leader node's push task actively delivers messages via `Basic.Deliver`, honoring the prefetch window set by `Basic.Qos` (see [Consuming](./Consuming.md)).

## Acknowledgement and the Unacked Index

A message delivered without `no-ack` enters the "unacked" state, tracked by an unacked index that records which connection/channel a given message was delivered to.

- `Basic.Ack` removes the corresponding record from storage.
- `Basic.Nack` / `Basic.Reject` (with `requeue=true`) or `Basic.Recover` puts the message back on the queue for redelivery; `requeue=false` discards it.
- When a connection or channel closes, every unacked message on it is automatically requeued.

See [Acknowledgement](./Acknowledgement.md) for details.

## Publisher Confirms

Once `Confirm.Select` is enabled, every `Basic.Publish` on that channel receives a matching `Basic.Ack` (success) or `Basic.Nack` (failure) once the message is actually durably written, correlated by the incrementing sequence number (delivery-tag) assigned at publish time. See [Publisher Confirms](./PublisherConfirms.md).

## Concept Mapping

| AMQP concept | RobustMQ implementation |
|---|---|
| Virtual Host | Tenant |
| Queue | Shared consume group + underlying storage shard |
| A `Basic.Consume` consumer | A shared consume group member |
| Queue pull/push coordinator | The group leader node elected by meta-service |
| Exchange / Queue / Binding metadata | Raft-replicated cluster metadata |
| Message storage | The File Segment engine (shared with Kafka / MQTT) |

## Further Reading

- [System Architecture](./SystemArchitecture.md)
- [Protocol Compatibility Matrix](./Protocol.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
