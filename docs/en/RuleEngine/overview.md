# Rule Engine Overview

RobustMQ Rule Engine provides lightweight in-connector data processing.
It is not designed to replace stream-compute systems like Flink. The goal is to cover common transformation needs in MQTT and connector pipelines with lower operational complexity.

## Processing Model

The runtime pipeline is:

`decode -> ops[] -> encode`

- `decode`: normalize source payload into `Vec<Map<String, Value>>`
- `ops[]`: run ordered operators on normalized records
- `encode`: serialize transformed records back to `Bytes`

## Current First Operator

The first implemented operator is `Extract`.

- Input: `field_mapping` (`source_field -> target_field`)
- Supports source path styles:
  - dot path, e.g. `session.mqtt.topic`
  - JSON Pointer, e.g. `/payload/alarms/0/active`
- If source field is missing, target value is filled with `"-"`
- Current `Extract` semantics are **projection output**: only mapped target fields are kept

## Related Example

See the end-to-end demo here:

- [Rule Engine Processing Demo](/en/RuleEngine/Demo)
