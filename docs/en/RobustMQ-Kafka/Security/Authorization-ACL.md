# ACL Authorization

RobustMQ Kafka implements Kafka's ACL management APIs (`CreateAcls` / `DescribeAcls` / `DeleteAcls`): ACL rules can be created, described, deleted, and persisted to meta-service.

> **Current state (please note): ACLs can be created/described/deleted and are persisted, but they are not yet wired into request authorization.** The request path never reads ACLs, so **nothing is actually blocked** today — no Produce / Fetch / metadata operation is gated. The `authorized_operations` field in responses such as `Metadata` always returns a sentinel value (not computed). ACLs are "manageable authorization metadata" — useful for pre-configuration or migration rehearsals, but they do not constitute access control.

## ACL model

The Kafka wire model fields:

| Field | Meaning |
|---|---|
| `resource_type` | resource type. **Only `TOPIC` is accepted** |
| `resource_name` | resource name (topic name) |
| `pattern_type` | match pattern. **Only `LITERAL` (exact) is accepted** |
| `principal` | principal, e.g. `User:alice` or `ClientId:x` (missing `:` → `InvalidPrincipalType`) |
| `host` | source host; empty is normalized to `*` |
| `operation` | operation, mapped to an internal action: `ALL` → all, `READ` → subscribe, `WRITE` → publish; others → `InvalidRequest` |
| `permission_type` | `ALLOW` / `DENY` |

A non-`TOPIC` resource, or a non-`LITERAL` pattern, returns `InvalidRequest`.

## Filter matching

`DescribeAcls` / `DeleteAcls` match via a filter: enum fields set to `ANY` match anything; string filters (resource name, principal, host) match anything when empty, otherwise do **exact string equality**. True prefix/wildcard matching (`PREFIXED` / `MATCH`) is not yet implemented.

## CLI examples

```bash
# Grant alice READ on topic orders
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:alice \
  --operation Read --topic orders

# List
kafka-acls.sh --bootstrap-server localhost:9092 --list --topic orders

# Remove
kafka-acls.sh --bootstrap-server localhost:9092 \
  --remove --allow-principal User:alice \
  --operation Read --topic orders
```

Like other security data, ACLs are persisted through Raft and broadcast to every broker's cache (see [Overview](./Overview.md#persistence-path)).

## Limitations at a glance

| Limitation | Detail |
|---|---|
| Not enforced | not consulted during request authorization; nothing is blocked |
| Resource type | `TOPIC` only |
| Pattern type | `LITERAL` only; no prefix/wildcard matching |
| Operations | only `ALL` / `READ` / `WRITE` are mappable |
| authorized_operations | always a "not computed" sentinel in responses |
