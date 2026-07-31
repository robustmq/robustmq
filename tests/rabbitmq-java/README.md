# RabbitMQ Java-client integration tests

End-to-end tests that drive RobustMQ's AMQP 0-9-1 protocol with the official
RabbitMQ Java client (`amqp-client`). They assume a RobustMQ broker is already
running (AMQP on `127.0.0.1:5672`, matching `scripts/cluster.sh`'s config).

## Run

```bash
# from the repo root
make rabbitmq-test

# against a specific client version (multi-version SDK coverage)
make rabbitmq-test RABBITMQ_CLIENT_VERSION=5.20.0

# or directly
cd tests/rabbitmq-java && mvn test
```

Overridable connection settings (system property or env var):

- `-Damqp.host=host` (or env `AMQP_BROKER_HOST`, default `127.0.0.1`)
- `-Damqp.port=port` (or env `AMQP_BROKER_PORT`, default `5672`)
- `-Damqp.user=user` (or env `AMQP_BROKER_USER`, default `admin`)
- `-Damqp.password=pass` (or env `AMQP_BROKER_PASSWORD`, default `robustmq`)

## Layout

The test code is version-agnostic; the RabbitMQ client version is the Maven
property `rabbitmq.client.version` (default: latest stable), overridable per
run. A single source tree therefore covers every client version — only the
property changes. Other-language SDK suites live in sibling directories (e.g.
`tests/kafka-java/`, a future `tests/rabbitmq-go/`).

Tests are grouped by protocol stage, in the order they exercise the broker:

- `ConnectionTest` — the Connection/Channel handshake (login, vhost,
  multiple channels, clean close) that every test below depends on.
- `DeclareSemanticsTest` — passive declare semantics and real
  (non-zero-stub) Declare/Delete accounting: `passive` on a missing/existing
  exchange, real `message_count`/`consumer_count` on `queue.declare`,
  `Queue.Purge`, the `if-unused`/`if-empty` failure paths on
  `Exchange.Delete`/`Queue.Delete`, and their success paths (including that
  a deleted queue's underlying shard doesn't resurrect old messages on
  redeclare — see the note on eventual consistency below).
- `PublishTest` — `Basic.Publish` routing: default exchange, a named direct
  exchange, `mandatory` returns, all four exchange types (direct/fanout/
  topic/headers), exchange-to-exchange binding chains (`Exchange.Bind`/
  `Exchange.Unbind`), and content/property round-tripping.
- `BasicGetTest` — the Java-client counterpart of the broker's own Rust
  `basic_get_test.rs` (lapin-based) suite: same scenarios (get/ack/nack
  requeue/no_ack/delivery-tag ordering, including `multiple=true` batch
  ack/nack semantics), driven by a different client implementation to
  confirm wire-protocol compatibility.
- `ConsumeTest` — single-queue, single-consumer `Basic.Consume`: delivery
  order, `no-ack`, the shared Get/Consume cursor, and property round-trip.
- `MultiConsumerTest` — multiple queues and multiple competing consumers on
  one queue (round-robin fan-out, `Basic.Cancel`).
- `ReliabilityTest` — `Confirm.Select` acks, `Basic.Qos` prefetch, and
  exclusive `Basic.Consume`. `Channel.Flow` has no coverage here: the
  RabbitMQ Java client (5.21.0+) dropped the client-side `Channel.flow()`
  API entirely, so it can't be driven from this suite — see the Rust unit
  tests instead.
- `MultiNodeTest` — runs against a real 3-node cluster
  (`scripts/cluster.sh`, ports 5672/5772/5872 by default; override with
  `-Damqp.node{1,2,3}Port`). `Basic.Get`/`Consume`/`Queue.Declare` must work
  correctly regardless of which node a connection lands on relative to the
  queue's elected leader.

**A note on eventual consistency**: metadata reads (`message_count`,
`consumer_count` on `queue.declare`) and the physical teardown of a
deleted queue's storage shard are asynchronous across the Raft-backed
cluster — deleting a queue and immediately redeclaring it under the same
name can transiently still see the old queue's messages for up to a few
seconds while the shard is torn down in the background. Tests that depend
on this poll with a bounded timeout (`Support.awaitCount`, `awaitMessageCount`,
`assertEventuallyEmpty`) rather than asserting immediately.
