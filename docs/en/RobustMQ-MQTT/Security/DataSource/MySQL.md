# MySQL Data Source

MySQL data source is suitable when users, ACL, and blacklist are already maintained in relational tables.

## Suitable Scenarios

- Existing identity/account system is in MySQL.
- You want to reuse existing tables without schema migration.
- Auth and policy data should stay in one central database.

## Core Capabilities

- Sync user/ACL/blacklist data via `query_user/query_acl/query_blacklist`.
- Query-result mapping model (no fixed table name requirement).
- Cache-first auth hot path; MySQL mainly acts as sync source.

## Runtime Model (Brief)

1. Broker runs configured queries and pulls auth-related data.
2. Results are written into local cache.
3. CONNECT auth checks only in-memory cache.

Use explicit `AS` aliases and keep field names consistent. Do not use `SELECT *`.

## Configuration

Key fields in `mysql_config`:

- `mysql_addr`: MySQL endpoint (for example `127.0.0.1:3306`)
- `database`: database name
- `username` / `password`: DB credentials
- `query_user`: SQL for user sync
- `query_acl`: SQL for ACL sync
- `query_blacklist`: SQL for blacklist sync

## Query Result Contracts

### `query_user`

Return 5 fields with these names (order is not required):

1. `username` (String)
2. `password` (String)
3. `salt` (`Option<String>`)
4. `is_superuser` (0/1)
5. `created` (datetime string)

### `query_acl`

Return 6 fields with these names (order is not required):

1. `permission` (0/1)
2. `ipaddr` (String)
3. `username` (String)
4. `clientid` (String)
5. `access` (0..5)
6. `topic` (`Option<String>`)

### `query_blacklist`

Return 4 fields with these names (order is not required):

1. `blacklist_type` (String)
2. `resource_name` (String)
3. `end_time` (u64)
4. `desc` (`Option<String>`)

## Example

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "mysql"

[mqtt.auth.config.storage_config.mysql_config]
mysql_addr = "127.0.0.1:3306"
database = "mqtt"
username = "root"
password = "123456"
query_user = "SELECT username AS username, password AS password, salt AS salt, is_superuser AS is_superuser, created AS created FROM user_table"
query_acl = "SELECT permission AS permission, ipaddr AS ipaddr, username AS username, clientid AS clientid, access AS access, topic AS topic FROM acl_table"
query_blacklist = "SELECT blacklist_type AS blacklist_type, resource_name AS resource_name, end_time AS end_time, `desc` AS `desc` FROM blacklist_table"
```
