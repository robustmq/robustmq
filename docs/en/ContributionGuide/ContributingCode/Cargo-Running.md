# Running with Cargo

The workspace currently runs RobustMQ as one broker binary. The old
`meta-service`, `mqtt-server`, and `journal-server` Cargo commands are no
longer part of the `cmd` package.

## Standalone broker

From the repository root:

```bash
make run
```

This is equivalent to running the broker binary directly:

```bash
cargo run --package cmd --bin broker-server -- --conf=config/server.toml
```

The checked-in `config/server.toml` is a single-node configuration. It uses
the `meta`, `broker`, and `engine` roles and stores data under `./data`.

The broker listens on these default local ports:

| Service | Port |
|---------|------|
| MQTT | 1883 |
| MQTT TLS | 1885 |
| MQTT WebSocket | 8083 |
| Kafka | 9092 |
| AMQP | 5672 |
| NATS | 4222 |
| Storage engine | 1779 |
| Admin HTTP API | 58080 |
| gRPC | 1228 |

## Verify

In another terminal:

```bash
curl -i http://127.0.0.1:58080/health/node
curl http://127.0.0.1:58080/metrics | head
```

For a real MQTT round trip, use MQTTX in two terminals:

```bash
mqttx sub -h 127.0.0.1 -p 1883 -u admin -P robustmq -t "test/topic"
mqttx pub -h 127.0.0.1 -p 1883 -u admin -P robustmq \
  -t "test/topic" -m "Hello RobustMQ!"
```

## Build only

```bash
cargo build --package cmd --bin broker-server
```

To run a different configuration, pass its path after `--conf`, for example:

```bash
cargo run --package cmd --bin broker-server -- \
  --conf=config/cluster/server-1.toml
```
