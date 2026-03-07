# 规则引擎处理 Demo

下面给一个完整的链路示例，展示规则引擎如何做：

- 原始格式（输入）
- 配置规则（`etl_rule`）
- 输出格式（结果）

## 原始格式（输入）

```json
{
  "ts": "2026-03-03T12:34:56.789Z",
  "gateway": {
    "id": "gw-shanghai-01",
    "region": "cn-east-1",
    "firmware": {
      "version": "3.4.2",
      "build": "20260301-rc1"
    },
    "network": {
      "wan": {
        "ip": "10.8.12.34",
        "rtt_ms": 42
      },
      "lan": {
        "clients": 128
      }
    }
  },
  "session": {
    "conn_id": "c-9f4a2e",
    "client_id": "meter-44001",
    "username": "tenant_a.device_44001",
    "proto": "mqtt",
    "mqtt": {
      "qos": 1,
      "topic": "factory/a/line3/meter/44001/data",
      "retain": false
    }
  },
  "payload": {
    "seq": 982341,
    "metrics": [
      { "name": "voltage", "value": 221.7, "unit": "V" },
      { "name": "current", "value": 12.3, "unit": "A" },
      { "name": "temperature", "value": 36.5, "unit": "C" }
    ],
    "alarms": [
      { "code": "A101", "level": "warning", "active": true },
      { "code": "A205", "level": "critical", "active": false }
    ],
    "tags": {
      "line": "line3",
      "shift": "night"
    }
  },
  "processing": {
    "rule_chain": ["input_type", "filter", "set", "rename"],
    "trace": {
      "ingest_ns": 1709469296789000000,
      "rule_engine_ns": 1709469296789123456,
      "sink_ns": 1709469296789234567
    }
  }
}
```

## 配置规则（etl_rule）

```json
{
  "decode_rule": {
    "Decode": {
      "data_type": "JsonObject",
      "line_separator": null,
      "token_separator": null,
      "kv_separator": null
    }
  },
  "ops_rule_list": [
    {
      "Extract": {
        "field_mapping": {
          "gateway.network.wan.ip": "wan_ip",
          "/payload/alarms/0/active": "alarm_active",
          "not.exists.path": "missing",
          "session.mqtt.topic": "mqtt_topic"
        }
      }
    }
  ],
  "encode_rule": {
    "Encode": {
      "data_type": "JsonObject",
      "line_separator": null,
      "token_separator": null,
      "kv_separator": null
    }
  }
}
```

### 这段规则是什么意思

这条规则的目标是：从一条复杂原始消息里提取 4 个关键字段，输出一个精简结果对象。

- `decode_rule.Decode.data_type = JsonObject`
  - 输入按单条 JSON 对象解析。
- `ops_rule_list` 里只有一个 `Extract` 算子
  - 根据 `field_mapping` 做“提取 + 重命名”：
    - `gateway.network.wan.ip -> wan_ip`
    - `/payload/alarms/0/active -> alarm_active`
    - `session.mqtt.topic -> mqtt_topic`
    - `not.exists.path -> missing`
  - 不存在的源字段写 `"-"`。
- `encode_rule.Encode.data_type = JsonObject`
  - 把提取结果编码为 JSON 对象输出。

也就是：输入是完整大对象，输出是只包含业务关键字段的小对象，便于后续存储、转发和分析。

## 输出格式（结果）

```json
{
  "alarm_active": true,
  "missing": "-",
  "mqtt_topic": "factory/a/line3/meter/44001/data",
  "wan_ip": "10.8.12.34"
}
```

## 说明

- `Extract` 当前语义是**投影输出**：只输出映射后的目标字段，不保留原始字段。
- `field_mapping` 里的 key 支持两种路径写法：
  - 点路径（如 `session.mqtt.topic`）
  - JSON Pointer（如 `/payload/alarms/0/active`）
- 如果源字段不存在，默认填充 `"-"`。
