# MQTT Broker 管理 HTTP API

> 本文档介绍 MQTT Broker 相关的所有 HTTP API 接口。通用信息请参考 [COMMON.md](COMMON.md)。

## API 接口列表

### 1. 集群概览

#### 1.1 集群概览信息
- **接口**: `GET /api/mqtt/overview`
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
        "roles": ["mqtt-broker"],
        "extend": [],
        "node_id": 1,
        "node_ip": "192.168.1.100",
        "grpc_addr": "192.168.1.100:9981",
        "engine_addr": "192.168.1.100:9982",
        "start_time": 1640995200,
        "register_time": 1640995200,
        "storage_fold": []
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
    "exclusive_subscribe_thread_num": 8,
    "share_subscribe_group_num": 50,
    "share_subscribe_num": 500,
    "share_subscribe_thread_num": 50,
    "connector_num": 5,
    "connector_thread_num": 3
  }
}
```

**字段说明**：
- `node_list`: 集群节点列表
  - `roles`: 节点角色列表
  - `extend`: 扩展信息（字节数组）
  - `node_id`: 节点ID
  - `node_ip`: 节点IP地址
  - `grpc_addr`: gRPC通信地址
  - `engine_addr`: 存储引擎地址
  - `start_time`: 节点启动时间戳
  - `register_time`: 节点注册时间戳
  - `storage_fold`: 存储目录列表
- `cluster_name`: 集群名称
- `message_in_rate`: 消息接收速率（消息/秒）
- `message_out_rate`: 消息发送速率（消息/秒）
- `connection_num`: 总连接数（所有租户、所有类型连接的总和）
- `session_num`: 会话总数（所有租户之和）
- `topic_num`: 主题总数（所有租户之和）
- `placement_status`: Placement Center 状态（Leader/Follower）
- `tcp_connection_num`: TCP 连接数
- `tls_connection_num`: TLS 连接数
- `websocket_connection_num`: WebSocket 连接数
- `quic_connection_num`: QUIC 连接数
- `subscribe_num`: 订阅总数（所有租户所有订阅之和）
- `exclusive_subscribe_num`: 独占订阅总数
- `exclusive_subscribe_thread_num`: 独占订阅推送线程数
- `share_subscribe_group_num`: 共享订阅组数量
- `share_subscribe_num`: 共享订阅总数
- `share_subscribe_thread_num`: 共享订阅推送线程数
- `connector_num`: 连接器总数（所有租户之和）
- `connector_thread_num`: 活跃连接器线程数

#### 1.2 监控数据查询
- **接口**: `GET /api/mqtt/monitor/data`
- **描述**: 获取指定类型的监控数据时间序列
- **请求参数**:
```json
{
  "data_type": "connection_num",      // 必填，监控数据类型
  "topic_name": "sensor/temperature", // 可选，部分类型需要
  "client_id": "client001",           // 可选，部分类型需要
  "path": "sensor/+",                 // 可选，部分类型需要
  "connector_name": "kafka_conn_01"   // 可选，连接器监控类型需要
}
```

**支持的监控数据类型 (data_type)**：

**基础监控类型**（无需额外参数）：
- `connection_num` - 连接数
- `topic_num` - 主题数
- `subscribe_num` - 订阅数
- `message_in_num` - 消息接收数
- `message_out_num` - 消息发送数
- `message_drop_num` - 消息丢弃数

**主题级监控类型**（需要 `topic_name`）：
- `topic_in_num` - 指定主题的接收消息数
- `topic_out_num` - 指定主题的发送消息数

**订阅级监控类型**（需要 `client_id` 和 `path`）：
- `subscribe_send_success_num` - 订阅发送成功消息数
- `subscribe_send_failure_num` - 订阅发送失败消息数

**订阅主题级监控类型**（需要 `client_id`、`path` 和 `topic_name`）：
- `subscribe_topic_send_success_num` - 订阅指定主题发送成功消息数
- `subscribe_topic_send_failure_num` - 订阅指定主题发送失败消息数

**会话级监控类型**（需要 `client_id`）：
- `session_in_num` - 会话接收消息数
- `session_out_num` - 会话发送消息数

**连接器监控类型**：
- `connector_send_success_total` - 所有连接器发送成功消息总数（无需额外参数）
- `connector_send_failure_total` - 所有连接器发送失败消息总数（无需额外参数）
- `connector_send_success` - 指定连接器发送成功消息数（需要 `connector_name`）
- `connector_send_failure` - 指定连接器发送失败消息数（需要 `connector_name`）

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": [
    {
      "date": 1640995200,
      "value": 1500
    },
    {
      "date": 1640995260,
      "value": 1520
    }
  ]
}
```

