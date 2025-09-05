# RobustMQ Admin Server HTTP API 文档

## 概述

RobustMQ Admin Server 是基于 Axum 框架构建的 HTTP 管理接口服务，提供对 MQTT 集群的全面管理功能。

- **基础地址**: `http://localhost:{port}`
- **请求方法**: 主要使用 `POST` 方法
- **数据格式**: JSON
- **响应格式**: JSON

## 通用响应格式

### 成功响应
```json
{
  "code": 200,
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
  "code": 200,
  "message": "success",
  "data": {
    "data": [...],
    "total_count": 100
  }
}
```

## 通用请求参数

大多数列表查询接口支持以下通用参数：

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `page_num` | `u32` | 否 | 页码，从1开始 |
| `page` | `u32` | 否 | 每页大小 |
| `sort_field` | `string` | 否 | 排序字段 |
| `sort_by` | `string` | 否 | 排序方式：asc/desc |
| `filter_field` | `string` | 否 | 过滤字段 |
| `filter_values` | `array` | 否 | 过滤值列表 |
| `exact_match` | `string` | 否 | 精确匹配 |

---

## API 接口列表

### 1. 基础接口

#### 1.1 服务状态查询
- **接口**: `GET /`
- **描述**: 获取服务版本信息
- **请求参数**: 无
- **响应示例**:
```json
"RobustMQ v0.1.0"
```

---

### 2. MQTT 集群概览

#### 2.1 集群概览信息
- **接口**: `POST /mqtt/overview`
- **描述**: 获取 MQTT 集群概览信息
- **请求参数**: 无
- **响应数据结构**:
```json
{
  "node_list": [...],
  "cluster_name": "string",
  "message_in_rate": 0,
  "message_out_rate": 0,
  "connection_num": 0,
  "session_num": 0,
  "topic_num": 0,
  "placement_status": "string",
  "tcp_connection_num": 0,
  "tls_connection_num": 0,
  "websocket_connection_num": 0,
  "quic_connection_num": 0,
  "subscribe_num": 0,
  "exclusive_subscribe_num": 0,
  "share_subscribe_leader_num": 0,
  "share_subscribe_resub_num": 0,
  "exclusive_subscribe_thread_num": 0,
  "share_subscribe_leader_thread_num": 0,
  "share_subscribe_follower_thread_num": 0
}
```

#### 2.2 集群监控指标
- **接口**: `POST /mqtt/overview/metrics`
- **描述**: 获取指定时间范围内的集群监控指标
- **请求参数**:
```json
{
  "start_time": 1640995200,  // Unix 时间戳，为0时默认为1小时前
  "end_time": 1640998800     // Unix 时间戳，为0时默认为当前时间
}
```
- **响应数据结构**:
```json
{
  "connection_num": "string",    // JSON格式的时间序列数据
  "topic_num": "string",
  "subscribe_num": "string", 
  "message_in_num": "string",
  "message_out_num": "string",
  "message_drop_num": "string"
}
```

---

### 3. 客户端管理

#### 3.1 客户端列表查询
- **接口**: `POST /mqtt/client/list`
- **描述**: 查询连接到集群的客户端列表
- **请求参数**:
```json
{
  "source_ip": "192.168.1.1",      // 可选，按源IP过滤
  "connection_id": 12345,           // 可选，按连接ID过滤
  "page_num": 1,                    // 可选，页码
  "page": 20,                       // 可选，每页大小
  "sort_field": "connection_id",    // 可选，排序字段
  "sort_by": "desc",                // 可选，排序方式
  "filter_field": "protocol",       // 可选，过滤字段
  "filter_values": ["MQTT"],        // 可选，过滤值
  "exact_match": "true"             // 可选，精确匹配
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "connection_id": 12345,
      "connection_type": "TCP",
      "protocol": "MQTT",
      "source_addr": "192.168.1.100:52341",
      "create_time": "2024-01-01 10:00:00"
    }
  ],
  "total_count": 100
}
```

---

### 4. 会话管理

#### 4.1 会话列表查询
- **接口**: `POST /mqtt/session/list`
- **描述**: 查询 MQTT 会话列表
- **请求参数**:
```json
{
  "client_id": "client001",         // 可选，按客户端ID过滤
  "page_num": 1,
  "page": 20,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["client001"],
  "exact_match": "false"
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "client_id": "client001",
      "session_expiry": 3600,
      "is_contain_last_will": true,
      "last_will_delay_interval": 30,
      "create_time": 1640995200,
      "connection_id": 12345,
      "broker_id": 1,
      "reconnect_time": 1640995300,
      "distinct_time": 1640995400
    }
  ],
  "total_count": 50
}
```

