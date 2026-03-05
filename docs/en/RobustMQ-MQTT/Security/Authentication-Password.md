# Password Authentication

Password authentication is the primary authentication method in current production usage.  
It verifies client identity using username and password.

## Configuration

Set `authn_type = "password_based"` and configure a storage backend in `storage_config`.

When using MySQL, the config should align with the `MysqlConfig` structure:

```rust
pub struct MysqlConfig {
    pub mysql_addr: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub query_user: String,
    pub query_acl: String,
    pub query_blacklist: String,
}
```

Field meanings:

- `mysql_addr`: MySQL endpoint (for example `127.0.0.1:3306`)
- `database`: database name
- `username` / `password`: database credentials
- `query_user`: SQL used to sync user data (core auth source)
- `query_acl`: SQL used to sync ACL data
- `query_blacklist`: SQL used to sync blacklist data

Password authentication does not require a fixed table schema.  
The key requirement is that query results match the expected mapping contract.

Supported storage backends:

- Built-in data source (Meta Service)
- MySQL
- PostgreSQL
- Redis
- HTTP

See storage-specific details:

- [Data Source Overview](./DataSource/)
- [Built-in Data Source (Meta Service)](./DataSource/BuiltIn.md)
- [MySQL Data Source](./DataSource/MySQL.md)
- [PostgreSQL Data Source](./DataSource/PostgreSQL.md)
- [Redis Data Source](./DataSource/Redis.md)
- [HTTP Data Source](./DataSource/HTTP.md)

## Usage

1. Enable `password_based` in broker auth config.
2. Configure the selected backend.
3. Client sends `username/password` in CONNECT.
4. On success, broker proceeds to session and authorization flow.

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

## Notes

- Disable password-free login in production.
- Keep auth cache hit rate high for better CONNECT latency.
- Ensure query result columns and types follow expected contracts.