**字段说明**：
- `date`: Unix 时间戳（秒）
- `value`: 该时间点的监控数值

**注意事项**：
- 数据保留时长：默认保留最近 1 小时的数据
- 数据采样间隔：根据系统配置，通常为 60 秒
- 如果缺少必需参数，将返回空数组
- 返回的数据按时间戳自然排序

---

### 2. 客户端管理

#### 2.1 客户端列表查询
- **接口**: `GET /api/mqtt/client/list`
- **描述**: 查询连接到集群的客户端列表，支持按租户过滤和 client_id 模糊搜索
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户精确过滤，指定时直接查询该租户缓存（性能更好） |
| `client_id` | string | 否 | 按客户端 ID 模糊搜索（包含匹配） |
| `limit` | u32 | 否 | 每页大小，最大采样 100 条 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段，支持 `client_id`、`connection_id` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |

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
          "tenant": "default",
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
  - `tenant`: 租户名称
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
  - 字段同会话管理接口响应

- **heartbeat**: 连接心跳信息（不可用时为 null）
  - `protocol`: MQTT 协议版本（Mqtt3, Mqtt4, Mqtt5）
  - `keep_live`: 保活间隔（秒）
  - `heartbeat`: 最后心跳时间戳

- **total_count**: 该租户（或全集群）的实际连接总数

---

### 3. 会话管理

#### 3.1 会话列表查询
- **接口**: `GET /api/mqtt/session/list`
- **描述**: 查询 MQTT 会话列表，支持按租户过滤和 client_id 模糊搜索
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户精确过滤，指定时直接查询该租户缓存（性能更好） |
| `client_id` | string | 否 | 按客户端 ID 模糊搜索（包含匹配） |
| `limit` | u32 | 否 | 每页大小，最大采样 100 条 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段，支持 `client_id`、`create_time` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "client_id": "client001",
        "session_expiry": 3600,
        "is_contain_last_will": true,
        "last_will_delay_interval": 30,
        "create_time": 1640995200,
        "connection_id": 12345,
        "broker_id": 1,
        "reconnect_time": 1640995300,
        "distinct_time": 1640995400,
        "last_will": {
          "client_id": "client001",
          "last_will": {
            "topic": "device/client001/status",
            "message": "offline",
            "qos": "AtLeastOnce",
            "retain": true
          },
          "last_will_properties": {
            "delay_interval": 30,
            "payload_format_indicator": 0,
            "message_expiry_interval": 3600,
            "content_type": "text/plain",
            "response_topic": null,
            "correlation_data": null,
            "user_properties": []
          }
        }
      }
    ],
    "total_count": 50
  }
}
```

**字段说明**：

- `client_id`: MQTT 客户端ID
- `session_expiry`: 会话过期间隔（秒）
- `is_contain_last_will`: 会话是否包含遗愿消息
- `last_will_delay_interval`: 遗愿消息延迟间隔（秒，可选）
- `create_time`: 会话创建时间戳
- `connection_id`: 关联的连接ID（可选）
- `broker_id`: 托管会话的 Broker 节点ID（可选）
- `reconnect_time`: 最后重连时间戳（可选）
- `distinct_time`: 最后断开连接时间戳（可选）
- `last_will`: 遗愿消息信息（无遗愿消息时为 null）
  - `client_id`: 客户端ID
  - `last_will`: 遗愿消息内容
    - `topic`: 遗愿消息主题
    - `message`: 遗愿消息内容
    - `qos`: QoS 级别（`AtMostOnce`/`AtLeastOnce`/`ExactlyOnce`）
    - `retain`: 是否为保留消息
  - `last_will_properties`: 遗愿消息属性（MQTT 5.0，可为 null）
- **total_count**: 该租户（或全集群）的实际会话总数

---

### 4. 主题管理

#### 4.1 主题列表查询
- **接口**: `GET /api/mqtt/topic/list`
- **描述**: 查询 MQTT 主题列表，支持按租户过滤、主题名模糊搜索和主题类型过滤
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户精确过滤，指定时直接查询该租户缓存（性能更好） |
| `topic_name` | string | 否 | 按主题名模糊搜索（包含匹配） |
| `topic_type` | string | 否 | 主题类型：`all`（默认）、`normal`（普通主题）、`system`（系统主题，含 `$`） |
| `limit` | u32 | 否 | 每页大小 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段，支持 `topic_name`、`tenant` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "topic_id": "01J9K5FHQP8NWXYZ1234567890",
        "topic_name": "sensor/temperature",
        "tenant": "default",
        "storage_type": "Memory",
        "partition": 1,
        "replication": 1,
        "storage_name_list": [],
        "create_time": 1640995200
      }
    ],
    "total_count": 25
  }
}
```