---

### 5. 主题管理

#### 5.1 主题列表查询
- **接口**: `POST /mqtt/topic/list`
- **描述**: 查询 MQTT 主题列表
- **请求参数**:
```json
{
  "topic_name": "sensor/+",         // 可选，按主题名过滤
  "page_num": 1,
  "page": 20,
  "sort_field": "topic_name",
  "sort_by": "asc",
  "filter_field": "topic_name",
  "filter_values": ["sensor"],
  "exact_match": "false"
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "topic_id": "topic_001",
      "topic_name": "sensor/temperature",
      "is_contain_retain_message": true
    }
  ],
  "total_count": 25
}
```

#### 5.2 主题重写规则列表
- **接口**: `POST /mqtt/topic-rewrite/list`
- **描述**: 查询主题重写规则列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "source_topic": "old/topic",
      "dest_topic": "new/topic", 
      "regex": "^old/(.*)$",
      "action": "publish"
    }
  ],
  "total_count": 10
}
```

#### 5.3 创建主题重写规则
- **接口**: `POST /mqtt/topic-rewrite/create`
- **描述**: 创建新的主题重写规则
- **请求参数**:
```json
{
  "action": "publish",              // 动作类型
  "source_topic": "old/topic",      // 源主题
  "dest_topic": "new/topic",        // 目标主题
  "regex": "^old/(.*)$"             // 正则表达式
}
```
- **响应**: 成功返回 "success"

#### 5.4 删除主题重写规则
- **接口**: `POST /mqtt/topic-rewrite/delete`
- **描述**: 删除主题重写规则
- **请求参数**:
```json
{
  "action": "publish",
  "source_topic": "old/topic"
}
```
- **响应**: 成功返回 "success"

---

### 6. 订阅管理

#### 6.1 订阅列表查询
- **接口**: `POST /mqtt/subscribe/list`
- **描述**: 查询订阅列表
- **请求参数**:
```json
{
  "client_id": "client001",         // 可选，按客户端ID过滤
  "page_num": 1,
  "page": 20,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["client001"],
  "exact_match": "false"
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "client_id": "client001",
      "path": "sensor/+",
      "broker_id": 1,
      "protocol": "MQTT",
      "qos": "1",
      "no_local": 0,
      "preserve_retain": 0,
      "retain_handling": "0",
      "create_time": "2024-01-01 10:00:00",
      "pk_id": 1,
      "properties": "{}",
      "is_share_sub": false
    }
  ],
  "total_count": 30
}
```

#### 6.2 订阅详情查询
- **接口**: `POST /mqtt/subscribe/detail`
- **描述**: 查询订阅详情（与订阅列表相同）
- **请求参数**: 与订阅列表查询相同
- **响应数据**: 与订阅列表查询相同

#### 6.3 自动订阅列表
- **接口**: `POST /mqtt/auto-subscribe/list`
- **描述**: 查询自动订阅规则列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "topic": "system/+",
      "qos": "1",
      "no_local": false,
      "retain_as_published": false,
      "retained_handling": "0"
    }
  ],
  "total_count": 5
}
```

#### 6.4 创建自动订阅规则
- **接口**: `POST /mqtt/auto-subscribe/create`
- **描述**: 创建新的自动订阅规则
- **请求参数**:
```json
{
  "topic": "system/+",              // 主题模式
  "qos": 1,                         // QoS 级别
  "no_local": false,                // 是否本地
  "retain_as_published": false,     // 保持发布状态
  "retained_handling": 0            // 保留消息处理方式
}
```
- **响应**: 成功返回 "success"

#### 6.5 删除自动订阅规则
- **接口**: `POST /mqtt/auto-subscribe/delete`
- **描述**: 删除自动订阅规则
- **请求参数**:
```json
{
  "topic_name": "system/+"
}
```
- **响应**: 成功返回 "success"

