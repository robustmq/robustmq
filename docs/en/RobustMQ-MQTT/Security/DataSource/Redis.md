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
- `query_user`: command to fetch user ID list (for example `SMEMBERS mqtt:users`)
- `query_acl`: command to fetch ACL ID list (for example `SMEMBERS mqtt:acls`)
- `query_blacklist`: command to fetch blacklist ID list (for example `SMEMBERS mqtt:blacklists`)

The adapter first executes `query_user/query_acl/query_blacklist` to get ID lists,
then reads hash details by key convention (for example `mqtt:user:{id}`, `mqtt:acl:{id}`, `mqtt:blacklist:{id}`).

## Hash Field Contract

### User hash (`mqtt:user:{username}`)

Required:

- `password`
- `is_superuser` (`1` or `0`)

Optional:

- `salt`
- `created` (unix seconds)

### ACL hash (`mqtt:acl:{id}`)

Required:

- `permission` (`1`=Allow, `0`=Deny)
- `access` (`0..5`)

Optional:

- `username`
- `clientid`
- `ipaddr`
- `topic`

### Blacklist hash (`mqtt:blacklist:{id}`)

Required:

- `blacklist_type`
- `resource_name`
- `end_time`

Optional:

- `desc`

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
query_user = "SMEMBERS mqtt:users"
query_acl = "SMEMBERS mqtt:acls"
query_blacklist = "SMEMBERS mqtt:blacklists"
```
