# PostgreSQL Data Source

PostgreSQL data source fits environments where auth-related data is maintained in PostgreSQL.

## Suitable Scenarios

- Core business database is PostgreSQL.
- You want to reuse existing account/ACL data models.
- You prefer relational governance and auditing workflows.

## Core Capabilities

- Sync user and ACL data from PostgreSQL into broker cache.
- Adapt existing schema through configured query statements.
- Keep CONNECT auth decoupled from per-request DB access.

## Runtime Model (Brief)

1. Broker loads auth data from PostgreSQL.
2. Results update local cache.
3. CONNECT auth checks use in-memory cache first.

## Configuration

Key fields in `postgres_config`:

- `postgre_addr`: PostgreSQL endpoint (for example `127.0.0.1:5432`)
- `database`: database name
- `username` / `password`: DB credentials
- `query`: query statement used by the adapter

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
query = "SELECT password_hash, salt FROM mqtt_user where username = ${username} LIMIT 1"
```
