# mq9 HTTP API

> This document describes the HTTP management endpoints for the mq9 protocol. For common conventions, see [COMMON.md](COMMON.md).
>
> **Prerequisite**: The following endpoints require the Broker to have the NATS/mq9 component enabled (`roles` contains `broker` and the nats-broker is running). If not enabled, the endpoint returns `"nats-broker is not running"`.

---

## 1. Mailbox List

### 1.1 Query Mailbox List

- **Endpoint**: `GET /api/mq9/mail/list`
- **Description**: Query registered mq9 Mailboxes, with optional filtering by tenant or address, and pagination support.
- **Request Parameters** (Query String):

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tenant` | string | No | Exact filter by tenant name |
| `mail_address` | string | No | Fuzzy match on Mailbox address |
| `limit` | u32 | No | Page size, default 20 |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field: `tenant`, `mail_address` |
| `sort_by` | string | No | Sort direction: `asc` \| `desc` |

- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "data": [
      {
        "tenant": "default",
        "mail_address": "agent.billing.001",
        "desc": "billing agent inbox",
        "ttl": 86400,
        "create_time": 1716451200000
      }
    ],
    "total_count": 1
  },
  "error": null
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `tenant` | string | Owning tenant |
| `mail_address` | string | Mailbox address, globally unique |
| `desc` | string | Description |
| `ttl` | u64 | Mailbox lifetime in seconds; 0 means no expiry |
| `create_time` | u64 | Creation timestamp (milliseconds) |

---

## 2. Agent List

### 2.1 Query Agent List

- **Endpoint**: `GET /api/mq9/agent/list`
- **Description**: Query Agents registered to the mq9 registry, with optional filtering by tenant or name, and pagination support.
- **Request Parameters** (Query String):

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `tenant` | string | No | Exact filter by tenant name |
| `name` | string | No | Fuzzy match on Agent name |
| `limit` | u32 | No | Page size, default 20 |
| `page` | u32 | No | Page number, starting from 1 |
| `sort_field` | string | No | Sort field: `tenant`, `name` |
| `sort_by` | string | No | Sort direction: `asc` \| `desc` |

- **Response Example**:
```json
{
  "code": 0,
  "data": {
    "data": [
      {
        "tenant": "default",
        "name": "agent.billing.001",
        "agent_info": "{\"name\":\"agent.billing.001\",\"description\":\"Handles invoice and payment processing\",\"capabilities\":[\"billing\",\"invoice\"]}",
        "create_time": 1716451200000
      }
    ],
    "total_count": 1
  },
  "error": null
}
```

**Response Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `tenant` | string | Owning tenant |
| `name` | string | Agent name, unique within tenant |
| `agent_info` | string | JSON string in A2A AgentCard format, describing Agent capabilities |
| `create_time` | u64 | Registration timestamp (milliseconds) |

---

## Notes

1. **Requires NATS Component**: Both endpoints depend on `nats_context`. Returns an error if the Broker has not started the NATS component.
2. **Data Source**: Data is read from the Broker's in-memory cache (`cache_manager.mail_info` / `cache_manager.agent_info`), reflecting currently loaded state.
3. **Mailbox Registration**: Mailboxes are created via the mq9 protocol command `AGENT.MAILBOX.CREATE`. The HTTP endpoint provides query access only.
4. **Agent Registration**: Agents are registered via the mq9 protocol command `AGENT.REGISTER`. The HTTP endpoint provides query access only.
