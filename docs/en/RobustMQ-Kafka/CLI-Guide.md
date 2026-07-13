# CLI Guide

RobustMQ Kafka is compatible with the official Kafka command-line tools (`kafka-*.sh`). This document lists common commands by tool, with notes. All examples use `localhost:9092`.

> With SASL enabled, any command that connects to a broker must add `--command-config client.properties` (or the matching `--producer.config` / `--consumer.config`). A SASL config example is in [Quick Start](./QuickStart.md#sasl-connection-optional).

## kafka-topics.sh — Topic management

```bash
# Create
kafka-topics.sh --bootstrap-server localhost:9092 \
  --create --topic orders --partitions 6

# List
kafka-topics.sh --bootstrap-server localhost:9092 --list

# Describe (partitions, leader, ISR)
kafka-topics.sh --bootstrap-server localhost:9092 --describe --topic orders

# Add partitions (increase only)
kafka-topics.sh --bootstrap-server localhost:9092 \
  --alter --topic orders --partitions 12

# Delete
kafka-topics.sh --bootstrap-server localhost:9092 --delete --topic orders
```

> `--replica-assignment` (manual replica assignment) is **not accepted** on create; replicas are managed automatically by the storage layer.

## kafka-console-producer.sh — Produce

```bash
# Produce line by line
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic orders

# With a key
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic orders \
  --property parse.key=true --property key.separator=:

# Enable idempotence
kafka-console-producer.sh --bootstrap-server localhost:9092 --topic orders \
  --producer-property enable.idempotence=true
```

## kafka-console-consumer.sh — Consume

```bash
# Consume from the beginning
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic orders --from-beginning

# Consume with a group, printing keys
kafka-console-consumer.sh --bootstrap-server localhost:9092 \
  --topic orders --group g1 \
  --property print.key=true --property print.offset=true
```

## kafka-consumer-groups.sh — Consumer groups

```bash
# List all groups
kafka-consumer-groups.sh --bootstrap-server localhost:9092 --list

# Describe a group's offsets and lag
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --describe --group g1

# Reset offsets to earliest (group must have no active members)
kafka-consumer-groups.sh --bootstrap-server localhost:9092 \
  --group g1 --topic orders --reset-offsets --to-earliest --execute

# Delete a group
kafka-consumer-groups.sh --bootstrap-server localhost:9092 --delete --group g1
```

## kafka-get-offsets.sh — Query offsets

```bash
# Latest offset per partition
kafka-get-offsets.sh --bootstrap-server localhost:9092 \
  --topic orders --time latest

# earliest / by timestamp (ms)
kafka-get-offsets.sh --bootstrap-server localhost:9092 --topic orders --time earliest
kafka-get-offsets.sh --bootstrap-server localhost:9092 --topic orders --time 1700000000000
```

Backed by `ListOffsets`, supporting earliest / latest / by-timestamp lookup.

## kafka-configs.sh — Configuration management

```bash
# View topic config
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type topics --entity-name orders --describe

# Modify topic config (incremental)
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type topics --entity-name orders \
  --alter --add-config retention.ms=604800000
```

> Most configs are **storable but not enforced** (see [Compatibility & Limitations](./Compatibility-and-Limitations.md)).

### SCRAM user management

`kafka-configs.sh` is also used to manage SASL/SCRAM user credentials (backed by `AlterUserScramCredentials` / `DescribeUserScramCredentials`):

```bash
# Create / update SCRAM-SHA-256 user alice
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type users --entity-name alice \
  --alter --add-config 'SCRAM-SHA-256=[iterations=8192,password=alice-secret]'

# Describe user credentials
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type users --entity-name alice --describe

# Delete credentials
kafka-configs.sh --bootstrap-server localhost:9092 \
  --entity-type users --entity-name alice \
  --alter --delete-config 'SCRAM-SHA-256'
```

## kafka-acls.sh — ACL management

```bash
# Grant alice read/write on orders
kafka-acls.sh --bootstrap-server localhost:9092 \
  --add --allow-principal User:alice \
  --operation Read --operation Write --topic orders

# List ACLs
kafka-acls.sh --bootstrap-server localhost:9092 --list
```

> ⚠️ ACL rules can be created / deleted / queried, but are **not enforced** for authorization — even a deny rule leaves requests allowed. ACLs are currently for metadata management only.

## kafka-delegation-tokens.sh — Delegation tokens

Delegation-token commands must run over an authenticated connection, so they require `--command-config` (SASL):

```bash
# Create a token
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client.properties \
  --create --max-life-time-period -1 --renewer-principal User:alice

# Describe
kafka-delegation-tokens.sh --bootstrap-server localhost:9092 \
  --command-config client.properties --describe
```

> Tokens are **metadata management**: create / renew / expire / describe work, but the token itself **does not participate in authentication**.

## kafka-cluster.sh — Cluster info

```bash
# Query cluster id
kafka-cluster.sh --bootstrap-server localhost:9092 --describe
```

Backed by `DescribeCluster`, returning the cluster id and broker list; the controller points to the current meta-service Raft leader.

## kafka-broker-api-versions.sh — API version negotiation

```bash
kafka-broker-api-versions.sh --bootstrap-server localhost:9092
```

Prints every API the broker advertises and its supported version range — **the most direct way to confirm whether an API is available**. APIs that are not advertised (e.g. transactions, Share Group) do not appear. For a per-API breakdown, see the [Protocol Compatibility Matrix](./Protocol.md).

## Further reading

- [Quick Start](./QuickStart.md)
- [Protocol Compatibility Matrix](./Protocol.md)
- [Compatibility & Limitations](./Compatibility-and-Limitations.md)
