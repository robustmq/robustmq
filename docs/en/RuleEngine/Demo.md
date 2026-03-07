# Rule Engine Processing Demo

This example shows one full chain:

- source payload (input)
- `etl_rule` configuration
- output payload (result)

## Source Payload (Input)

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

## Rule Config (`etl_rule`)

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

## Output Payload (Result)

```json
{
  "alarm_active": true,
  "missing": "-",
  "mqtt_topic": "factory/a/line3/meter/44001/data",
  "wan_ip": "10.8.12.34"
}
```

## Notes

- `Extract` currently uses **projection output** semantics: only mapped target fields are kept.
- `field_mapping` source path supports:
  - dot path (e.g. `session.mqtt.topic`)
  - JSON Pointer (e.g. `/payload/alarms/0/active`)
- missing source values are filled with `"-"`.
