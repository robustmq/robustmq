# MQTT Broker 管理 HTTP API

> 本文档介绍 MQTT Broker 相关的所有 HTTP API 接口。通用信息请参考 [COMMON.md](COMMON.md)。

## API 接口列表

### 1. 集群概览

#### 1.1 集群概览信息
- **接口**: `POST /api/mqtt/overview`
- **描述**: 获取 MQTT 集群概览信息
- **请求参数**: 空 JSON 对象
```json
{}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success", 
  "data": {
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
}
```

#### 1.2 集群监控指标
- **接口**: `POST /api/mqtt/overview/metrics`
- **描述**: 获取指定时间范围内的集群监控指标
- **请求参数**:
```json
{
  "start_time": 1640995200,  // Unix 时间戳，开始时间
  "end_time": 1640998800     // Unix 时间戳，结束时间
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "connection_num": "[{\"timestamp\":1640995200,\"value\":1500}]",
    "topic_num": "[{\"timestamp\":1640995200,\"value\":50}]", 
    "subscribe_num": "[{\"timestamp\":1640995200,\"value\":2000}]",
    "message_in_num": "[{\"timestamp\":1640995200,\"value\":10000}]",
    "message_out_num": "[{\"timestamp\":1640995200,\"value\":8500}]",
    "message_drop_num": "[{\"timestamp\":1640995200,\"value\":15}]"
  }
}
```

---

### 2. 客户端管理

#### 2.1 客户端列表查询
- **接口**: `POST /api/mqtt/client/list`
- **描述**: 查询连接到集群的客户端列表
- **请求参数**:
```json
{
  "source_ip": "192.168.1.1",      // 可选，按源IP过滤
  "connection_id": 12345,           // 可选，按连接ID过滤
  "limit": 20,                      // 可选，每页大小
  "page": 1,                        // 可选，页码
  "sort_field": "connection_id",    // 可选，排序字段
  "sort_by": "desc",                // 可选，排序方式
  "filter_field": "client_id",      // 可选，过滤字段（例如："connection_id", "client_id"）
  "filter_values": ["client001"],   // 可选，过滤值
  "exact_match": "true"             // 可选，精确匹配
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "client_id": "client001",
        "connection_id": 12345,
        "mqtt_connection": {
          "connect_id": 12345,
          "client_id": "client001",
          "is_login": true,
          "source_ip_addr": "192.168.1.100",
          "login_user": "user001",
          "keep_alive": 60,
          "topic_alias": {},
          "client_max_receive_maximum": 65535,
          "max_packet_size": 268435455,
          "topic_alias_max": 65535,
          "request_problem_info": 1,
          "receive_qos_message": 0,
          "sender_qos_message": 0,
          "create_time": 1640995200
        },
        "network_connection": {
          "connection_type": "Tcp",
          "connection_id": 12345,
          "protocol": "MQTT5",
          "addr": "192.168.1.100:52341",
          "last_heartbeat_time": 1640995200,
          "create_time": 1640995200
        },
        "session": {
          "client_id": "client001",
          "session_expiry": 3600,
          "is_contain_last_will": true,
          "last_will_delay_interval": 30,
          "create_time": 1640995200,
          "connection_id": 12345,
          "broker_id": 1,
          "reconnect_time": 1640995300,
          "distinct_time": 1640995400
        },
        "heartbeat": {
          "protocol": "Mqtt5",
          "keep_live": 60,
          "heartbeat": 1640995500
        }
      }
    ],
    "total_count": 100
  }
}
```

**字段说明**:

- **mqtt_connection**: MQTT 协议层连接信息
  - `connect_id`: 连接ID
  - `client_id`: MQTT 客户端ID
  - `is_login`: 客户端是否已登录
  - `source_ip_addr`: 客户端源IP地址
  - `login_user`: 已认证的用户名
  - `keep_alive`: 保活间隔（秒）
  - `topic_alias`: 该连接的主题别名映射
  - `client_max_receive_maximum`: 可同时接收的 QoS 1 和 QoS 2 消息的最大数量
  - `max_packet_size`: 最大数据包大小（字节）
  - `topic_alias_max`: 主题别名的最大数量
  - `request_problem_info`: 是否返回详细错误信息（0 或 1）
  - `receive_qos_message`: 待接收的 QoS 1/2 消息数量
  - `sender_qos_message`: 待发送的 QoS 1/2 消息数量
  - `create_time`: 连接创建时间戳