#### 6.6 慢订阅列表
- **接口**: `POST /mqtt/slow-subscribe/list`
- **描述**: 查询慢订阅列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "client_id": "slow_client",
      "topic_name": "heavy/topic",
      "time_span": 5000,
      "node_info": "node1",
      "create_time": "2024-01-01 10:00:00",
      "subscribe_name": "sub001"
    }
  ],
  "total_count": 3
}
```

---

### 7. 用户管理

#### 7.1 用户列表查询
- **接口**: `POST /mqtt/user/list`
- **描述**: 查询 MQTT 用户列表
- **请求参数**:
```json
{
  "user_name": "admin",             // 可选，按用户名过滤
  "page_num": 1,
  "page": 20,
  "sort_field": "username",
  "sort_by": "asc",
  "filter_field": "username",
  "filter_values": ["admin"],
  "exact_match": "false"
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "username": "admin",
      "is_superuser": true
    }
  ],
  "total_count": 10
}
```

#### 7.2 创建用户
- **接口**: `POST /mqtt/user/create`
- **描述**: 创建新的 MQTT 用户
- **请求参数**:
```json
{
  "username": "newuser",            // 用户名
  "password": "password123",        // 密码
  "is_superuser": false             // 是否为超级用户
}
```
- **响应**: 成功返回 "success"

#### 7.3 删除用户
- **接口**: `POST /mqtt/user/delete`
- **描述**: 删除 MQTT 用户
- **请求参数**:
```json
{
  "username": "olduser"
}
```
- **响应**: 成功返回 "success"

---

### 8. ACL 管理

#### 8.1 ACL 列表查询
- **接口**: `POST /mqtt/acl/list`
- **描述**: 查询访问控制列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "resource_type": "topic",
      "resource_name": "sensor/+",
      "topic": "sensor/temperature",
      "ip": "192.168.1.0/24",
      "action": "publish",
      "permission": "allow"
    }
  ],
  "total_count": 15
}
```

#### 8.2 创建 ACL 规则
- **接口**: `POST /mqtt/acl/create`
- **描述**: 创建新的 ACL 规则
- **请求参数**: 待补充（需要查看具体实现）
- **响应**: 成功返回 "success"

#### 8.3 删除 ACL 规则
- **接口**: `POST /mqtt/acl/delete`
- **描述**: 删除 ACL 规则
- **请求参数**: 待补充（需要查看具体实现）
- **响应**: 成功返回 "success"

---

### 9. 黑名单管理

#### 9.1 黑名单列表查询
- **接口**: `POST /mqtt/blacklist/list`
- **描述**: 查询黑名单列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "blacklist_type": "client_id",
      "resource_name": "malicious_client",
      "end_time": "2024-12-31 23:59:59",
      "desc": "Blocked due to suspicious activity"
    }
  ],
  "total_count": 5
}
```

#### 9.2 创建黑名单
- **接口**: `POST /mqtt/blacklist/create`
- **描述**: 添加新的黑名单项
- **请求参数**:
```json
{
  "blacklist_type": "client_id",       // 黑名单类型
  "resource_name": "bad_client",       // 资源名称
  "end_time": 1735689599,              // 结束时间（Unix时间戳）
  "desc": "Blocked for security"       // 描述
}
```
- **响应**: 成功返回 "success"

#### 9.3 删除黑名单
- **接口**: `POST /mqtt/blacklist/delete`
- **描述**: 删除黑名单项
- **请求参数**:
```json
{
  "blacklist_type": "client_id",
  "resource_name": "bad_client"
}
```
- **响应**: 成功返回 "success"

---

### 10. 连接器管理

#### 10.1 连接器列表查询
- **接口**: `POST /mqtt/connector/list`
- **描述**: 查询连接器列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "connector_name": "kafka_connector",
      "connector_type": "kafka",
      "config": "{}",
      "topic_id": "topic_001",
      "status": "running",
      "broker_id": "broker_1",
      "create_time": "2024-01-01 10:00:00",
      "update_time": "2024-01-01 11:00:00"
    }
  ],
  "total_count": 8
}
```

#### 10.2 创建连接器
- **接口**: `POST /mqtt/connector/create`
- **描述**: 创建新的连接器
- **请求参数**:
```json
{
  "connector_name": "new_connector",   // 连接器名称
  "connector_type": "kafka",           // 连接器类型
  "config": "{}",                      // 配置信息（JSON字符串）
  "topic_id": "topic_001"              // 关联的主题ID
}
```
- **响应**: 成功返回 "success"

#### 10.3 删除连接器
- **接口**: `POST /mqtt/connector/delete`
- **描述**: 删除连接器
- **请求参数**:
```json
{
  "connector_name": "old_connector"
}
```
- **响应**: 成功返回 "success"

---

### 11. Schema 管理

