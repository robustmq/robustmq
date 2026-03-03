# MQTT 桥接连接器

## 概述

MQTT 桥接连接器是 RobustMQ 提供的数据集成组件，用于将本地 MQTT 消息转发到远程 MQTT Broker。该连接器实现了 Sink（出站）方向的 MQTT 桥接，支持 MQTT 3.1、3.1.1 和 5.0 协议，适用于跨集群消息同步、多层 IoT 架构数据上报、边缘到云消息转发等场景。

## 功能特性

- 支持 MQTT 3.1 / 3.1.1 / 5.0 协议
- 支持 TLS 加密连接
- 支持用户名/密码认证
- 支持 QoS 0/1/2
- 支持自定义 topic 前缀
- 支持消息保留标志
- 支持批量消息转发

## 配置说明

### 连接器配置

MQTT 桥接连接器使用 `MqttBridgeConnectorConfig` 结构进行配置：

```rust
pub struct MqttBridgeConnectorConfig {
    pub server: String,                       // 远程 MQTT Broker 地址
    pub client_id_prefix: Option<String>,     // 客户端 ID 前缀
    pub username: Option<String>,             // 用户名
    pub password: Option<String>,             // 密码
    pub protocol_version: MqttProtocolVersion, // 协议版本（V3/V4/V5）
    pub keepalive_secs: u64,                  // 心跳间隔（秒）
    pub connect_timeout_secs: u64,            // 连接超时（秒）
    pub enable_tls: bool,                     // 是否启用 TLS
    pub topic_prefix: Option<String>,         // Topic 前缀
    pub qos: i32,                             // QoS 等级（0/1/2）
    pub retain: bool,                         // 是否保留消息
    pub max_retries: u32,                     // 最大重试次数
}
```

### 配置参数

| 参数名 | 类型 | 必填 | 默认值 | 说明 | 示例 |
|--------|------|------|--------|------|------|
| `server` | String | 是 | - | 远程 MQTT Broker 地址，长度不超过 512 字符 | `tcp://broker.example.com:1883` |
| `client_id_prefix` | String | 否 | `robustmq-bridge` | 客户端 ID 前缀，长度不超过 64 字符 | `my-bridge` |
| `username` | String | 否 | - | 连接用户名 | `admin` |
| `password` | String | 否 | - | 连接密码 | `password123` |
| `protocol_version` | String | 否 | `v5` | MQTT 协议版本：`v3`、`v4`、`v5` | `v5` |
| `keepalive_secs` | Number | 否 | `60` | 心跳间隔（秒），范围：1-65535 | `60` |
| `connect_timeout_secs` | Number | 否 | `10` | 连接超时时间（秒），范围：1-300 | `10` |
| `enable_tls` | Boolean | 否 | `false` | 是否启用 TLS 加密 | `true` |
| `topic_prefix` | String | 否 | - | 转发消息的 Topic 前缀 | `remote/` |
| `qos` | Number | 否 | `1` | 消息 QoS 等级：0、1、2 | `1` |
| `retain` | Boolean | 否 | `false` | 是否设置消息保留标志 | `false` |
| `max_retries` | Number | 否 | `3` | 发送失败最大重试次数，范围：0-10 | `3` |

### 配置示例

#### JSON 配置格式

**基础配置**
```json
{
  "server": "tcp://remote-broker:1883"
}
```

**带认证和 TLS 配置**
```json
{
  "server": "ssl://remote-broker:8883",
  "username": "bridge_user",
  "password": "bridge_pass",
  "enable_tls": true,
  "protocol_version": "v5",
  "keepalive_secs": 30,
  "connect_timeout_secs": 15
}
```

**带 Topic 前缀和 QoS 配置**
```json
{
  "server": "tcp://remote-broker:1883",
  "client_id_prefix": "edge-bridge",
  "topic_prefix": "cloud/edge01",
  "qos": 2,
  "retain": true,
  "max_retries": 5
}
```

#### 完整连接器配置

```json
{
  "cluster_name": "default",
  "connector_name": "mqtt_bridge_01",
  "connector_type": "mqtt",
  "config": "{\"server\": \"tcp://remote-broker:1883\", \"username\": \"admin\", \"password\": \"secret\", \"topic_prefix\": \"remote/\", \"qos\": 1}",
  "topic_name": "sensor/data",
  "status": "Idle",
  "broker_id": null,
  "create_time": 1640995200,
  "update_time": 1640995200
}
```

## Topic 映射规则

MQTT 桥接连接器支持 Topic 前缀功能，用于在转发消息时重写目标 Topic：

