# RobustMQ Admin Server HTTP API 通用指南

## 概述

RobustMQ Admin Server 是 HTTP 管理接口服务，提供对 RobustMQ 集群的全面管理功能。

- **基础地址**: `http://localhost:8080`
- **API 前缀**: `/api` (所有管理接口都使用此前缀)
- **请求方法**: 主要使用 `POST` 方法
- **数据格式**: JSON
- **响应格式**: JSON

## API 文档导航

- 📋 **[集群管理 API](CLUSTER.md)** - 集群配置和状态管理
- 🔧 **[MQTT Broker API](MQTT.md)** - MQTT 代理相关的所有管理接口

---

## 通用响应格式

### 成功响应
```json
{
  "code": 0,
  "message": "success",
  "data": {...}
}
```

### 错误响应
```json
{
  "code": 500,
  "message": "error message",
  "data": null
}
```

### 分页响应格式
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [...],
    "total_count": 100
  }
}
```

---

## 通用请求参数

大多数列表查询接口支持以下通用参数：

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `limit` | `u32` | 否 | 每页大小，默认 10000 |
| `page` | `u32` | 否 | 页码，从1开始，默认 1 |
| `sort_field` | `string` | 否 | 排序字段 |
| `sort_by` | `string` | 否 | 排序方式：asc/desc |
| `filter_field` | `string` | 否 | 过滤字段 |
| `filter_values` | `array` | 否 | 过滤值列表 |
| `exact_match` | `string` | 否 | 精确匹配：true/false |

### 分页参数示例
```json
{
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "status",
  "filter_values": ["active"],
  "exact_match": "false"
}
```

---

## 基础接口

### 服务版本查询
- **接口**: `GET /`
- **描述**: 获取服务版本信息
- **请求参数**: 无
- **响应示例**:
```json
"RobustMQ API v0.1.34"
```

### 集群状态查询
- **接口**: `POST /api/status`
- **描述**: 获取集群状态、版本和节点信息
- **请求参数**: `{}`（空对象）
- **响应示例**:
```json
{
  "code": 0,
  "data": {
    "version": "0.2.1",
    "cluster_name": "broker-server",
    "start_time": 1760828141,
    "broker_node_list": [
      {
        "cluster_name": "broker-server",
        "roles": ["meta", "broker"],
        "extend": "{\"mqtt\":{\"grpc_addr\":\"192.168.100.100:1228\",\"mqtt_addr\":\"192.168.100.100:1883\",\"mqtts_addr\":\"192.168.100.100:1885\",\"websocket_addr\":\"192.168.100.100:8083\",\"websockets_addr\":\"192.168.100.100:8085\",\"quic_addr\":\"192.168.100.100:9083\"}}",
        "node_id": 1,
        "node_ip": "192.168.100.100",
        "node_inner_addr": "192.168.100.100:1228",
        "start_time": 1760828141,
        "register_time": 1760828142
      }
    ],
    "meta": {
      "running_state": {
        "Ok": null
      },
      "id": 1,
      "current_term": 1,
      "vote": {
        "leader_id": {
          "term": 1,
          "node_id": 1
        },
        "committed": true
      },
      "last_log_index": 422,
      "last_applied": {
        "leader_id": {
          "term": 1,
          "node_id": 1
        },
        "index": 422
      },
      "snapshot": null,
      "purged": null,
      "state": "Leader",
      "current_leader": 1,
      "millis_since_quorum_ack": 0,
      "last_quorum_acked": 1760828146763525625,
      "membership_config": {
        "log_id": {
          "leader_id": {
            "term": 0,
            "node_id": 0
          },
          "index": 0
        },
        "membership": {
          "configs": [[1]],
          "nodes": {
            "1": {
              "node_id": 1,
              "rpc_addr": "127.0.0.1:1228"
            }
          }
        }
      },
      "heartbeat": {
        "1": 1760828146387602084
      },
      "replication": {
        "1": {
          "leader_id": {
            "term": 1,
            "node_id": 1
          },
          "index": 422
        }
      }
    }
  }
}
```

**响应字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | `string` | RobustMQ 版本号 |
| `cluster_name` | `string` | 集群名称 |
| `start_time` | `u64` | 服务启动时间（Unix时间戳，秒） |
| `broker_node_list` | `array` | Broker 节点列表 |
| `meta` | `object` | Meta 集群 Raft 状态信息（结构化对象） |

**Broker 节点字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `node_id` | `u64` | 节点 ID |
| `node_ip` | `string` | 节点 IP 地址 |
| `node_inner_addr` | `string` | 节点内部通信地址（gRPC地址） |
| `cluster_name` | `string` | 所属集群名称 |
| `roles` | `array` | 节点角色列表（如 `["meta", "broker"]`） |
| `extend` | `string` | 扩展信息（JSON字符串），包含各协议的监听地址 |
| `start_time` | `u64` | 节点启动时间（Unix时间戳，秒） |
| `register_time` | `u64` | 节点注册时间（Unix时间戳，秒） |

**扩展信息（extend）字段说明**:

`extend` 字段是一个 JSON 字符串，包含以下 MQTT 协议相关的地址信息：

```json
{
  "mqtt": {
    "grpc_addr": "192.168.100.100:1228",
    "mqtt_addr": "192.168.100.100:1883",
    "mqtts_addr": "192.168.100.100:1885",
    "websocket_addr": "192.168.100.100:8083",
    "websockets_addr": "192.168.100.100:8085",
    "quic_addr": "192.168.100.100:9083"
  }
}
```

| 字段 | 说明 |
|------|------|
| `grpc_addr` | gRPC 服务地址 |
| `mqtt_addr` | MQTT 协议监听地址 |
| `mqtts_addr` | MQTT over TLS 监听地址 |
| `websocket_addr` | WebSocket 协议监听地址 |
| `websockets_addr` | WebSocket over TLS 监听地址 |
| `quic_addr` | QUIC 协议监听地址 |

**Meta 集群状态（meta）字段说明**:

`meta` 字段是一个结构化对象，包含 Meta 集群的 Raft 状态信息：

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `u64` | 节点 ID |
| `state` | `string` | 当前节点状态（Leader/Follower/Candidate） |
| `current_leader` | `u64` | 当前 Leader 节点 ID |
| `current_term` | `u64` | 当前任期号 |
| `last_log_index` | `u64` | 最后一条日志索引 |
| `running_state` | `object` | 运行状态（通常为 `{"Ok": null}` 表示正常） |
| `vote` | `object` | 投票信息，包含 `leader_id` 和 `committed` |
| `last_applied` | `object` | 最后应用的日志信息 |
| `snapshot` | `object/null` | 快照信息 |
| `purged` | `object/null` | 清理信息 |
| `millis_since_quorum_ack` | `u64` | 距离法定人数确认的毫秒数 |
| `last_quorum_acked` | `u128` | 最后法定人数确认的时间戳（纳秒） |
| `membership_config` | `object` | 集群成员配置信息 |
| `heartbeat` | `object` | 心跳信息（节点ID到时间戳的映射） |
| `replication` | `object` | 复制状态信息 |

---

## 错误码说明

| 错误码 | 说明 |
|--------|------|
| 0 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 未授权 |
| 403 | 禁止访问 |
| 404 | 资源不存在 |
| 500 | 服务器内部错误 |

---

## 使用示例

### 基本请求示例
```bash
# 获取服务版本
curl -X GET http://localhost:8080/

