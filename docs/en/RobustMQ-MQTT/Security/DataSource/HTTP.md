# HTTP Data Source

HTTP data source is suitable when user/ACL/blacklist data is maintained in external services and exposed by HTTP APIs.

## Suitable Scenarios

- Identity and policy data is already managed by an external platform.
- You want to integrate through HTTP APIs instead of direct DB access.
- You still want broker cache-first auth behavior.

## Core Capabilities

- Pull `user` / `acl` / `blacklist` data through HTTP.
- Configure separate endpoints via `query_user/query_acl/query_blacklist`.
- Support array response or wrapped array (`users`/`acls`/`blacklists`/`data`).

## Runtime Model (Brief)

1. Broker periodically calls configured HTTP endpoints.
2. Responses are parsed and written into local cache.
3. CONNECT and access checks use in-memory cache, not per-request HTTP calls.

## Configuration

Key fields in `http_config`:

- `url`: base endpoint
- `method`: `GET` or `POST`
- `query_user`: user endpoint (full URL or relative path)
- `query_acl`: ACL endpoint (full URL or relative path)
- `query_blacklist`: blacklist endpoint (full URL or relative path)
- `headers`: request headers template
- `body`: request body template

Template variable `${resource}` is supported (`user` / `acl` / `blacklist`).

Response can be:

- top-level array: `[{...}, {...}]`
- object with list field: `{ "users": [...] }`, `{ "acls": [...] }`, `{ "blacklists": [...] }`
- generic list wrapper: `{ "data": [...] }`

## Example

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "http"

[mqtt.auth.config.storage_config.http_config]
url = "http://127.0.0.1:8080/auth-source"
method = "POST"
query_user = "/users"
query_acl = "/acls"
query_blacklist = "/blacklists"

[mqtt.auth.config.storage_config.http_config.headers]
content-type = "application/json"

[mqtt.auth.config.storage_config.http_config.body]
resource = "${resource}"
```