#### 4.2 主题详情查询
- **接口**: `GET /api/mqtt/topic/detail`
- **描述**: 查询指定主题的详细信息，包括主题基本信息、保留消息和订阅列表
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 是 | 租户名称 |
| `topic_name` | string | 是 | 主题名称（精确匹配） |


- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "topic_info": {
      "topic_id": "01J9K5FHQP8NWXYZ1234567890",
      "topic_name": "sensor/temperature",
      "tenant": "default",
      "storage_type": "Memory",
      "partition": 1,
      "replication": 1,
      "storage_name_list": [],
      "create_time": 1640995200
    },
    "retain_message": { ... },
    "retain_message_at": 1640995300,
    "sub_list": [
      { "client_id": "client001", "path": "sensor/temperature" }
    ],
    "storage_list": { ... }
  }
}
```

#### 4.3 删除主题
- **接口**: `POST /api/mqtt/topic/delete`
- **描述**: 删除指定的主题
- **请求参数**:

| 字段 | 类型 | 必填 | 校验 | 说明 |
|------|------|------|------|------|
| `tenant` | string | 是 | - | 租户名称 |
| `topic_name` | string | 是 | 长度 1-256 | 要删除的主题名称 |


- **响应**: 成功返回 "success"

**注意事项**：
- 删除主题会删除该主题的所有数据，包括保留消息
- 此操作不可逆，请谨慎使用

#### 4.4 主题重写规则列表
- **接口**: `GET /api/mqtt/topic-rewrite/list`
- **描述**: 查询主题重写规则列表，支持按租户过滤
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户精确过滤 |
| `limit` | u32 | 否 | 每页大小 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段，支持 `source_topic`、`dest_topic`、`action`、`tenant` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |
| `filter_field` | string | 否 | 过滤字段名 |
| `filter_values` | string[] | 否 | 过滤值列表 |
| `exact_match` | string | 否 | 是否精确匹配：`exact` |


- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
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

#### 4.5 创建主题重写规则
- **接口**: `POST /api/mqtt/topic-rewrite/create`
- **描述**: 创建新的主题重写规则
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称，长度 1-128
  "action": "All",                  // 必填，动作类型：All, Publish, Subscribe
  "source_topic": "old/topic/+",   // 必填，源主题模式，长度 1-256
  "dest_topic": "new/topic/$1",    // 必填，目标主题模式，长度 1-256
  "regex": "^old/topic/(.+)$"      // 必填，正则表达式，长度 1-500
}
```

- **响应**: 成功返回 "success"

