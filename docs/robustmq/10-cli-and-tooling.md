# 10. CLI & Binaries

## Binaries

`bin/` produces three artifacts (see [bin/](../../bin)):

| Binary | Crate | Purpose |
|--------|-------|---------|
| `robust-server` | [src/cmd/broker-server](../../src/cmd) | Long-running broker process (the daemon) |
| `robust-ctl` | [src/cmd/cli-command](../../src/cmd) | Operator CLI (cluster, topic, ACL, connector, schema, mq9) |
| `robust-bench` | [src/cmd/cli-bench](../../src/cmd) | Built-in load generator / benchmark client |

`src/cmd/*` are thin wrappers that parse arguments and call into the corresponding
library crate (`broker-server`, `cli-command`, `cli-bench`).

## `cli-command` — `robust-ctl`

Crate: [src/cli-command](../../src/cli-command)

```
src/cli-command/src
├── lib.rs
├── handler.rs   # top-level command dispatcher
├── output.rs    # JSON / table / yaml renderers
├── cluster/     # cluster status, node membership
├── engine/      # storage-engine inspection (shards, segments, ISR)
└── mqtt/        # MQTT-specific ops (sessions, retain, ACL, blacklist)
```

`robust-ctl` calls Meta and broker gRPC services through `grpc-clients`, so it works
against any node in a cluster.

## `cli-bench` — `robust-bench`

Crate: [src/cli-bench](../../src/cli-bench)

A multi-protocol load tool used in CI and for capacity planning. Scenarios include
MQTT pub/sub, shared subscriptions, Kafka producer/consumer, and mq9 mailbox flows.

## `cmd/broker-server`

Crate: [src/cmd/broker-server](../../src/cmd)

The actual `main.rs` for `robust-server`. Steps:

1. Parse CLI flags (`--conf`, role overrides, log level).
2. Initialize logging via `tracing` (see [config/logger.toml](../../config/logger.toml)).
3. Load config from `--conf` into `BrokerConfig` global.
4. Construct `BrokerServer::new()` and invoke the boot sequence described in
   [03-startup-flow](03-startup-flow.md).
5. Block until the daemon’s shutdown signal completes.

## Configs the binaries read

- [config/server.toml](../../config/server.toml) — single-node default.
- [config/cluster/server-{1,2,3}.toml](../../config/cluster) — sample 3-node cluster.
- [config/logger.toml](../../config/logger.toml) — `tracing-subscriber` config.
- [config/version.ini](../../config/version.ini) — build metadata.

Continue to [End-to-End Flows](11-end-to-end-flows.md).
