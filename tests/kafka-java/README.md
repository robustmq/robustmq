# Kafka Java-client integration tests

End-to-end tests that drive RobustMQ's Kafka protocol with the official Apache
Kafka Java client (`kafka-clients`). They assume a RobustMQ broker is already
running (Kafka on `localhost:9092`, admin HTTP on `127.0.0.1:58080`).

## Run

```bash
# from the repo root
make kafka-test

# against a specific client version (multi-version SDK coverage)
make kafka-test KAFKA_CLIENTS_VERSION=3.9.1

# or directly
cd tests/kafka-java && mvn test
```

Overridable endpoints:

- `-Dbootstrap.servers=host:port` (or env `KAFKA_BOOTSTRAP_SERVERS`)
- `-Dadmin.url=http://host:port` (or env `ROBUSTMQ_ADMIN_ADDR`)

## Layout

The test code is version-agnostic; the Kafka client version is the Maven
property `kafka.clients.version` (default: latest), overridable per run. A
single source tree therefore covers every client version — only the property
changes. Other-language SDK suites live in sibling directories (e.g. a future
`tests/kafka-go/`).