#### 4.6 删除主题重写规则
- **接口**: `POST /api/mqtt/topic-rewrite/delete`
- **描述**: 删除主题重写规则
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称
  "action": "All",
  "source_topic": "old/topic/+"
}
```

- **响应**: 成功返回 "success"

---

### 5. 订阅管理

#### 5.1 订阅列表查询
- **接口**: `GET /api/mqtt/subscribe/list`
- **描述**: 查询订阅列表，支持按租户过滤和 client_id 模糊搜索
- **请求参数**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `tenant` | string | 否 | 按租户精确过滤，指定时直接查询该租户缓存（性能更好） |
| `client_id` | string | 否 | 按客户端 ID 模糊搜索（包含匹配） |
| `limit` | u32 | 否 | 每页大小 |
| `page` | u32 | 否 | 页码，从 1 开始 |
| `sort_field` | string | 否 | 排序字段，支持 `client_id`、`tenant` |
| `sort_by` | string | 否 | 排序方向：`asc` / `desc` |


- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
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

**新增字段**：
- `tenant`: 该订阅所属的租户名称

#### 5.2 订阅详情查询
- **接口**: `GET /api/mqtt/subscribe/detail`
- **描述**: 查询订阅详情，支持查询独占订阅和共享订阅的详细信息
- **请求参数**:
```json
{
  "tenant": "default",          // 必填，租户名称
  "client_id": "client001",     // 必填，客户端ID
  "path": "sensor/temperature"  // 必填，订阅路径
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "share_sub": false,
    "group_leader_info": null,
    "sub_data": {
      "client_id": "client001",
      "path": "sensor/temperature",
      "push_subscribe": {
        "sensor/temperature": {
          "client_id": "client001",
          "sub_path": "sensor/temperature",
          "rewrite_sub_path": null,
          "tenant": "default",
          "topic_name": "sensor/temperature",
          "group_name": "",
          "protocol": "MQTTv5",
          "qos": "AtLeastOnce",
          "no_local": false,
          "preserve_retain": true,
          "retain_forward_rule": "SendAtSubscribe",
          "subscription_identifier": null,
          "create_time": 1704067200
        }
      },
      "push_thread": {
        "sensor/temperature": {
          "push_success_record_num": 1520,
          "push_error_record_num": 3,
          "last_push_time": 1704067800,
          "last_run_time": 1704067810,
          "create_time": 1704067200,
          "bucket_id": "bucket_0"
        }
      },
      "leader_id": null
    }
  }
}
```

#### 5.3 自动订阅规则管理

##### 5.3.1 自动订阅列表
- **接口**: `GET /api/mqtt/auto-subscribe/list`
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
        "tenant": "default",
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

**新增字段**：
- `tenant`: 自动订阅规则所属的租户名称

##### 5.3.2 创建自动订阅规则
- **接口**: `POST /api/mqtt/auto-subscribe/create`
- **描述**: 创建新的自动订阅规则
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称，长度 1-256
  "topic": "system/+",              // 必填，主题模式，长度 1-256
  "qos": 1,                         // 必填，QoS 级别：0, 1, 2
  "no_local": false,                // 必填，是否本地
  "retain_as_published": false,     // 必填，保持发布状态
  "retained_handling": 0            // 必填，保留消息处理方式：0, 1, 2
}
```

- **响应**: 成功返回 "success"

##### 5.3.3 删除自动订阅规则
- **接口**: `POST /api/mqtt/auto-subscribe/delete`
- **描述**: 删除自动订阅规则
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称
  "topic": "system/+"               // 必填，主题模式
}
```

- **响应**: 成功返回 "success"

#### 5.4 慢订阅监控

##### 5.4.1 慢订阅列表
- **接口**: `GET /api/mqtt/slow-subscribe/list`
- **描述**: 查询慢订阅列表，支持按租户过滤
- **请求参数**:
```json
{
  "tenant": "default",              // 可选，按租户过滤（指定后仅扫描该租户的记录）
  "limit": 20,
  "page": 1,
  "sort_field": "time_span",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["slow_client"],
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
        "tenant": "default",
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

**字段说明**：

- `tenant`: 所属租户
- `client_id`: 客户端ID
- `topic_name`: 订阅主题
- `time_span`: 慢订阅耗时（毫秒）
- `node_info`: 所在节点信息
- `create_time`: 记录创建时间（本地时间格式）
- `subscribe_name`: 订阅名称

---

### 6. 用户管理

#### 6.1 用户列表查询
- **接口**: `GET /api/mqtt/user/list`
- **描述**: 查询 MQTT 用户列表，支持按租户过滤、用户名模糊搜索
- **请求参数**:
```json
{
  "tenant": "default",              // 可选，按租户精确过滤
  "user_name": "admin",             // 可选，按用户名模糊搜索（包含匹配）
  "limit": 20,
  "page": 1,
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
  "code": 0,
  "message": "success",
  "data": {
    "data": [
      {
        "tenant": "default",
        "username": "admin",
        "is_superuser": true,
        "create_time": 1640995200
      }
    ],
    "total_count": 10
  }
}
```

**字段说明**：
- `tenant`: 用户所属租户
- `username`: 用户名
- `is_superuser`: 是否为超级用户
- `create_time`: 用户创建时间戳（秒）

#### 6.2 创建用户
- **接口**: `POST /api/mqtt/user/create`
- **描述**: 创建新的 MQTT 用户
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称，长度 1-64
  "username": "newuser",            // 必填，用户名，长度 1-64
  "password": "password123",        // 必填，密码，长度 1-128
  "is_superuser": false             // 必填，是否为超级用户
}
```

- **响应**: 成功返回 "success"

#### 6.3 删除用户
- **接口**: `POST /api/mqtt/user/delete`
- **描述**: 删除 MQTT 用户
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称，长度 1-64
  "username": "olduser"             // 必填，用户名，长度 1-64
}
```