- **network_connection**: 网络层连接信息（断开连接时为 null）
  - `connection_type`: 连接类型（Tcp, Tls, Websocket, Websockets, Quic）
  - `connection_id`: 网络连接ID
  - `protocol`: 协议版本（MQTT3, MQTT4, MQTT5）
  - `addr`: 客户端套接字地址
  - `last_heartbeat_time`: 最后心跳时间戳
  - `create_time`: 网络连接创建时间戳

- **session**: MQTT 会话信息（无会话时为 null）
  - `client_id`: MQTT 客户端ID
  - `session_expiry`: 会话过期间隔（秒）
  - `is_contain_last_will`: 会话是否包含遗愿消息
  - `last_will_delay_interval`: 遗愿消息延迟间隔（秒，可选）
  - `create_time`: 会话创建时间戳
  - `connection_id`: 关联的连接ID（可选）
  - `broker_id`: 托管会话的 Broker 节点ID（可选）
  - `reconnect_time`: 最后重连时间戳（可选）
  - `distinct_time`: 最后断开连接时间戳（可选）

- **heartbeat**: 连接心跳信息（不可用时为 null）
  - `protocol`: MQTT 协议版本（Mqtt3, Mqtt4, Mqtt5）
  - `keep_live`: 保活间隔（秒）
  - `heartbeat`: 最后心跳时间戳

---

### 3. 会话管理

#### 3.1 会话列表查询
- **接口**: `POST /api/mqtt/session/list`
- **描述**: 查询 MQTT 会话列表
- **请求参数**:
```json
{
  "client_id": "client001",         // 可选，按客户端ID过滤
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",      // 可选，排序字段
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["client001"],
  "exact_match": "false"
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

---

### 4. 主题管理

#### 4.1 主题列表查询
- **接口**: `POST /api/mqtt/topic/list`
- **描述**: 查询 MQTT 主题列表
- **请求参数**:
```json
{
  "topic_name": "sensor/+",         // 可选，按主题名过滤
  "limit": 20,
  "page": 1,
  "sort_field": "topic_name",       // 可选，排序字段
  "sort_by": "asc",
  "filter_field": "topic_name",
  "filter_values": ["sensor"],
  "exact_match": "false"
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "topic_id": "topic_001",
        "topic_name": "sensor/temperature",
        "is_contain_retain_message": true
      }
    ],
    "total_count": 25
  }
}
```

#### 4.2 主题重写规则列表
- **接口**: `POST /api/mqtt/topic-rewrite/list`
- **描述**: 查询主题重写规则列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "source_topic": "old/topic/+",
        "dest_topic": "new/topic/$1", 
        "regex": "^old/topic/(.+)$",
        "action": "All"
      }
    ],
    "total_count": 10
  }
}
```

#### 4.3 创建主题重写规则
- **接口**: `POST /api/mqtt/topic-rewrite/create`
- **描述**: 创建新的主题重写规则
- **请求参数**:
```json
{
  "action": "All",                  // 动作类型：All, Publish, Subscribe
  "source_topic": "old/topic/+",   // 源主题模式
  "dest_topic": "new/topic/$1",     // 目标主题模式
  "regex": "^old/topic/(.+)$"       // 正则表达式
}
```

- **响应**: 成功返回 "success"

#### 4.4 删除主题重写规则
- **接口**: `POST /api/mqtt/topic-rewrite/delete`
- **描述**: 删除主题重写规则
- **请求参数**:
```json
{
  "action": "All",
  "source_topic": "old/topic/+"
}
```

- **响应**: 成功返回 "success"

---

### 5. 订阅管理

#### 5.1 订阅列表查询
- **接口**: `POST /api/mqtt/subscribe/list`
- **描述**: 查询订阅列表
- **请求参数**:
```json
{
  "client_id": "client001",         // 可选，按客户端ID过滤
  "limit": 20,
  "page": 1,
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
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "client_id": "client001",
        "path": "sensor/+",
        "broker_id": 1,
        "protocol": "MQTTv5",
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
}
```

#### 5.2 订阅详情查询
- **接口**: `POST /api/mqtt/subscribe/detail`
- **描述**: 查询订阅详情
- **请求参数**: 空 JSON 对象
- **响应**: 当前返回空字符串（功能待实现）

#### 5.3 自动订阅规则管理

##### 5.3.1 自动订阅列表
- **接口**: `POST /api/mqtt/auto-subscribe/list`
- **描述**: 查询自动订阅规则列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

##### 5.3.2 创建自动订阅规则
- **接口**: `POST /api/mqtt/auto-subscribe/create`
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

