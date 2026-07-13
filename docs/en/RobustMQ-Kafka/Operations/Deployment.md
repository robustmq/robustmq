# Deployment

RobustMQ is a single binary; the `roles` list in `config/server.toml` decides which roles a node takes on. The Kafka protocol is served by the `broker` role, data lands on `engine` (storage), and metadata is managed by `meta` (Raft).

## Roles

| Role | Responsibility |
|---|---|
| `meta` | Raft-based metadata service: Controller / Coordinator, node registry, dynamic config, offsets. |
| `broker` | Protocol front end: serves Kafka (and MQTT, etc.) protocols. |
| `engine` | File Segment storage engine: message persistence, offset index, ISR replication. |

## Single-node deployment

One process takes all three roles — good for development and trials:

```toml
cluster_name = "broker-server"
broker_id = 1
broker_ip = "127.0.0.1"
roles = ["meta", "broker", "engine"]
grpc_port = 1228
http_port = 58080
meta_addrs = { 1 = "127.0.0.1:1228" }

[kafka_runtime]
tcp_port = 9092
```

Once started, connect with `localhost:9092` as `bootstrap.servers`.

## Cluster deployment

With multiple nodes, each has a unique `broker_id`, and `meta_addrs` lists all meta nodes forming the Raft group. Key points:

- Each node's `broker_ip` must be an address reachable within the cluster / by clients (see below).
- `meta_addrs` is identical on all nodes, keyed by each meta node's `broker_id`.
- `kafka_runtime.tcp_port` may be the same across nodes (different machines).

```toml
# Node 1
broker_id = 1
broker_ip = "10.0.0.1"
roles = ["meta", "broker", "engine"]
meta_addrs = { 1 = "10.0.0.1:1228", 2 = "10.0.0.2:1228", 3 = "10.0.0.3:1228" }
```

## Key ports

| Port | Config key | Purpose |
|---|---|---|
| 9092 | `kafka_runtime.tcp_port` | Kafka protocol access |
| 1228 | `grpc_port` | Inter-node gRPC / meta communication |
| 58080 | `http_port` | admin HTTP (incl. the [dynamic config](../Configuration/DynamicConfig.md) API) |

## Address caveats under Docker / Kubernetes

This is the most common pitfall: a Kafka client connects directly using the address the broker **advertises**, not the address in your `bootstrap.servers`.

- `broker_ip` determines the advertised address. In a container the socket may bind on `0.0.0.0`, but the advertised address must be a **client-reachable** value.
- Docker: if clients are on the host / other hosts, set `broker_ip` to a host-reachable IP, not the container-internal IP.
- Kubernetes: each broker needs its own stable, routable address (e.g. a headless Service + per-Pod address).
- Symptom: bootstrap succeeds but the subsequent direct connect times out — almost always an unreachable advertised address.

Full explanation in [Advertised Address](./AdvertisedListeners.md).

## Health checks

- **Port liveness**: TCP-probe `kafka_runtime.tcp_port` (e.g. 9092) to confirm the protocol port is up.
- **admin HTTP**: `http_port` (e.g. 58080) can serve readiness / liveness probes and config queries.
- **Protocol-level check**: `kafka-broker-api-versions.sh --bootstrap-server <host>:9092` or `kafka-topics.sh --list` confirms the Kafka path works.
- **Cluster state**: `DescribeCluster` (e.g. via `kafka-metadata` / AdminClient) shows `controller_id` and the broker list; a `controller_id` of `-1` means the Raft Leader is not ready yet.

Related: [Broker Configuration](../Configuration/BrokerConfig.md) · [Cluster & Controller](./ClusterAndController.md) · [Advertised Address](./AdvertisedListeners.md).
