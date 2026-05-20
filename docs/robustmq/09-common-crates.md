# 9. Common Crates — `src/common/*`

Reusable infrastructure shared by every layer. Each crate has a narrow responsibility.

| Crate | Path | Responsibility |
|-------|------|----------------|
| `common-base` | [src/common/base](../../src/common/base) | Errors (`CommonError`), IDs (`unique_id`), role flags (`is_broker_node`, `is_engine_node`, `is_meta_node`), Tokio runtime builders, `TaskSupervisor`, time utilities |
| `common-config` | [src/common/config](../../src/common/config) | Typed config: `BrokerConfig`, per-protocol runtime configs, `StorageType` enum, TOML loading |
| `metadata-struct` | [src/common/metadata-struct](../../src/common/metadata-struct) | Canonical entities crossing layers: `AdapterWriteRecord`, `StorageRecord`, `EngineShard`, `EngineSegment`, `Topic`, `Tenant`, `AgentCard`, ACLs |
| `rocksdb-engine` | [src/common/rocksdb-engine](../../src/common/rocksdb-engine) | `RocksDBEngine` wrapper, column-family registry, test fixtures |
| `mmap-file` | [src/common/mmap-file](../../src/common/mmap-file) | mmap-backed append-only file used by storage-engine segments |
| `network-server` | [src/common/network-server](../../src/common/network-server) | Generic async server framework: TCP/TLS/WS/QUIC, `ConnectionManager`, `RequestChannel`, `CommandRegistry` |
| `node-call` | [src/common/node-call](../../src/common/node-call) | `NodeCallManager` — typed peer-to-peer RPC helper built on `grpc-clients` |
| `group` | [src/common/group](../../src/common/group) | Consumer-group abstraction (`OffsetManager`) used by Kafka / mq9 / connectors |
| `delay-task` | [src/common/delay-task](../../src/common/delay-task) | Generic delayed-task wheel used by `delay-message`, connectors, controllers |
| `rate-limit` | [src/common/rate-limit](../../src/common/rate-limit) | `GlobalRateLimiterManager`, per-resource limiters (token bucket + leaky bucket) |
| `security` | [src/common/security](../../src/common/security) | `SecurityManager`: TLS material, JWT, login flows, super-user provisioning |
| `metrics` | [src/common/metrics](../../src/common/metrics) | Prometheus registry, common metric helpers; `init_metrics()` |
| `pprof-monitor` | (used by admin-server) | CPU profiling integration via `pprof` crate |
| `healthy` | [src/common/healthy](../../src/common/healthy) | Readiness probes, `wait_for_grpc_ready` |
| `system-info` | [src/common/system-info](../../src/common/system-info) | Host metadata (CPU, mem, disk) for dashboard & autoscaling |
| `search-engine` | [src/common/search-engine](../../src/common/search-engine) | LanceDB-backed vector store (used by mq9 semantic discovery) |
| `llm-engine` | [src/common/llm-engine](../../src/common/llm-engine) | Embedding model integration (fastembed) for mq9 |
| `third-driver` | [src/common/third-driver](../../src/common/third-driver) | Shared client helpers for external systems used by connectors |

## How they compose

- **Every** binary/library transitively depends on `common-base`, `common-config`,
  `metadata-struct`.
- **Brokers** add `network-server`, `security`, `rate-limit`, `metrics`, `group`,
  `delay-task`.
- **Storage** adds `rocksdb-engine`, `mmap-file`.
- **mq9** adds `search-engine`, `llm-engine`.

The split keeps compile times reasonable and forces explicit dependencies: if a new
broker needs ACLs, it must opt into `security` — there is no global god-crate.

Continue to [CLI & Binaries](10-cli-and-tooling.md).
