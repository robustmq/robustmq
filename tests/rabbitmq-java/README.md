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
- `PublishTest` — `Basic.Publish` routing: default exchange, a named direct
  exchange, `mandatory` returns, and content/property round-tripping.
- `BasicGetTest` — the Java-client counterpart of the broker's own Rust
  `basic_get_test.rs` (lapin-based) suite: same scenarios (get/ack/nack
  requeue/no_ack/delivery-tag ordering), driven by a different client
  implementation to confirm wire-protocol compatibility.
