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
- **请求参数**: 无请求体（空JSON对象 `{}`）
- **响应数据结构**:
```json
{
  "node_list": [
    {
      "node_id": 1,
      "node_ip": "192.168.1.100",
      "node_inner_addr": "192.168.1.100:9981",
      "extend_info": "{}",
      "create_time": 1640995200
    }
  ],
  "cluster_name": "robustmq-cluster",
  "message_in_rate": 100,
  "message_out_rate": 85,
  "connection_num": 1500,
  "session_num": 1200,
  "topic_num": 50,
  "placement_status": "Leader",
  "tcp_connection_num": 800,
  "tls_connection_num": 400,
  "websocket_connection_num": 200,
  "quic_connection_num": 100,
  "subscribe_num": 2000,
  "exclusive_subscribe_num": 1500,
  "share_subscribe_leader_num": 300,
  "share_subscribe_resub_num": 200,
  "exclusive_subscribe_thread_num": 8,
  "share_subscribe_leader_thread_num": 4,
  "share_subscribe_follower_thread_num": 4
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
  "connection_num": "[{\"timestamp\":1640995200,\"value\":1500}]",
  "topic_num": "[{\"timestamp\":1640995200,\"value\":50}]",
  "subscribe_num": "[{\"timestamp\":1640995200,\"value\":2000}]",
  "message_in_num": "[{\"timestamp\":1640995200,\"value\":10000}]",
  "message_out_num": "[{\"timestamp\":1640995200,\"value\":8500}]",
  "message_drop_num": "[{\"timestamp\":1640995200,\"value\":15}]"
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
  "sort_field": "connection_id",    // 可选，排序字段：connection_id, connection_type, protocol, source_addr
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
  "sort_field": "create_time",      // 可选，排序字段：client_id
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
  "sort_field": "topic_name",       // 可选，排序字段：topic_name
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
  "filter_field": "client_id",      // 可选，过滤字段：client_id
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
      "protocol": "MQTTv4",
      "qos": "QoS1",
      "no_local": 0,
      "preserve_retain": 0,
      "retain_handling": "SendAtSubscribe",
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
- **描述**: 查询订阅详情
- **请求参数**:
```json
{}
```
- **响应**: 当前返回空字符串（功能待实现）

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
      "qos": "QoS1",
      "no_local": false,
      "retain_as_published": false,
      "retained_handling": "SendAtSubscribe"
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
  "qos": 1,                         // QoS 级别：0, 1, 2
  "no_local": false,                // 是否本地
  "retain_as_published": false,     // 保持发布状态
  "retained_handling": 0            // 保留消息处理方式：0, 1, 2
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
  "sort_field": "username",         // 可选，排序字段：username
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
      "resource_type": "ClientId",
      "resource_name": "client001",
      "topic": "sensor/+",
      "ip": "192.168.1.0/24",
      "action": "Publish",
      "permission": "Allow"
    }
  ],
  "total_count": 15
}
```

#### 8.2 创建 ACL 规则
- **接口**: `POST /mqtt/acl/create`
- **描述**: 创建新的 ACL 规则
- **请求参数**:
```json
{
  "resource_type": "ClientId",       // 资源类型：ClientId, Username, IpAddress, All
  "resource_name": "client001",      // 资源名称
  "topic": "sensor/+",               // 主题模式
  "ip": "192.168.1.100",             // IP地址
  "action": "Publish",               // 动作：Publish, Subscribe, All
  "permission": "Allow"              // 权限：Allow, Deny
}
```
- **响应**: 成功返回 "success"