| 源 Topic | topic_prefix | 目标 Topic |
|----------|-------------|-----------|
| `sensor/temperature` | 无 | `sensor/temperature` |
| `sensor/temperature` | `remote/` | `remote/sensor/temperature` |
| `device/status` | `cloud/edge01` | `cloud/edge01/device/status` |

## 使用 robust-ctl 创建 MQTT 桥接连接器

### 基本语法

```bash
robust-ctl mqtt connector create \
  --connector-name <连接器名称> \
  --connector-type <连接器类型> \
  --config <配置> \
  --topic-id <主题ID>
```

### 创建示例

#### 1. 基本创建命令

```bash
robust-ctl mqtt connector create \
  --connector-name "mqtt_bridge_01" \
  --connector-type "mqtt" \
  --config '{"server": "tcp://remote-broker:1883"}' \
  --topic-id "sensor/data"
```

#### 2. 带认证的 MQTT 桥接

```bash
robust-ctl mqtt connector create \
  --connector-name "mqtt_bridge_auth" \
  --connector-type "mqtt" \
  --config '{"server": "ssl://remote-broker:8883", "username": "bridge_user", "password": "bridge_pass", "enable_tls": true, "qos": 2}' \
  --topic-id "device/status"
```

#### 3. 带 Topic 前缀的桥接

```bash
robust-ctl mqtt connector create \
  --connector-name "mqtt_bridge_prefix" \
  --connector-type "mqtt" \
  --config '{"server": "tcp://remote-broker:1883", "topic_prefix": "cloud/edge01", "client_id_prefix": "edge-bridge"}' \
  --topic-id "sensor/#"
```

### 管理连接器

```bash
# 列出所有连接器
robust-ctl mqtt connector list

# 查看指定连接器
robust-ctl mqtt connector list --connector-name "mqtt_bridge_01"

# 删除连接器
robust-ctl mqtt connector delete --connector-name "mqtt_bridge_01"
```

### 完整操作示例

#### 场景：边缘到云消息转发

```bash
# 1. 创建传感器数据桥接
robust-ctl mqtt connector create \
  --connector-name "edge_to_cloud_sensors" \
  --connector-type "mqtt" \
  --config '{"server": "ssl://cloud-broker:8883", "username": "edge01", "password": "secret", "enable_tls": true, "topic_prefix": "cloud/edge01", "qos": 1}' \
  --topic-id "sensor/+"

# 2. 创建设备状态桥接
robust-ctl mqtt connector create \
  --connector-name "edge_to_cloud_status" \
  --connector-type "mqtt" \
  --config '{"server": "ssl://cloud-broker:8883", "username": "edge01", "password": "secret", "enable_tls": true, "topic_prefix": "cloud/edge01", "qos": 1}' \
  --topic-id "device/status"

# 3. 查看创建的连接器
robust-ctl mqtt connector list
```

## 性能优化建议

### 1. 连接设置
- 根据网络延迟合理设置 `connect_timeout_secs`
- 对于广域网桥接，适当增加 `keepalive_secs`
- 使用唯一的 `client_id_prefix` 避免客户端冲突

### 2. QoS 选择
- 对于高吞吐但可容忍丢失的场景使用 QoS 0
- 对于需要可靠传输的场景使用 QoS 1
- 仅在需要精确一次语义时使用 QoS 2（性能开销较大）

### 3. 安全建议
- 生产环境建议启用 TLS（`enable_tls: true`）
- 使用独立的桥接专用用户名/密码
- 定期轮换认证凭证

## 监控和故障排查

### 1. 查看连接器状态

```bash
robust-ctl mqtt connector list --connector-name "mqtt_bridge_01"
```

### 2. 常见问题

**问题 1：连接失败**
- 检查远程 MQTT Broker 是否正常运行
- 验证 `server` 地址和端口是否正确
- 检查网络连通性和防火墙配置
- 确认用户名/密码是否正确

**问题 2：消息丢失**
- 检查 QoS 设置是否满足需求
- 确认远程 Broker 的消息队列是否已满
- 查看连接器日志是否有发送错误

**问题 3：延迟过高**
- 检查网络延迟
- 适当降低 QoS 等级以提升吞吐
- 检查远程 Broker 负载情况

## 当前限制

- 目前仅支持 Sink（出站）方向，Source（入站）方向将在后续版本支持
- 暂不支持 Topic 通配符映射规则

## 总结

MQTT 桥接连接器是 RobustMQ 数据集成系统的重要组件，提供了跨 MQTT 集群消息转发的能力。通过简单的配置即可实现：

- **跨集群同步**：在多个 MQTT 集群之间同步消息
- **边缘到云**：将边缘设备消息转发到云端 Broker
- **协议兼容**：支持 MQTT 3.1 / 3.1.1 / 5.0，兼容各类 Broker
- **安全传输**：支持 TLS 加密和用户名/密码认证
