# mq9 HTTP API

> 本文档介绍 mq9 协议相关的 HTTP 管理接口。通用信息请参考 [COMMON.md](COMMON.md)。
>
> **前提条件**: 以下接口需要 Broker 启用了 NATS/mq9 组件（`roles` 包含 `broker` 且 nats-broker 正在运行）。若未启用，接口返回 `"nats-broker is not running"`。

---

## 1. Mailbox 列表

### 1.1 查询 Mailbox 列表

- **接口**: `GET /api/mq9/mail/list`
- **描述**: 查询已注册的 mq9 Mailbox 列表，支持按租户、地址过滤和分页。
- **请求参数**（Query String）:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户名精确过滤 |
| `mail_address` | string | 否 | 按 Mailbox 地址模糊匹配 |
| `limit` | u32 | 否 | 每页数量，默认 20 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段：`tenant`、`mail_address` |
| `sort_by` | string | 否 | 排序方向：`asc` \| `desc` |

- **响应示例**:
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

**响应字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `tenant` | string | 所属租户 |
| `mail_address` | string | Mailbox 地址，全局唯一 |
| `desc` | string | 描述信息 |
| `ttl` | u64 | Mailbox 生命周期（秒），0 表示永不过期 |
| `create_time` | u64 | 创建时间戳（毫秒） |

---

## 2. Agent 列表

### 2.1 查询 Agent 列表

- **接口**: `GET /api/mq9/agent/list`
- **描述**: 查询已注册到 mq9 注册中心的 Agent 列表，支持按租户、名称过滤和分页。
- **请求参数**（Query String）:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户名精确过滤 |
| `name` | string | 否 | 按 Agent 名称模糊匹配 |
| `limit` | u32 | 否 | 每页数量，默认 20 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段：`tenant`、`name` |
| `sort_by` | string | 否 | 排序方向：`asc` \| `desc` |

- **响应示例**:
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

**响应字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `tenant` | string | 所属租户 |
| `name` | string | Agent 名称，租户内唯一 |
| `agent_info` | string | JSON 字符串，A2A AgentCard 格式，包含 Agent 能力描述 |
| `create_time` | u64 | 注册时间戳（毫秒） |

---

## 注意事项

1. **依赖 NATS 组件**: 两个接口均依赖 `nats_context`，Broker 未启动 NATS 时返回错误
2. **数据来源**: 数据从 Broker 内存缓存读取（`cache_manager.mail_info` / `cache_manager.agent_info`），反映当前已加载的状态
3. **Mailbox 注册**: Mailbox 通过 mq9 协议的 `AGENT.MAILBOX.CREATE` 命令创建，HTTP 接口仅提供查询功能
4. **Agent 注册**: Agent 通过 mq9 协议的 `AGENT.REGISTER` 命令注册，HTTP 接口仅提供查询功能
