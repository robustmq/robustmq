# HTTP Data Source

HTTP data source is suitable when authentication is centralized in external services (IAM, API gateway, auth center).

## Suitable Scenarios

- Existing auth logic already lives in external HTTP services.
- Multiple systems share one identity platform.
- You want broker integration without duplicating business auth rules.

## Core Capabilities

- Delegate auth decision to external HTTP API.
- Support templated request parameters (`username/password/clientid/source_ip`).
- Reuse existing IAM infrastructure with minimal broker-side logic.

## Runtime Model (Brief)

1. On CONNECT, broker builds request and calls external auth API.
2. Broker interprets response (`allow/deny/ignore`).
3. Successful auth continues to session and authorization flow.

## Configuration

Key fields in `http_config`:

- `url`: auth endpoint
- `method`: `GET` or `POST`
- `headers`: templated request headers
- `body`: templated request body

Common variables: `${username}`, `${password}`, `${clientid}`, `${source_ip}`.

## Example

```toml
[[mqtt.auth]]
authn_type = "password_based"

[mqtt.auth.config.storage_config]
storage_type = "http"

[mqtt.auth.config.storage_config.http_config]
url = "http://127.0.0.1:8080/auth"
method = "POST"

[mqtt.auth.config.storage_config.http_config.headers]
content-type = "application/json"

[mqtt.auth.config.storage_config.http_config.body]
username = "${username}"
password = "${password}"
clientid = "${clientid}"
source_ip = "${source_ip}"
```