#### 8.3 删除 ACL 规则
- **接口**: `POST /mqtt/acl/delete`
- **描述**: 删除 ACL 规则
- **请求参数**:
```json
{
  "resource_type": "ClientId",
  "resource_name": "client001",
  "topic": "sensor/+",
  "ip": "192.168.1.100",
  "action": "Publish",
  "permission": "Allow"
}
```
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
      "blacklist_type": "ClientId",
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
  "blacklist_type": "ClientId",        // 黑名单类型：ClientId, IpAddress, Username
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
  "blacklist_type": "ClientId",
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
      "connector_type": "Kafka",
      "config": "{\"bootstrap_servers\":\"localhost:9092\"}",
      "topic_id": "topic_001",
      "status": "Running",
      "broker_id": "1",
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
  "connector_type": "Kafka",           // 连接器类型：LocalFile, Kafka, GreptimeDB
  "config": "{\"path\":\"/tmp/mqtt.log\"}",  // 配置信息（JSON字符串）
  "topic_id": "topic_001"              // 关联的主题ID
}
```

**连接器配置示例**：

**LocalFile 连接器**:
```json
{
  "connector_type": "LocalFile",
  "config": "{\"path\":\"/tmp/mqtt_messages.log\",\"max_file_size\":\"100MB\"}"
}
```

**Kafka 连接器**:
```json
{
  "connector_type": "Kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\",\"acks\":\"all\"}"
}
```

**GreptimeDB 连接器**:
```json
{
  "connector_type": "GreptimeDB",
  "config": "{\"host\":\"localhost\",\"port\":4001,\"database\":\"mqtt_data\",\"table\":\"messages\"}"
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
      "schema_type": "JSON",
      "desc": "Temperature sensor data schema",
      "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"},\"unit\":{\"type\":\"string\"}}}"
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
  "schema_name": "sensor_data_schema",   // Schema名称
  "schema_type": "json",                 // Schema类型：json, avro, protobuf
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",  // Schema定义
  "desc": "Sensor data validation schema"  // 描述
}
```

**Schema 类型示例**：

**JSON Schema**:
```json
{
  "schema_type": "json",
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\",\"minimum\":-50,\"maximum\":100},\"humidity\":{\"type\":\"number\",\"minimum\":0,\"maximum\":100}}}"
}
```

**AVRO Schema**:
```json
{
  "schema_type": "avro",
  "schema": "{\"type\":\"record\",\"name\":\"SensorData\",\"fields\":[{\"name\":\"temperature\",\"type\":\"double\"},{\"name\":\"humidity\",\"type\":\"double\"}]}"
}
```

**Protobuf Schema**:
```json
{
  "schema_type": "protobuf",
  "schema": "syntax = \"proto3\"; message SensorData { double temperature = 1; double humidity = 2; }"
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
  "sort_field": "data_type",
  "sort_by": "asc",
  "filter_field": "data_type",
  "filter_values": ["resource"],
  "exact_match": "false"
}
```
- **响应数据结构**:
```json
{
  "data": [
    {
      "data_type": "resource",
      "data": ["sensor_data_schema", "device_status_schema"]
    },
    {
      "data_type": "schema",
      "data": ["sensor/temperature", "sensor/humidity", "device/status"]
    }
  ],
  "total_count": 2
}
```

#### 11.5 创建 Schema 绑定
- **接口**: `POST /mqtt/schema-bind/create`
- **描述**: 创建 Schema 与资源的绑定关系
- **请求参数**:
```json
{
  "schema_name": "sensor_data_schema",  // Schema名称
  "resource_name": "sensor/temperature" // 资源名称（通常是主题名）
}
```
- **响应**: 成功返回 "success"

#### 11.6 删除 Schema 绑定
- **接口**: `POST /mqtt/schema-bind/delete`
- **描述**: 删除 Schema 绑定关系
- **请求参数**:
```json
{
  "schema_name": "sensor_data_schema",
  "resource_name": "sensor/temperature"
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
      "message": "Memory usage exceeded 80% threshold",
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
  "config_type": "SlowSubscribe",      // 配置类型：SlowSubscribe, OfflineMessage
  "config": "{\"enable\":true,\"threshold\":1000}" // 配置内容（JSON字符串）
}
```
- **响应**: 成功返回 "success"
- **注意**: 当前实现中此接口功能待完善

#### 12.3 连接抖动检测列表
- **接口**: `POST /mqtt/flapping_detect/list`
- **描述**: 查询连接抖动检测列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "data": [
    {
      "client_id": "flapping_client",
      "before_last_windows_connections": 15,
      "first_request_time": 1640995200
    }
  ],
  "total_count": 2
}
```

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

## 枚举值说明

### ACL 资源类型 (resource_type)
- `ClientId`: 客户端ID
- `Username`: 用户名
- `IpAddress`: IP地址
- `All`: 所有资源

### ACL 动作 (action)
- `Publish`: 发布消息
- `Subscribe`: 订阅主题
- `All`: 所有动作

### ACL 权限 (permission)
- `Allow`: 允许
- `Deny`: 拒绝

### 黑名单类型 (blacklist_type)
- `ClientId`: 客户端ID
- `IpAddress`: IP地址
- `Username`: 用户名

### 连接器类型 (connector_type)
- `LocalFile`: 本地文件
- `Kafka`: Kafka消息队列
- `GreptimeDB`: GreptimeDB时序数据库

### Schema 类型 (schema_type)
- `json`: JSON Schema
- `avro`: Apache Avro
- `protobuf`: Protocol Buffers

### QoS 级别
- `0`: 最多一次传递
- `1`: 至少一次传递
- `2`: 恰好一次传递

### 保留消息处理方式 (retained_handling)
- `0`: 发送保留消息
- `1`: 仅在新订阅时发送保留消息
- `2`: 不发送保留消息

---

## 使用示例

### 查询集群概览
```bash
curl -X POST http://localhost:8080/mqtt/overview \
  -H "Content-Type: application/json" \
  -d '{}'
```

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

### 创建ACL规则
```bash
curl -X POST http://localhost:8080/mqtt/acl/create \
  -H "Content-Type: application/json" \
  -d '{
    "resource_type": "ClientId",
    "resource_name": "sensor001",
    "topic": "sensor/+",
    "ip": "192.168.1.100",
    "action": "Publish",
    "permission": "Allow"
  }'
```

### 创建连接器
```bash
curl -X POST http://localhost:8080/mqtt/connector/create \
  -H "Content-Type: application/json" \
  -d '{
    "connector_name": "kafka_bridge",
    "connector_type": "Kafka",
    "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",
    "topic_id": "sensor/+"
  }'
```

### 创建Schema
```bash
curl -X POST http://localhost:8080/mqtt/schema/create \
  -H "Content-Type: application/json" \
  -d '{
    "schema_name": "sensor_schema",
    "schema_type": "json",
    "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",
    "desc": "Sensor data validation schema"
  }'
```

---

## 注意事项

1. **请求方法**: 除了根路径 `/` 使用 GET 方法外，所有其他接口都使用 POST 方法
2. **请求体**: 即使是查询操作，也需要发送 JSON 格式的请求体
3. **时间格式**: 
   - 输入时间使用 Unix 时间戳（秒）
   - 输出时间使用本地时间格式字符串 "YYYY-MM-DD HH:MM:SS"
4. **分页**: 页码 `page_num` 从 1 开始计数
5. **配置验证**: 创建连接器时会验证配置格式的正确性
6. **Schema验证**: 创建Schema时会验证Schema语法的正确性
7. **权限控制**: 建议在生产环境中添加适当的认证和授权机制
8. **错误处理**: 所有错误都会返回详细的错误信息，便于调试

---

*文档版本: v2.0*  
*最后更新: 2024-01-01*  
*基于代码版本: RobustMQ Admin Server*