##### 5.3.3 删除自动订阅规则
- **接口**: `POST /api/mqtt/auto-subscribe/delete`
- **描述**: 删除自动订阅规则
- **请求参数**:
```json
{
  "topic_name": "system/+"
}
```

- **响应**: 成功返回 "success"

#### 5.4 慢订阅监控

##### 5.4.1 慢订阅列表
- **接口**: `POST /api/mqtt/slow-subscribe/list`
- **描述**: 查询慢订阅列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

---

### 6. 用户管理

#### 6.1 用户列表查询
- **接口**: `POST /api/mqtt/user/list`
- **描述**: 查询 MQTT 用户列表
- **请求参数**:
```json
{
  "user_name": "admin",             // 可选，按用户名过滤
  "limit": 20,
  "page": 1,
  "sort_field": "username",         // 可选，排序字段
  "sort_by": "asc",
  "filter_field": "username",
  "filter_values": ["admin"],
  "exact_match": "false"
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "username": "admin",
        "is_superuser": true
      }
    ],
    "total_count": 10
  }
}
```

#### 6.2 创建用户
- **接口**: `POST /api/mqtt/user/create`
- **描述**: 创建新的 MQTT 用户
- **请求参数**:
```json
{
  "username": "newuser",            // 用户名
  "password": "password123",        // 密码
  "is_superuser": false             // 是否为超级用户
}
```

- **响应**: 成功返回 "Created successfully!"

#### 6.3 删除用户
- **接口**: `POST /api/mqtt/user/delete`
- **描述**: 删除 MQTT 用户
- **请求参数**:
```json
{
  "username": "olduser"
}
```

- **响应**: 成功返回 "Deleted successfully!"

---

### 7. ACL 管理

#### 7.1 ACL 列表查询
- **接口**: `POST /api/mqtt/acl/list`
- **描述**: 查询访问控制列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

#### 7.2 创建 ACL 规则
- **接口**: `POST /api/mqtt/acl/create`
- **描述**: 创建新的 ACL 规则
- **请求参数**:
```json
{
  "resource_type": "ClientId",       // 资源类型：ClientId, Username, IpAddress
  "resource_name": "client001",      // 资源名称
  "topic": "sensor/+",               // 主题模式
  "ip": "192.168.1.100",             // IP地址
  "action": "Publish",               // 动作：Publish, Subscribe, All
  "permission": "Allow"              // 权限：Allow, Deny
}
```

- **响应**: 成功返回 "Created successfully!"

#### 7.3 删除 ACL 规则
- **接口**: `POST /api/mqtt/acl/delete`
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

- **响应**: 成功返回 "Deleted successfully!"

---

### 8. 黑名单管理

#### 8.1 黑名单列表查询
- **接口**: `POST /api/mqtt/blacklist/list`
- **描述**: 查询黑名单列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

#### 8.2 创建黑名单
- **接口**: `POST /api/mqtt/blacklist/create`
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

- **响应**: 成功返回 "Created successfully!"

#### 8.3 删除黑名单
- **接口**: `POST /api/mqtt/blacklist/delete`
- **描述**: 删除黑名单项
- **请求参数**:
```json
{
  "blacklist_type": "ClientId",
  "resource_name": "bad_client"
}
```

- **响应**: 成功返回 "Deleted successfully!"

---

### 9. 连接器管理

#### 9.1 连接器列表查询
- **接口**: `POST /api/mqtt/connector/list`
- **描述**: 查询连接器列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

#### 9.2 创建连接器
- **接口**: `POST /api/mqtt/connector/create`
- **描述**: 创建新的连接器
- **请求参数**:
```json
{
  "connector_name": "new_connector",   // 连接器名称
  "connector_type": "Kafka",           // 连接器类型
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\"}",  // 配置信息（JSON字符串）
  "topic_id": "sensor/+"               // 关联的主题ID
}
```

**连接器类型和配置示例**：

**Kafka 连接器**:
```json
{
  "connector_type": "Kafka",
  "config": "{\"bootstrap_servers\":\"localhost:9092\",\"topic\":\"mqtt_messages\",\"acks\":\"all\"}"
}
```

**文件连接器**:
```json
{
  "connector_type": "File",
  "config": "{\"path\":\"/tmp/mqtt_messages.log\",\"max_file_size\":\"100MB\"}"
}
```

- **响应**: 成功返回 "Created successfully!"

#### 9.3 删除连接器
- **接口**: `POST /api/mqtt/connector/delete`
- **描述**: 删除连接器
- **请求参数**:
```json
{
  "connector_name": "old_connector"
}
```