- **响应**: 成功返回 "success"

---

### 7. ACL 管理

#### 7.1 ACL 列表查询
- **接口**: `GET /api/mqtt/acl/list`
- **描述**: 查询访问控制列表，支持按租户直接查询
- **请求参数**:
```json
{
  "tenant": "default",              // 可选，按租户过滤（直接查询，性能更好）
  "limit": 20,
  "page": 1,
  "sort_field": "resource_name",
  "sort_by": "asc",
  "filter_field": "action",
  "filter_values": ["Publish"],
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
        "tenant": "default",
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

**新增字段**：
- `tenant`: ACL 规则所属的租户名称

#### 7.2 创建 ACL 规则
- **接口**: `POST /api/mqtt/acl/create`
- **描述**: 创建新的 ACL 规则
- **请求参数**:
```json
{
  "tenant": "default",               // 必填，租户名称，长度 1-64
  "resource_type": "ClientId",       // 必填，资源类型：ClientId, User, Ip
  "resource_name": "client001",      // 必填，资源名称，长度 1-256
  "topic": "sensor/+",               // 必填，主题模式，长度 1-256
  "ip": "192.168.1.100",             // 必填，IP地址，长度不超过 128
  "action": "Publish",               // 必填，动作：Publish, Subscribe, All
  "permission": "Allow"              // 必填，权限：Allow, Deny
}
```

- **参数验证规则**:
  - `resource_type`: 必须是 `ClientId`、`User` 或 `Ip`
  - `action`: 必须是 `Publish`、`Subscribe` 或 `All`
  - `permission`: 必须是 `Allow` 或 `Deny`

- **响应**: 成功返回 "success"

#### 7.3 删除 ACL 规则
- **接口**: `POST /api/mqtt/acl/delete`
- **描述**: 删除 ACL 规则
- **请求参数**:
```json
{
  "tenant": "default",
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

### 8. 黑名单管理

#### 8.1 黑名单列表查询
- **接口**: `GET /api/mqtt/blacklist/list`
- **描述**: 查询黑名单列表，支持按租户直接查询
- **请求参数**:
```json
{
  "tenant": "default",              // 可选，按租户过滤（直接查询，性能更好）
  "limit": 20,
  "page": 1,
  "sort_field": "resource_name",
  "sort_by": "asc",
  "filter_field": "blacklist_type",
  "filter_values": ["ClientId"],
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
        "tenant": "default",
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

**新增字段**：
- `tenant`: 黑名单所属的租户名称

#### 8.2 创建黑名单
- **接口**: `POST /api/mqtt/blacklist/create`
- **描述**: 添加新的黑名单项
- **请求参数**:
```json
{
  "tenant": "default",                 // 必填，租户名称，长度 1-64
  "blacklist_type": "ClientId",        // 必填，黑名单类型（见枚举说明）
  "resource_name": "bad_client",       // 必填，资源名称，长度 1-256
  "end_time": 1735689599,              // 必填，结束时间（Unix时间戳），必须大于 0
  "desc": "Blocked for security"       // 可选，描述，长度不超过 500
}
```

- **参数验证规则**:
  - `blacklist_type`: 必须是 `ClientId`、`User`、`Ip`、`ClientIdMatch`、`UserMatch` 或 `IPCIDR`
  - `end_time`: 必须大于 0

- **响应**: 成功返回 "success"

#### 8.3 删除黑名单
- **接口**: `POST /api/mqtt/blacklist/delete`
- **描述**: 删除黑名单项
- **请求参数**:
```json
{
  "tenant": "default",
  "blacklist_type": "ClientId",
  "resource_name": "bad_client"
}
```

- **响应**: 成功返回 "success"

---

### 9. 连接器管理

> 连接器 API 内容较多，已独立为单独文档，请参考 [连接器管理 HTTP API](Connector.md)。

**主要变更**：连接器的 `list`、`detail`、`delete` 接口均新增 `tenant` 参数，详见 Connector.md。

---

### 10. Schema 管理

#### 10.1 Schema 列表查询
- **接口**: `GET /api/mqtt/schema/list`
- **描述**: 查询 Schema 列表，支持按租户直接查询
- **请求参数**:
```json
{
  "tenant": "default",              // 可选，按租户过滤（直接查询，性能更好）
  "limit": 20,
  "page": 1,
  "sort_field": "name",
  "sort_by": "asc",
  "filter_field": "schema_type",
  "filter_values": ["json"],
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
        "tenant": "default",
        "name": "temperature_schema",
        "schema_type": "json",
        "desc": "Temperature sensor data schema",
        "schema": "{\"type\":\"object\",\"properties\":{\"temp\":{\"type\":\"number\"}}}"
      }
    ],
    "total_count": 12
  }
}
```

**新增字段**：
- `tenant`: Schema 所属的租户名称

#### 10.2 创建 Schema
- **接口**: `POST /api/mqtt/schema/create`
- **描述**: 创建新的 Schema
- **请求参数**:
```json
{
  "tenant": "default",                   // 必填，租户名称，长度 1-128
  "schema_name": "sensor_data_schema",   // 必填，Schema名称，长度 1-128
  "schema_type": "json",                 // 必填，Schema类型：json, avro, protobuf
  "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"}}}",
  "desc": "Sensor data validation schema"  // 可选，描述，长度不超过 500
}
```

- **响应**: 成功返回 "success"

#### 10.3 删除 Schema
- **接口**: `POST /api/mqtt/schema/delete`
- **描述**: 删除 Schema
- **请求参数**:
```json
{
  "tenant": "default",              // 必填，租户名称
  "schema_name": "old_schema"       // 必填，Schema名称
}
```

- **响应**: 成功返回 "success"

#### 10.4 Schema 绑定管理

##### 10.4.1 Schema 绑定列表查询
- **接口**: `GET /api/mqtt/schema-bind/list`
- **描述**: 查询 Schema 绑定关系列表
- **请求参数**:
```json
{
  "tenant": "default",                    // 可选，租户名称
  "resource_name": "sensor/temperature",  // 可选，资源名称过滤
  "schema_name": "temp_schema",           // 可选，Schema名称过滤
  "limit": 20,
  "page": 1,
  "sort_field": "data_type",
  "sort_by": "asc"
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
  "tenant": "default",                   // 必填，租户名称，长度 1-128
  "schema_name": "sensor_data_schema",   // 必填，Schema名称，长度 1-128
  "resource_name": "sensor/temperature"  // 必填，资源名称（通常是主题名），长度 1-256
}
```

- **响应**: 成功返回 "success"

##### 10.4.3 删除 Schema 绑定
- **接口**: `POST /api/mqtt/schema-bind/delete`
- **描述**: 删除 Schema 绑定关系
- **请求参数**:
```json
{
  "tenant": "default",
  "schema_name": "sensor_data_schema",
  "resource_name": "sensor/temperature"
}
```

- **响应**: 成功返回 "success"

---

### 11. 消息管理

#### 11.1 发送消息
- **接口**: `POST /api/mqtt/message/send`
- **描述**: 通过HTTP API发送MQTT消息到指定主题
- **请求参数**:
```json
{
  "tenant": "default",            // 必填，租户名称，长度 1-256
  "topic": "sensor/temperature",  // 必填，主题名称，长度 1-256
  "payload": "25.5",              // 必填，消息内容，不超过 1MB
  "retain": false                 // 可选，是否保留消息，默认false
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "offsets": [12345]
  }
}
```

**注意事项**：
- 发送的消息使用QoS 1（至少一次）级别
- 如果主题不存在，系统会自动创建
- 消息默认过期时间为3600秒（1小时）

#### 11.2 读取消息
- **接口**: `POST /api/mqtt/message/read`
- **描述**: 从指定主题读取消息
- **请求参数**:
```json
{
  "topic": "sensor/temperature",  // 必填，主题名称
  "offset": 0                     // 必填，起始offset
}
```

- **响应数据结构**:
```json
{
  "code": 0,
  "message": "success",
  "data": {
    "messages": [
      {
        "offset": 12345,
        "content": "25.5",
        "timestamp": 1640995200000
      }
    ]
  }
}
```

**注意事项**：
- 每次请求最多返回100条消息
- 时间戳为毫秒级Unix时间戳

---

### 12. 系统监控

#### 12.1 系统告警列表
- **接口**: `GET /api/mqtt/system-alarm/list`
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
        "create_time": 1640995200
      }
    ],
    "total_count": 3
  }
}
```

