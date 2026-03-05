# PostgreSQL Data Source

PostgreSQL data source is for setups where user/ACL/blacklist data already lives in PostgreSQL.

## Suitable Scenarios

- Your main identity and policy data is in PostgreSQL.
- You want to reuse existing schema with SQL adaptation.
- You need cache-first broker auth without per-CONNECT DB reads.

## Core Capabilities

- Sync user/ACL/blacklist via `query_user/query_acl/query_blacklist`.
- Map result columns by field name (alias-based), not by position.
- Keep auth hot path in memory for stable connection handling.

## Runtime Model (Brief)

1. Broker periodically runs configured PostgreSQL queries.
2. Results are loaded into local cache.
3. CONNECT auth checks use cache first; PostgreSQL is not queried on every CONNECT.

## Configuration

Key fields in `postgres_config`:

- `postgre_addr`: PostgreSQL endpoint (for example `127.0.0.1:5432`)
- `database`: database name
- `username` / `password`: DB credentials
- `query_user`: SQL used for user sync
- `query_acl`: SQL used for ACL sync
- `query_blacklist`: SQL used for blacklist sync

## Field Contract

### `query_user`

Result must include:

- `username`
- `password`
- `salt`
- `is_superuser` (`1` or `0`)
- `created` (recommended as `YYYY-MM-DD HH:MM:SS` text, or parseable timestamp text)

### `query_acl`

Result must include:

- `permission` (`1`=Allow, `0`=Deny)
- `ipaddr`
- `username`
- `clientid`
- `access` (`0..5` => All/Subscribe/Publish/PubSub/Retain/Qos)
- `topic`

### `query_blacklist`

Result must include:

- `blacklist_type` (`ClientId` / `User` / `Ip` / `ClientIdMatch` / `UserMatch` / `IPCIDR`)
- `resource_name`
- `end_time` (non-negative unix seconds)
- `desc`

## Example

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "postgresql"

[mqtt.auth.config.storage_config.postgres_config]
postgre_addr = "127.0.0.1:5432"
database = "mqtt"
username = "postgres"
password = "postgres"
query_user = "SELECT username AS username, password AS password, salt AS salt, is_superuser AS is_superuser, created::text AS created FROM user_table"
query_acl = "SELECT permission AS permission, ipaddr AS ipaddr, username AS username, clientid AS clientid, access AS access, topic AS topic FROM acl_table"
query_blacklist = "SELECT blacklist_type AS blacklist_type, resource_name AS resource_name, end_time AS end_time, \"desc\" AS \"desc\" FROM blacklist_table"
```