- **响应**: 成功返回 "Deleted successfully!"

---

### 10. Schema 管理

#### 10.1 Schema 列表查询
- **接口**: `POST /api/mqtt/schema/list`
- **描述**: 查询 Schema 列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "name": "temperature_schema",
        "schema_type": "json",
        "desc": "Temperature sensor data schema",
        "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"},\"unit\":{\"type\":\"string\"}}}"
      }
    ],
    "total_count": 12
  }
}
```

#### 10.2 创建 Schema
- **接口**: `POST /api/mqtt/schema/create`
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
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\",\"minimum\":-50,\"maximum\":100}}}"
}
```

**AVRO Schema**:
```json
{
  "schema_type": "avro", 
  "schema": "{\"type\":\"record\",\"name\":\"SensorData\",\"fields\":[{\"name\":\"temperature\",\"type\":\"double\"}]}"
}
```

- **响应**: 成功返回 "Created successfully!"

#### 10.3 删除 Schema
- **接口**: `POST /api/mqtt/schema/delete`
- **描述**: 删除 Schema
- **请求参数**:
```json
{
  "schema_name": "old_schema"
}
```

- **响应**: 成功返回 "Deleted successfully!"

#### 10.4 Schema 绑定管理

##### 10.4.1 Schema 绑定列表查询
- **接口**: `POST /api/mqtt/schema-bind/list`
- **描述**: 查询 Schema 绑定关系列表
- **请求参数**:
```json
{
  "resource_name": "sensor/temperature", // 可选，资源名称过滤
  "schema_name": "temp_schema",          // 可选，Schema名称过滤
  "limit": 20,
  "page": 1,
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
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "data_type": "resource",
        "data": ["sensor_data_schema", "device_status_schema"]
      }
    ],
    "total_count": 2
  }
}
```

##### 10.4.2 创建 Schema 绑定
- **接口**: `POST /api/mqtt/schema-bind/create`
- **描述**: 创建 Schema 与资源的绑定关系
- **请求参数**:
```json
{
  "schema_name": "sensor_data_schema",  // Schema名称
  "resource_name": "sensor/temperature" // 资源名称（通常是主题名）
}
```

- **响应**: 成功返回 "Created successfully!"

##### 10.4.3 删除 Schema 绑定
- **接口**: `POST /api/mqtt/schema-bind/delete`
- **描述**: 删除 Schema 绑定关系
- **请求参数**:
```json
{
  "schema_name": "sensor_data_schema",
  "resource_name": "sensor/temperature"
}
```

- **响应**: 成功返回 "Deleted successfully!"

---

### 11. 系统监控

#### 11.1 系统告警列表
- **接口**: `POST /api/mqtt/system-alarm/list`
- **描述**: 查询系统告警列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
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
}
```

#### 11.2 连接抖动检测列表
- **接口**: `POST /api/mqtt/flapping_detect/list`
- **描述**: 查询连接抖动检测列表
- **请求参数**: 支持通用分页和过滤参数
- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "client_id": "flapping_client",
        "before_last_windows_connections": 15,
        "first_request_time": 1640995200
      }
    ],
    "total_count": 2
  }
}
```

---

## 枚举值说明

### ACL 资源类型 (resource_type)
- `ClientId`: 客户端ID
- `Username`: 用户名
- `IpAddress`: IP地址

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
- `Kafka`: Kafka消息队列
- `File`: 本地文件

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
curl -X POST http://localhost:8080/api/mqtt/overview \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 查询客户端列表
```bash
curl -X POST http://localhost:8080/api/mqtt/client/list \
  -H "Content-Type: application/json" \
  -d '{
    "limit": 10,
    "page": 1,
    "sort_field": "connection_id",
    "sort_by": "desc"
  }'
```

### 创建用户
```bash
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "testpass123",
    "is_superuser": false
  }'
```

### 创建ACL规则
```bash
curl -X POST http://localhost:8080/api/mqtt/acl/create \
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
curl -X POST http://localhost:8080/api/mqtt/connector/create \
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
curl -X POST http://localhost:8080/api/mqtt/schema/create \
  -H "Content-Type: application/json" \
  -d '{
    "schema_name": "sensor_schema",
    "schema_type": "json",
    "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"},\"humidity\":{\"type\":\"number\"}}}",
    "desc": "Sensor data validation schema"
  }'
```

---

*文档版本: v4.0*  
*最后更新: 2025-09-20*  
*基于代码版本: RobustMQ Admin Server v0.1.34*
