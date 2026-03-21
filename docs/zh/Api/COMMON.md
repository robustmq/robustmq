# RobustMQ Admin Server HTTP API 通用指南

## 概述

RobustMQ Admin Server 是 HTTP 管理接口服务，提供对 RobustMQ 集群的全面管理功能。

- **基础地址**: `http://localhost:8080`
- **API 前缀**: `/api` (所有管理接口都使用此前缀)
- **请求方法**: 列表/详情查询使用 `GET`，创建/删除操作使用 `POST`
- **数据格式**: JSON
- **响应格式**: JSON

## API 文档导航

- 📋 **[集群管理 API](CLUSTER.md)** - 集群配置和状态管理
- 🔧 **[MQTT Broker API](MQTT.md)** - MQTT 代理相关的所有管理接口
- 🔌 **[连接器 API](Connector.md)** - 连接器管理接口

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
- **接口**: `GET /api/status`
- **描述**: 获取集群完整状态信息，包括 RobustMQ 版本、集群名称、启动时间、Broker 节点列表以及 Meta 集群的 Raft 状态
- **请求参数**: 
```json
{}
```
（空对象）

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
        "roles": ["mqtt-broker"],
        "extend": [],
        "node_id": 1,
        "node_ip": "192.168.100.100",
        "grpc_addr": "192.168.100.100:1228",
        "engine_addr": "192.168.100.100:1229",
        "start_time": 1760828141,
        "register_time": 1760828142,
        "storage_fold": []
      }
    ],
    "nodes": ["192.168.100.100", "127.0.0.1"],
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
| `nodes` | `array` | 集群中所有唯一节点的 IP 地址列表（去重后） |
| `meta` | `object` | Meta 集群 Raft 状态信息（结构化对象） |

**Broker 节点字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `roles` | `array` | 节点角色列表（如 `["mqtt-broker"]`） |
| `extend` | `array` | 扩展信息（字节数组） |
| `node_id` | `u64` | 节点 ID |
| `node_ip` | `string` | 节点 IP 地址 |
| `grpc_addr` | `string` | gRPC 通信地址 |
| `engine_addr` | `string` | 存储引擎地址 |
| `start_time` | `u64` | 节点启动时间（Unix时间戳，秒） |
| `register_time` | `u64` | 节点注册时间（Unix时间戳，秒） |
| `storage_fold` | `array` | 存储目录列表 |

---

**Meta 集群状态（meta）字段说明**:

`meta` 字段包含 Meta 集群的 Raft 共识状态信息，用于监控集群的分布式一致性状态：

| 字段 | 类型 | 说明 |
|------|------|------|
| `running_state` | `object` | 运行状态，`{"Ok": null}` 表示正常运行 |
| `id` | `u64` | 当前节点 ID |
| `current_term` | `u64` | Raft 当前任期号 |
| `vote` | `object` | 投票信息 |
| `vote.leader_id` | `object` | Leader 标识，包含 `term` 和 `node_id` |
| `vote.committed` | `boolean` | 投票是否已提交 |
| `last_log_index` | `u64` | 最后一条日志的索引 |
| `last_applied` | `object` | 最后应用的日志信息 |
| `last_applied.leader_id` | `object` | Leader 标识 |
| `last_applied.index` | `u64` | 已应用的日志索引 |
| `snapshot` | `object/null` | 快照信息（如果存在） |
| `purged` | `object/null` | 已清理的日志信息（如果存在） |
| `state` | `string` | 当前节点 Raft 状态：`Leader`、`Follower` 或 `Candidate` |
| `current_leader` | `u64` | 当前 Leader 节点的 ID |
| `millis_since_quorum_ack` | `u64` | 自上次获得法定人数确认以来的毫秒数 |
| `last_quorum_acked` | `u128` | 最后一次法定人数确认的时间戳（纳秒精度） |
| `membership_config` | `object` | 集群成员配置信息 |
| `membership_config.log_id` | `object` | 配置对应的日志 ID |
| `membership_config.membership` | `object` | 成员信息 |
| `membership_config.membership.configs` | `array` | 配置数组，如 `[[1]]` 表示节点 1 |
| `membership_config.membership.nodes` | `object` | 节点映射，键为节点 ID 字符串，值为节点信息 |
| `heartbeat` | `object` | 心跳映射，键为节点 ID，值为心跳时间戳（纳秒） |
| `replication` | `object` | 复制状态映射，键为节点 ID，值包含 `leader_id` 和 `index` |

**使用场景说明**:
- 通过 `state` 字段判断节点是否为 Leader
- 通过 `current_leader` 字段找到当前集群的 Leader 节点
- 通过 `last_log_index` 和 `last_applied.index` 检查日志同步状态
- 通过 `heartbeat` 监控集群节点的活跃状态
- 通过 `membership_config` 了解集群成员配置

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
curl -X GET http://localhost:8080/api/status

# 带分页的列表查询
curl "http://localhost:8080/api/mqtt/user/list?limit=10&page=1&sort_field=username&sort_by=asc"
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
   - 列表/详情查询（`/list`、`/detail`、`/api/status` 等）使用 `GET`，参数通过 query string 传递
   - 创建/删除操作（`/create`、`/delete`）使用 `POST`，参数通过 JSON body 传递
2. **请求体**: POST 接口需要设置 `Content-Type: application/json` 并传递 JSON body；GET 接口参数通过 URL query string 传递
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

*最后更新: 2026-03-20*
