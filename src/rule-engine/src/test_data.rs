// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bytes::Bytes;
use serde_json::{Map, Value};

pub fn gateway_source_json_str() -> &'static str {
    r#"
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
"#
}

pub fn gateway_source_json_bytes() -> Bytes {
    Bytes::from(gateway_source_json_str())
}

pub fn gateway_source_json_map() -> Map<String, Value> {
    let value: Value = serde_json::from_str(gateway_source_json_str()).unwrap();
    value.as_object().unwrap().clone()
}
