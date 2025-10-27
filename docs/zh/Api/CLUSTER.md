# 集群管理 HTTP API

> 本文档介绍集群配置和管理相关的 HTTP API 接口。通用信息请参考 [COMMON.md](COMMON.md)。

## 集群配置管理

### 1. 获取集群配置

- **接口**: `POST /api/cluster/config/get`
- **描述**: 获取当前集群的配置信息
- **请求参数**:
```json
{}
```

- **响应示例**:
```json
{
  "code": 0,
  "message": "success",
  "data": "配置信息的JSON字符串"
}
```

- **配置信息结构** (BrokerConfig):
```json
{
  "cluster_name": "robustmq-cluster",
  "broker_id": 1,
  "meta_service": {
    "addr": ["127.0.0.1:1228"],
    "timeout_ms": 30000
  },
  "network": {
    "tcp_port": 1883,
    "tls_port": 1885,
    "ws_port": 8083,
    "wss_port": 8085,
    "quic_port": 9083
  },
  "mqtt_offline_message": {
    "enable": true,
    "storage_type": "memory",
    "max_num": 1000
  },
  "mqtt_slow_subscribe": {
    "enable": true,
    "threshold_ms": 1000
  },
  "mqtt_system_monitor": {
    "enable": true,
    "os_cpu_high_watermark": 80.0,
    "os_cpu_low_watermark": 40.0,
    "os_memory_high_watermark": 80.0,
    "os_cpu_check_interval_ms": 5000
  },
  "mqtt_flapping_detect": {
    "enable": true,
    "window_time": 60,
    "max_client_connections": 10,
    "ban_time": 300
  }
}
```

### 2. 设置集群配置

- **接口**: `POST /api/cluster/config/set`
- **描述**: 设置集群配置
- **请求参数**:
```json
{
  "config_type": "broker",           // 配置类型，目前支持 "broker"
  "config": "{\"cluster_name\":\"new-cluster\",\"broker_id\":2,...}"  // 完整的配置JSON字符串
}
```

- **响应示例**:
```json
{
  "code": 0,
  "message": "success",
  "data": "Configuration updated successfully"
}
```

---

## 使用示例

### 获取集群配置
```bash
curl -X POST http://localhost:8080/api/cluster/config/get \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 设置集群配置
```bash
# 设置离线消息功能
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "broker",
    "config": "{\"mqtt_offline_message\":{\"enable\":true,\"storage_type\":\"rocksdb\",\"max_num\":10000}}"
  }'
```

### 启用系统监控
```bash
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "broker",
    "config": "{\"mqtt_system_monitor\":{\"enable\":true,\"os_cpu_high_watermark\":90.0,\"os_cpu_low_watermark\":30.0}}"
  }'
```

### 配置连接抖动检测
```bash
curl -X POST http://localhost:8080/api/cluster/config/set \
  -H "Content-Type: application/json" \
  -d '{
    "config_type": "broker",
    "config": "{\"mqtt_flapping_detect\":{\"enable\":true,\"window_time\":120,\"max_client_connections\":20,\"ban_time\":600}}"
  }'
```

---

## 配置项说明

### 离线消息配置 (mqtt_offline_message)
| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | `bool` | 是否启用离线消息 |
| `storage_type` | `string` | 存储类型：memory, rocksdb, mysql |
| `max_num` | `u32` | 最大离线消息数量 |

### 慢订阅配置 (mqtt_slow_subscribe)
| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | `bool` | 是否启用慢订阅检测 |
| `threshold_ms` | `u64` | 慢订阅阈值（毫秒） |

### 系统监控配置 (mqtt_system_monitor)
| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | `bool` | 是否启用系统监控 |
| `os_cpu_high_watermark` | `f32` | CPU 高水位线（百分比） |
| `os_cpu_low_watermark` | `f32` | CPU 低水位线（百分比） |
| `os_memory_high_watermark` | `f32` | 内存高水位线（百分比） |
| `os_cpu_check_interval_ms` | `u64` | CPU 检查间隔（毫秒） |

### 连接抖动检测配置 (mqtt_flapping_detect)
| 字段 | 类型 | 说明 |
|------|------|------|
| `enable` | `bool` | 是否启用连接抖动检测 |
| `window_time` | `u64` | 时间窗口（秒） |
| `max_client_connections` | `u32` | 最大连接次数 |
| `ban_time` | `u64` | 封禁时间（秒） |

---

## 注意事项

1. **配置格式**: 配置必须是有效的 JSON 字符串
2. **配置验证**: 服务会验证配置的格式和有效性
3. **热更新**: 大部分配置支持热更新，无需重启服务
4. **备份建议**: 修改配置前建议先获取当前配置进行备份
5. **权限要求**: 配置修改通常需要管理员权限

---

*文档版本: v4.0*
*最后更新: 2025-09-20*
*基于代码版本: RobustMQ Admin Server v0.1.34*
