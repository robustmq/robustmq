# Client Quotas

RobustMQ Kafka implements the client-quota management APIs (`AlterClientQuotas` / `DescribeClientQuotas`): quotas can be set, queried, and persisted to meta-service.

> **Current state (please note): quotas can be set, queried, and persisted, but they are not yet wired into throttling.** The Produce / Fetch paths never read quotas, so setting one does **not actually throttle**; quotas are loaded into the broker cache, but no call site consults them to rate-limit a producer or consumer. Quotas are "manageable metadata" today.

## Supported scope

| Dimension | Support |
|---|---|
| Entity type | **`client-id` only**; others (e.g. `user`) return `InvalidRequest` |
| Entity dimensions | single-dimension only; multi-dimension returns `InvalidRequest` |
| Quota keys | `producer_byte_rate`, `consumer_byte_rate` |
| Key value | for non-remove ops, the value must be positive and finite, else `InvalidRequest` |
| Entity name | may not be empty, may not be the reserved `__default__` |

## CLI examples

```bash
# Set producer/consumer byte-rate limits (bytes/sec) for client-id = svc-a
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --entity-type clients --entity-name svc-a \
  --add-config 'producer_byte_rate=1048576,consumer_byte_rate=2097152'

# Describe
kafka-configs.sh --bootstrap-server localhost:9092 \
  --describe --entity-type clients --entity-name svc-a

# Delete a key
kafka-configs.sh --bootstrap-server localhost:9092 \
  --alter --entity-type clients --entity-name svc-a \
  --delete-config 'producer_byte_rate'
```

`DescribeClientQuotas` filter matching supports `EXACT`, `DEFAULT`, and `ANY` match types, with an optional `strict` flag.

Quotas are persisted through Raft and broadcast to every broker's cache (see [Overview](./Overview.md#persistence-path)).

## Limitations at a glance

| Limitation | Detail |
|---|---|
| Not enforced | no throttling; setting a quota does not rate-limit produce/consume |
| Entity type | `client-id` only; no user or user+client-id |
| Quota keys | byte-rate only; no request rate (`request_percentage`), etc. |