#### 11.1 Schema 列表查询
- **接口**: `POST /mqtt/schema/list`
- **描述**: 查询 Schema 列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "name": "temperature_schema",
      "schema_type": "json",
      "desc": "Temperature sensor data schema",
      "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"}}}"
    }
  ],
  "total_count": 12
}
```

#### 11.2 创建 Schema
- **接口**: `POST /mqtt/schema/create`
- **描述**: 创建新的 Schema
- **请求参数**:
```json
{
  "schema_name": "new_schema",          // Schema名称
  "schema_type": "json",                // Schema类型
  "schema": "{\"type\":\"object\"}",    // Schema定义
  "desc": "Description of schema"       // 描述
}
```
- **响应**: 成功返回 "success"

#### 11.3 删除 Schema
- **接口**: `POST /mqtt/schema/delete`
- **描述**: 删除 Schema
- **请求参数**:
```json
{
  "schema_name": "old_schema"
}
```
- **响应**: 成功返回 "success"

#### 11.4 Schema 绑定列表查询
- **接口**: `POST /mqtt/schema-bind/list`
- **描述**: 查询 Schema 绑定关系列表
- **请求参数**:
```json
{
  "resource_name": "topic001",         // 可选，资源名称过滤
  "schema_name": "temp_schema",         // 可选，Schema名称过滤
  "page_num": 1,
  "page": 20,
  "sort_field": "resource_name",
  "sort_by": "asc",
  "filter_field": "schema_name",
  "filter_values": ["temp_schema"],
  "exact_match": "false"
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "data_type": "topic",
      "data": ["topic001", "topic002"]
    }
  ],
  "total_count": 5
}
```

#### 11.5 创建 Schema 绑定
- **接口**: `POST /mqtt/schema-bind/create`
- **描述**: 创建 Schema 与资源的绑定关系
- **请求参数**:
```json
{
  "schema_name": "temp_schema",         // Schema名称
  "resource_name": "sensor_topic"       // 资源名称
}
```
- **响应**: 成功返回 "success"

#### 11.6 删除 Schema 绑定
- **接口**: `POST /mqtt/schema-bind/delete`
- **描述**: 删除 Schema 绑定关系
- **请求参数**:
```json
{
  "schema_name": "temp_schema",
  "resource_name": "sensor_topic"
}
```
- **响应**: 成功返回 "success"

---

### 12. 系统管理

#### 12.1 系统告警列表
- **接口**: `POST /mqtt/system-alarm/list`
- **描述**: 查询系统告警列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "name": "High Memory Usage",
      "message": "Memory usage exceeded 80%",
      "activate_at": "2024-01-01 10:00:00",
      "activated": true
    }
  ],
  "total_count": 3
}
```

#### 12.2 集群配置设置
- **接口**: `POST /mqtt/cluster-config/set`
- **描述**: 设置集群配置
- **请求参数**:
```json
{
  "config_type": "mqtt_config",        // 配置类型
  "config": "{\"max_connections\":1000}" // 配置内容（JSON字符串）
}
```
- **响应**: 成功返回 "success"

#### 12.3 连接抖动检测列表
- **接口**: `POST /mqtt/flapping_detect/list`
- **描述**: 查询连接抖动检测列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据**: 与黑名单列表相同格式

---

## 错误码说明

| 错误码 | 说明 |
|--------|------|
| 200 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 未授权 |
| 403 | 禁止访问 |
| 404 | 资源不存在 |
| 500 | 服务器内部错误 |

---

## 使用示例

### 查询客户端列表
```bash
curl -X POST http://localhost:8080/mqtt/client/list \
  -H "Content-Type: application/json" \
  -d '{
    "page_num": 1,
    "page": 10,
    "sort_field": "connection_id",
    "sort_by": "desc"
  }'
```

### 创建用户
```bash
curl -X POST http://localhost:8080/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "testpass123",
    "is_superuser": false
  }'
```

### 查询集群概览
```bash
curl -X POST http://localhost:8080/mqtt/overview \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

## 注意事项

1. 所有接口都需要使用 POST 方法，即使是查询操作
2. 请求体必须是有效的 JSON 格式
3. 时间戳使用 Unix 时间戳格式（秒）
4. 分页参数 `page_num` 从 1 开始计数
5. 部分接口的具体参数可能需要根据实际业务需求进行调整
6. 建议在生产环境中添加适当的认证和授权机制

---

*文档版本: v1.0*  
*最后更新: 2024-01-01*