#### 12.2 连接抖动检测列表

- **接口**: `GET /api/mqtt/flapping_detect/list`
- **描述**: 查询连接抖动检测列表，支持按租户过滤
- **请求参数**:

```json
{
  "tenant": "default",              // 可选，按租户过滤（指定后仅返回该租户数据）
  "limit": 20,
  "page": 1,
  "sort_field": "first_request_time",
  "sort_by": "desc",
  "filter_field": "client_id",
  "filter_values": ["flapping_client"],
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
        "tenant": "default",
        "client_id": "flapping_client",
        "before_last_window_connections": 15,
        "first_request_time": 1640995200
      }
    ],
    "total_count": 2
  }
}
```

**字段说明**：

- `tenant`: 所属租户
- `client_id`: 客户端ID
- `before_last_window_connections`: 上一个统计窗口内的连接次数
- `first_request_time`: 首次触发时间戳（秒）

#### 12.3 封禁日志列表

- **接口**: `GET /api/mqtt/ban-log/list`
- **描述**: 查询客户端封禁日志，支持按租户过滤
- **请求参数**:

```json
{
  "tenant": "default",              // 可选，按租户过滤（指定后仅扫描该租户的记录）
  "limit": 20,
  "page": 1,
  "sort_field": "create_time",
  "sort_by": "desc",
  "filter_field": "resource_name",
  "filter_values": ["bad_client"],
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
        "tenant": "default",
        "ban_type": "ClientId",
        "resource_name": "bad_client_001",
        "ban_source": "manual",
        "end_time": "2024-12-31 23:59:59",
        "create_time": "2024-01-01 10:00:00"
      }
    ],
    "total_count": 5
  }
}
```

