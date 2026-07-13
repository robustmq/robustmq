# Cluster & Controller

In native Kafka the Controller is a special broker elected to own metadata and coordination (via KRaft, or ZooKeeper in older versions). RobustMQ reuses its own Raft-based meta-service: **the Controller and the various Coordinators all resolve to the current Raft Leader**, with no separate Controller election.

![Controller and Coordinator resolution](../../../images/kafka-controller.svg)

## Controller = Raft Leader

- The `controller_id` in a client `Metadata` response returns the **node id of the current Raft Leader**.
- If the Leader cannot be determined momentarily, `controller_id` returns `-1` and the client retries later.
- The Leader is resolved through a gRPC lookup with an approximately **3-second TTL cache**, so not every request hits meta. The cache refreshes on expiry; a briefly stale answer self-corrects (see `NOT_COORDINATOR` below).

## Coordinators resolve the same way

The group coordinator and the transaction coordinator use the same logic:

- `FindCoordinator` returns the current Leader node as the coordinator, together with a directly-connectable host:port (from the [node registry](./AdvertisedListeners.md)).
- Only the current Leader node performs coordination. If a coordination request (e.g. `JoinGroup`, `Heartbeat`) lands on a **non-Leader node**, that node returns `NOT_COORDINATOR`.
- On `NOT_COORDINATOR` the client re-runs `FindCoordinator` and redirects to the correct node. So even if a briefly-stale cache points at an old Leader, the flow self-corrects.

## DescribeCluster

`DescribeCluster` returns:

| Field | Value |
|---|---|
| cluster id | The cluster name (`cluster_name`, from `config/server.toml`) |
| controller id | The current Raft Leader node id |
| brokers | The broker list built from the node registry (each node id + advertised address) |

## Differences from native Kafka

| Aspect | Native Kafka | RobustMQ |
|---|---|---|
| Controller source | KRaft / ZooKeeper election | meta-service Raft Leader |
| Coordinator resolution | Hashed per group / txn to a broker | Uniformly the Raft Leader |
| After a Leader change | Client refreshes metadata | Same; `NOT_COORDINATOR` triggers redirect |

Related: [System Architecture](../SystemArchitecture.md) · [Advertised Address](./AdvertisedListeners.md).