# 获取集群状态
curl -X POST http://localhost:8080/api/status \
  -H "Content-Type: application/json" \
  -d '{}'

# 带分页的列表查询
curl -X POST http://localhost:8080/api/mqtt/user/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "sort_field": "username",
    "sort_by": "asc"
  }'
```

### 错误处理示例
```bash
# 当请求失败时，会返回错误信息
{
  "code": 400,
  "message": "Invalid parameter: username is required",
  "data": null
}
```

---

## 注意事项

1. **请求方法**:
   - 根路径 `/` 使用 GET 方法
   - 其他所有接口（包括 `/api/status`）使用 POST 方法
2. **请求体**: 即使是查询操作，也需要发送 JSON 格式的请求体（可以是空对象 `{}`）
3. **时间格式**:
   - 输入时间使用 Unix 时间戳（秒）
   - 输出时间使用本地时间格式字符串 "YYYY-MM-DD HH:MM:SS"
4. **分页**: 页码 `page` 从 1 开始计数
5. **配置验证**: 创建资源时会验证配置格式的正确性
6. **权限控制**: 建议在生产环境中添加适当的认证和授权机制
7. **错误处理**: 所有错误都会返回详细的错误信息，便于调试
8. **内容类型**: 请求必须设置 `Content-Type: application/json` 头部

---

## 开发和调试

### 启动服务
```bash
# 启动 admin-server
cargo run --bin admin-server

# 或者使用已编译的二进制文件
./target/release/admin-server
```

### 测试连接
```bash
# 测试服务是否正常运行
curl -X GET http://localhost:8080/
```

### 日志查看
服务运行时会输出详细的日志信息，包括：
- 请求路径和参数
- 响应状态和数据
- 错误信息和堆栈跟踪

---

*文档版本: v4.0*
*最后更新: 2025-09-20*
*基于代码版本: RobustMQ Admin Server v0.1.34*