**字段说明**：

- `tenant`: 所属租户
- `ban_type`: 封禁类型（`ClientId` / `User` / `Ip` / `ClientIdMatch` / `UserMatch` / `IPCIDR`）
- `resource_name`: 被封禁的资源名称（客户端ID、用户名或IP）
- `ban_source`: 封禁来源（如 `manual` 或 `auto`）
- `end_time`: 封禁到期时间（本地时间格式）
- `create_time`: 封禁创建时间（本地时间格式）

---

### 13. 租户管理

> MQTT Broker 支持多租户隔离。租户是资源（用户、ACL、主题、订阅等）的顶级命名空间。

#### 13.1 租户列表查询
- **接口**: `GET /api/mqtt/tenant/list`
- **描述**: 查询 MQTT 租户列表，支持按租户名直接查询
- **请求参数**:
```json
{
  "tenant_name": "default",         // 可选，按租户名精确查询
  "limit": 20,
  "page": 1,
  "sort_field": "tenant_name",      // 可选，排序字段
  "sort_by": "asc",
  "filter_field": "tenant_name",
  "filter_values": ["default"],
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
        "tenant_name": "default",
        "desc": "Default tenant",
        "create_time": 1640995200,
        "max_connections_per_node": 1000000,
        "max_create_connection_rate_per_second": 10000,
        "max_topics": 500000,
        "max_sessions": 5000000,
        "max_publish_rate": 10000
      }
    ],
    "total_count": 3
  }
}
```

**字段说明**：
- `tenant_name`: 租户名称
- `desc`: 租户描述
- `create_time`: 租户创建时间戳（秒）
- `max_connections_per_node`: 每节点最大连接数
- `max_create_connection_rate_per_second`: 每秒最大新建连接速率
- `max_topics`: 最大 Topic 数量
- `max_sessions`: 最大 Session 数量
- `max_publish_rate`: 每秒最大 Publish 消息速率

#### 13.2 创建租户
- **接口**: `POST /api/mqtt/tenant/create`
- **描述**: 创建新的 MQTT 租户
- **请求参数**:
```json
{
  "tenant_name": "new_tenant",                        // 必填，租户名称，长度 1-128
  "desc": "Production environment",                   // 可选，租户描述，长度不超过 500
  "max_connections_per_node": 1000000,                // 可选，每节点最大连接数
  "max_create_connection_rate_per_second": 10000,     // 可选，每秒最大新建连接速率
  "max_topics": 500000,                               // 可选，最大 Topic 数量
  "max_sessions": 5000000,                            // 可选，最大 Session 数量
  "max_publish_rate": 10000                           // 可选，每秒最大 Publish 消息速率
}
```

