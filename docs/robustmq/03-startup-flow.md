# 3. Startup & Runtime

The single binary entrypoint is `robust-server` ([bin/robust-server](../../bin/robust-server)),
which calls into [src/cmd/broker-server](../../src/cmd/broker-server). That in turn
constructs and starts a `BrokerServer` from [src/broker-server/src/lib.rs](../../src/broker-server/src/lib.rs).

## `BrokerServer::new()` — wiring phase

From [lib.rs](../../src/broker-server/src/lib.rs):

1. **Metrics** — `common_metrics::init_metrics()` registers global Prometheus metrics.
2. **Config** — `broker_config()` returns the parsed `BrokerConfig` (loaded from
   `config/server.toml` or cluster file). Schema lives in
   [src/common/config](../../src/common/config).
3. **`init_base`** builds shared infrastructure:
   - `ClientPool` ([grpc-clients/src/pool.rs](../../src/grpc-clients/src/pool.rs)) — pooled
     gRPC channels to peers.
   - `RocksDBEngine` ([common/rocksdb-engine](../../src/common/rocksdb-engine)) — embedded
     KV store used by Meta + RocksDB storage driver.
   - `GlobalRateLimiterManager` — connection-rate guard.
   - `NodeCacheManager` ([broker-core/src/cache.rs](../../src/broker-core/src/cache.rs)) —
     local cache of cluster metadata.
   - `NetworkConnectionManager` ([common/network-server](../../src/common/network-server)) —
     generic TCP/TLS/WS/QUIC connection tracking.
   - `TaskSupervisor`, `OffsetManager`, `NodeCallManager`.
   - The four Tokio runtimes (`server`, `meta`, `broker`, `engine`).
4. **`init_storage`** creates `StorageEngineParams`, the `StorageDriverManager`
   ([storage-adapter/src/driver.rs](../../src/storage-adapter/src/driver.rs)),
   `DelayTaskManager`, `DelayMessageManager`, and `MetaServiceServerParams`.
5. **`init_protocol_params`** assembles `MqttBrokerServerParams`,
   `KafkaBrokerServerParams`, `AmqpBrokerServerParams`, `NatsBrokerServerParams`, plus a
   shared `RequestChannel` used by the network layer to deliver decoded frames to
   protocol handlers.

At this point nothing is listening yet; the wiring is complete.

## Boot sequence — `start_*` methods

From [server.rs](../../src/broker-server/src/server.rs) (called by `broker-server`’s
main flow):

| Step | Method | Role gate | What starts |
|------|--------|-----------|-------------|
| 1 | `start_grpc_server` | always | Internal gRPC for Meta + brokers + engine ([grpc.rs](../../src/broker-server/src/grpc.rs)) |
| 2 | `start_meta_service` | meta nodes | Raft node + meta state machine ([meta.rs](../../src/broker-server/src/meta.rs)) |
| 3 | `register_node_and_start_heartbeat` | all | Registers node with Meta and keeps heartbeat ([broker-core/heartbeat.rs](../../src/broker-core/src/heartbeat.rs)) |
| 4 | `wait_for_grpc_ready` | all | Waits until gRPC is reachable ([common/healthy](../../src/common/healthy)) |
| 5 | `start_load_cache` + `start_update_cache` | brokers | Initial cache load + delta subscription ([load_cache.rs](../../src/broker-server/src/load_cache.rs), [update_cache.rs](../../src/broker-server/src/update_cache.rs)) |
| 6 | `start_storage_engine` | engine nodes | Commit log + ISR + storage gRPC ([engine.rs](../../src/broker-server/src/engine.rs)) |
| 7 | `start_mqtt_broker` / `start_kafka_broker` / `start_nats_broker` / `start_amqp_broker` | broker nodes | Per-protocol listeners |
| 8 | `start_admin_server` | always | HTTP admin/dashboard ([admin-server](../../src/admin-server)) |
| 9 | `start_daemon` | always | Heartbeats, signal handling, graceful shutdown ([daemon.rs](../../src/broker-server/src/daemon.rs)) |

Role gating uses helpers like `common_base::role::is_broker_node` and `is_engine_node`.

## Per-protocol startup pattern

Each protocol broker follows the same shape (example: MQTT in
[broker-server/src/mqtt.rs](../../src/broker-server/src/mqtt.rs)):

1. Build a `*BrokerServerParams` (managers, caches, storage driver handle).
2. Register decoders / command handlers into the shared `network-server` command registry.
3. Bind listeners (TCP, TLS, WebSocket, QUIC) using `common/network-server`.
4. Hand decoded frames to per-protocol handler modules (in each broker’s `handler/` or
   `core/` directory).

## Graceful shutdown

[daemon.rs](../../src/broker-server/src/daemon.rs) installs SIGINT/SIGTERM handlers, then:

1. Stops accepting new connections.
2. Drains in-flight requests via `TaskSupervisor`.
3. Flushes storage engine segments and Raft logs.
4. Deregisters the node from Meta.

Continue to [Meta Service](04-meta-service.md).
