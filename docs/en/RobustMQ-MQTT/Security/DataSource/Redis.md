# Redis Data Source

Redis data source is suitable for low-latency identity lookups and cache-oriented architectures.

## Suitable Scenarios

- Identity and ACL data are already modeled in Redis.
- You need lightweight integration with existing cache systems.
- You want fast sync source behavior with simple key/hash contracts.

## Core Capabilities

- Read user and ACL data using agreed key/hash patterns.
- Keep auth hot path memory-based inside broker.
- Reuse existing Redis structures without mandatory schema migration.

## Runtime Model (Brief)

1. Broker syncs auth-related data from Redis.
2. Synced data updates local cache.
3. CONNECT auth checks in-memory cache first.

## Configuration

Key fields in `redis_config`:

- `redis_addr`: Redis endpoint (for example `127.0.0.1:6379`)
- `mode`: `Single` / `Cluster` / `Sentinel`
- `database`: DB index
- `password`: Redis password
- `query`: query template for your model

## Example

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "redis"

[mqtt.auth.config.storage_config.redis_config]
redis_addr = "127.0.0.1:6379"
mode = "Single"
database = 0
password = ""
query = "HMGET mqtt_user:${username} password_hash salt"
```
