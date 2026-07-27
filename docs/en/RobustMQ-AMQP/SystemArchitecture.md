# System Architecture

RobustMQ's AMQP capability isn't a standalone message queue service — it's an **AMQP 0-9-1 protocol compatibility layer** built on top of the RobustMQ unified kernel. It reuses RobustMQ's networking framework, the File Segment storage engine, and the Raft-based metadata service, presenting itself externally as a standard AMQP 0-9-1 broker that official RabbitMQ clients can connect to directly.

Its most important design choice is **reusing the shared consume-group infrastructure**: an AMQP queue's "competing consumers" model is the same mechanism as MQTT/NATS shared subscriptions, so AMQP doesn't implement a separate queue engine — instead, every queue maps to a shared consume group, driven by the cluster-elected leader node responsible for pulling and delivering that queue's messages.

![RobustMQ AMQP System Architecture](../../images/amqp-architecture.svg)

## Layered Architecture

From top to bottom, there are five layers:

### 1. Client Layer

Any standard AMQP 0-9-1 client can connect: Java `com.rabbitmq:amqp-client`, Python `pika`, Node.js `amqplib`, and others. Clients connect via the standard `Connection.Start`/`Tune`/`Open` handshake with no RobustMQ-specific extensions required.

### 2. Protocol Layer (`amq-protocol` + `src/amqp-broker/src/handler`)

Encoding/decoding of the AMQP 0-9-1 wire protocol (Method / Content Header / Content Body / Heartbeat frame types) is built on the `amq-protocol` crate. Key points:

- **Handler dispatch** (`handler/command.rs`): routes requests to the corresponding processing module by `AMQPClass` (Connection / Channel / Exchange / Queue / Basic / Tx / Confirm).
- **Atomic multi-frame writes**: a single `Basic.Deliver`/`Basic.Get-Ok` requires writing three consecutive frames — Method + Header + Body. When multiple concurrent queue-push tasks write to the same connection, the write must hold the same connection lock across all three frames, otherwise frames from different deliveries interleave on the TCP stream and corrupt the client's parsing.

### 3. Core Handling Layer (`src/amqp-broker/src/amqp`)

Where AMQP semantics are implemented, organized by AMQP class:

- **connection / channel**: handshake, SASL PLAIN authentication, vhost→tenant mapping, channel lifecycle.
- **exchange / queue**: declare (including passive semantics), delete (`if-unused`/`if-empty`), bind/unbind; metadata is written to Raft and synced to the local cache.
- **publish**: `Basic.Publish` routing (default-exchange direct-by-name plus the four exchange-type matchers), `mandatory` returns.
- **consume**: `Basic.Get` (pull) and `Basic.Consume` (registering a shared consume group member).
- **basic**: `Ack`/`Nack`/`Reject`/`Recover`/`Qos`/`Confirm.Select` — message state and reliability semantics.

### 4. Push & Coordination Layer (`src/amqp-broker/src/push` + `src/amqp-broker/src/core`)

The key layer that realizes AMQP's queue model as a shared consume group:

- **`AmqpPushManager` / `AmqpQueuePush`**: each queue's push task round-robins messages to the group's members (consumers), honoring the prefetch window set by `Basic.Qos`.
- **Leader election and resolution** (`resolve_queue_leader`): the first time a queue is accessed, meta-service creates/looks up its shared consume group and returns the leader node; the result comes directly from the create/lookup call itself, with no extra read-back needed to confirm it.
- **Cross-node forwarding**: a `Basic.Get` landing on a non-leader node is forwarded to the leader via one gRPC call (`FetchAmqpQueueMessage`); `Basic.Consume` push delivery similarly uses `SendShareGroupMessage` to deliver a message to whichever node the consumer's actual connection lives on.

### 5. Storage Layer (File Segment Engine)

Accessed via `StorageDriverManager`, shared with Kafka and MQTT:

- Every AMQP queue maps to an internal topic whose physical shard is append-written in segments.
- Consume position (acked/unacked) is tracked via a separate offset store and an unacked index, backing `Basic.Ack`/`Nack`/`Recover` semantics.

### 6. Metadata Layer (meta-service · Raft)

Cluster metadata is managed by the Raft-based meta-service:

- **Exchange / Queue / Binding**: declared metadata is persisted in Raft and replicated to every node, surviving restarts.
- **Shared consume group (ShareGroup / ShareGroupMember)**: AMQP queues reuse the existing shared consume-group data structures and election logic already built for MQTT/NATS, rather than maintaining a separate queue table.
- **Dynamic config**: for creation operations (e.g. the first `Queue.Declare` that creates the shared group), the leader node handling the request returns the authoritative result directly — avoiding the "write, then immediately read back stale state" race.

## Request Flow

- **Declarative operations (Exchange/Queue declare/delete/bind)**: protocol layer decodes → core layer processes → write to Raft metadata → update local cache → return an ack frame.
- **Publish (`Basic.Publish`)**: protocol layer decodes → routing match (exchange type + bindings) → write to the target queue's storage shard.
- **Pull consumption (`Basic.Get`)**: the core layer resolves the queue's leader — if it's the local node, read the next message directly from storage; otherwise forward via gRPC to the leader.
- **Push consumption (`Basic.Consume`)**: the push task on the queue's leader node continuously polls storage and round-robins delivery to whichever group member currently has spare prefetch quota.

## Key Differences from Native RabbitMQ

| Dimension | Native RabbitMQ | RobustMQ |
|---|---|---|
| Queue engine | Erlang processes + Mnesia | Shared consume group + File Segment storage engine |
| Queue coordinator | The node the queue lives on (mirrored/quorum queues have their own leader election) | The shared consume group's leader, elected by meta-service |
| Multi-protocol | AMQP only (needs Shovel/Federation plugins to cross protocols) | AMQP / Kafka / MQTT share the same data |
| Management HTTP API | Built-in `rabbitmqadmin` / management plugin | No separate management API yet; managed via the AMQP protocol itself |
| Transactions / dead-letter queues | Supported | Not yet supported (see [Compatibility & Limitations](./Compatibility-and-Limitations.md)) |

> See the [Protocol Compatibility Matrix](./Protocol.md) for per-method support status.