- **响应**: 成功返回 "success"

#### 13.3 删除租户
- **接口**: `POST /api/mqtt/tenant/delete`
- **描述**: 删除 MQTT 租户
- **请求参数**:
```json
{
  "tenant_name": "old_tenant"        // 必填，租户名称，长度 1-128
}
```

- **响应**: 成功返回 "success"

**注意事项**：
- 删除租户前请确认该租户下的用户、ACL、订阅等资源已清理
- 系统默认租户 `default` 不建议删除

---

## 枚举值说明

### ACL 资源类型 (resource_type)
- `ClientId`: 客户端ID
- `User`: 用户名
- `Ip`: IP地址

### ACL 动作 (action)
- `Publish`: 发布消息
- `Subscribe`: 订阅主题
- `All`: 所有动作

### ACL 权限 (permission)
- `Allow`: 允许
- `Deny`: 拒绝

### 黑名单类型 (blacklist_type)
- `ClientId`: 精确匹配客户端ID
- `User`: 精确匹配用户名
- `Ip`: 精确匹配IP地址
- `ClientIdMatch`: 通配符匹配客户端ID（支持 `*` 通配符）
- `UserMatch`: 通配符匹配用户名（支持 `*` 通配符）
- `IPCIDR`: CIDR 网段匹配（如 `192.168.1.0/24`）

### 连接器类型 (connector_type)

> 完整的连接器类型列表和配置参数请参考 [连接器管理 HTTP API](Connector.md#枚举值参考)。

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
curl -X GET http://localhost:8080/api/mqtt/overview
```

### 查询指定租户的客户端
```bash
curl "http://localhost:8080/api/mqtt/client/list?tenant=default&limit=10&page=1"
```

### 查询指定租户的会话
```bash
curl "http://localhost:8080/api/mqtt/session/list?tenant=default&limit=20&page=1"
```

### 查询订阅列表（指定租户）
```bash
curl "http://localhost:8080/api/mqtt/subscribe/list?tenant=default&limit=20&page=1"
```

### 创建租户
```bash
curl -X POST http://localhost:8080/api/mqtt/tenant/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_name": "production",
    "desc": "Production environment tenant"
  }'
```

### 删除主题
```bash
curl -X POST http://localhost:8080/api/mqtt/topic/delete \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "topic_name": "sensor/temperature"
  }'
```

### 创建用户
```bash
curl -X POST http://localhost:8080/api/mqtt/user/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
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
    "tenant": "default",
    "resource_type": "ClientId",
    "resource_name": "sensor001",
    "topic": "sensor/+",
    "ip": "*",
    "action": "Publish",
    "permission": "Allow"
  }'
```

### 创建黑名单（通配符匹配）
```bash
curl -X POST http://localhost:8080/api/mqtt/blacklist/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "blacklist_type": "ClientIdMatch",
    "resource_name": "bad_client_*",
    "end_time": 1735689599,
    "desc": "Block all clients matching bad_client_*"
  }'
```

### 创建Schema
```bash
curl -X POST http://localhost:8080/api/mqtt/schema/create \
  -H "Content-Type: application/json" \
  -d '{
    "tenant": "default",
    "schema_name": "sensor_schema",
    "schema_type": "json",
    "schema": "{\"type\":\"object\",\"properties\":{\"temperature\":{\"type\":\"number\"}}}",
    "desc": "Sensor data validation schema"
  }'
```

### 查询监控数据
```bash
# 查询连接数监控数据
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=connection_num"

# 查询指定主题的消息接收数
curl "http://localhost:8080/api/mqtt/monitor/data?data_type=topic_in_num&topic_name=sensor/temperature"
```

### 发送消息
```bash
curl -X POST http://localhost:8080/api/mqtt/message/send \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "sensor/temperature",
    "payload": "25.5",
    "retain": false
  }'
```

---

*文档版本: v5.0*
*最后更新: 2026-03-15*
*基于代码版本: RobustMQ Admin Server v0.1.35*
