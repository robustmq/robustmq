# Admin HTTP API Authentication

> This document describes the authentication mechanism, login API, and configuration for the RobustMQ Admin HTTP API.

## Overview

RobustMQ Admin Server uses JWT (JSON Web Token) based authentication:

- **Local access (no auth required)**: Requests from `127.0.0.1` or `::1` are allowed through without a token. This covers local operations, CLI tools, and curl on the same machine.
- **Remote access requires a token**: Requests from any other IP must include a valid Bearer token in the HTTP header.
- **Exempt paths**: `/api/v1/login`, `/health/*`, and `/metrics` are always accessible without a token.

---

## Authentication Flow

```text
Client
  │
  ├─ Local (127.0.0.1 / ::1)
  │    └─ Access all endpoints directly, no token needed ✅
  │
  └─ Remote IP
       │
       ├─ POST /api/v1/login  →  obtain JWT token
       │
       └─ Include Authorization: Bearer <token> in subsequent requests
```

---

## Login Endpoint

### `POST /api/v1/login`

Exchange username and password for a JWT token.

**Request body:**

```json
{
  "username": "admin",
  "password": "<your-password>"
}
```

**Success response (HTTP 200):**

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "token": "<JWT_TOKEN>",
    "expires_in": 28800
  }
}
```

| Field | Type | Description |
|---|---|---|
| `token` | string | JWT token to use in subsequent requests |
| `expires_in` | number | Token validity in seconds (default: 8 hours = 28800 s) |

**Failure response (HTTP 401):**

```json
{
  "code": 401,
  "message": "Invalid username or password",
  "data": null
}
```

---

## Using the Token

After obtaining a token, include the following header in every API request:

```http
Authorization: Bearer <token>
```

**curl examples:**

```bash
# 1. Login to get a token (required for remote access)
TOKEN=$(curl -s -X POST http://<host>:58080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"<your-password>"}' \
  | jq -r '.data.token')

# 2. Call an API endpoint with the token
curl -H "Authorization: Bearer $TOKEN" \
  http://<host>:58080/api/mqtt/overview
```

**Local access (no token needed):**

```bash
# Running on the same machine — no token required
curl http://127.0.0.1:58080/api/mqtt/overview
```

---

## Configuration

Authentication settings are in the `[admin]` section of `config/server.toml`:

```toml
[admin]
# Admin username. Default: admin
username = "admin"

# Admin password. Change this in production.
password = "<your-password>"

# HMAC-SHA256 secret used to sign JWT tokens.
# Use a random string of 32+ characters in production.
jwt_secret = "<your-jwt-secret>"

# Token validity in hours. Default: 8
token_ttl_hours = 8
```

> ⚠️ **Security notice**: Always change `password` and `jwt_secret` before deploying to production. Never use the default values.

---

## Client Integration

### Dashboard (Web UI)

The Dashboard has a built-in login page. After entering credentials, it calls `/api/v1/login` automatically, stores the token, and attaches it to every subsequent request. When the token expires, the UI redirects to the login page.

### curl / Scripts

```bash
# On the server (local) — no token needed
curl http://127.0.0.1:58080/api/cluster/config/get

# From a remote machine — login first
TOKEN=$(curl -s -X POST http://<host>:58080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"<your-password>"}' \
  | jq -r '.data.token')

curl -H "Authorization: Bearer $TOKEN" \
  http://<host>:58080/api/cluster/config/get
```

### CLI Tool (cli-command)

The CLI tool connects to `127.0.0.1` by default, so it benefits from local access and requires no token. For remote connections, pass the token via `--token` or set it in the CLI configuration file.

---

## Error Reference

| HTTP Status | `code` field | Meaning |
|---|---|---|
| 401 | 401 | Missing token, invalid token, or token expired |
| 200 | 0 | Authenticated, normal response |

When a token expires, call `/api/v1/login` again to obtain a new one.
