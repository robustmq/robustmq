---
name: connector-delivery
description: Implements new RobustMQ MQTT connector integrations end-to-end using project conventions. Use when the user asks to add, implement, or support a new connector type such as webhook, opentsdb, clickhouse, influxdb, cassandra, mqtt bridge, or protocol-compatible targets.
---

# Connector Delivery

## Purpose

Deliver a new RobustMQ connector end-to-end with consistent patterns across:

- metadata struct and validation
- connector runtime implementation
- connector type registration and dispatch wiring
- admin API parsing and validation
- documentation and verification

Use this skill when the user asks: "add/support/implement <connector>".

## Input Contract

Collect only missing essentials. If omitted, apply defaults and proceed.

1. Connector type name (`s3`, `kinesis`, `foo`).
2. Direction (default: sink/outbound).
3. Required config fields.
4. MQTT record -> target payload mapping.

### Default assumptions

- Direction: sink
- Batch behavior: one `send_batch` call handles one pulled batch
- Retry/failure policy: reuse existing framework, do not invent per-connector policy
- Serialization: JSON unless target protocol requires specific format

## Fast-path command

If user says only "implement X connector", do this without extra questions:

1. Build minimal usable config with strict validation.
2. Implement sink writing with clear, deterministic mapping.
3. Wire all registration points.
4. Update both zh/en API and overview docs.
5. Run targeted `cargo check`.

## Implementation Workflow

### 1) Metadata Config

Files:

- `src/common/metadata-struct/src/connector/config_<type>.rs`
- `src/common/metadata-struct/src/connector/mod.rs`

Requirements:

1. Add config struct with `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`.
2. Add `Default` with practical defaults.
3. Add `validate(&self) -> Result<(), CommonError>`:
   - required fields non-empty
   - protocol/URL format checks
   - numeric range checks
   - dependent field pair checks
4. Export module in `mod.rs`.

Validation style:

- Prefer explicit error messages with exact field names.
- Keep bounds conservative and consistent with existing connectors.

### 2) ConnectorType Registration

File:

- `src/common/metadata-struct/src/connector/connector_type.rs`

Required edits:

1. Add `CONNECTOR_TYPE_<TYPE>` constant.
2. Add enum variant in `ConnectorType`.
3. Extend `as_str()`.
4. Extend `FromStr` with default config constructor.
5. Add import for new config type.

Rule: no partial registration. All four points must be updated.

### 3) Runtime Connector Module

Files:

- `src/connector/src/<module>/mod.rs`
- optional dependency updates in `src/connector/Cargo.toml`

Implement `ConnectorSink`:

- `validate()`: call config validation
- `init_sink()`: build client/operator/resource
- `send_batch()`: convert and send batch
- `cleanup_sink()`: optional if resource needs shutdown

Mapping rules:

- Keep payload mapping deterministic.
- Preserve important fields (`key`, `headers`, `tags`, `timestamp`) when reasonable.
- Do not hide conversion loss; if lossy, document it.

Error rules:

- Use `CommonError` with actionable messages.
- Do not silently swallow target write errors.

### 4) Runtime Wiring

Files:

- `src/connector/src/lib.rs`
- `src/connector/src/core.rs`

Checklist:

- Module exposed in `src/connector/src/lib.rs` or module tree.
- Startup path can construct the new sink.
- Branching by connector type includes new variant.

### 5) Admin API Wiring

File:

- `src/admin-server/src/mqtt/connector.rs`

Required edits:

1. Allow connector type in `validate_connector_type`.
2. Parse config in `parse_connector_type`.
3. Ensure `validate()` is called on decoded config.

### 6) Failure Strategy Semantics

Never change global semantics while adding a connector.

Must preserve:

- `discard`: terminal, commit offset.
- `discard_after_retry`: retry then terminal.
- `dead_message_queue`:
  - retry first;
  - write DLQ only after retries exhausted;
  - DLQ write failure returns error and continues retry;
  - only terminal-success path should allow offset commit.

### 7) Documentation Sync

Update all relevant docs when connector is user-facing.

Files:

- `docs/zh/RobustMQ-MQTT/Bridge/Overview.md`
- `docs/en/RobustMQ-MQTT/Bridge/Overview.md`
- `docs/zh/Api/Connector.md`
- `docs/en/Api/Connector.md`
- Sidebar entries only if adding new pages.

Document:

- connector purpose,
- config schema,
- examples,
- protocol-compatibility notes (if support is via compatible protocol).

### 8) Validation and Checks

Run after edits:

1. `cargo check -p metadata-struct -p connector -p admin-server`
2. Lint check for touched Rust files.
3. Confirm no missing match arms for `ConnectorType`.
4. Confirm docs include type list + config section + example request.

## File Matrix (must touch)

- Metadata config: `config_<type>.rs`
- Metadata mod export: `connector/mod.rs`
- Connector type enum: `connector/connector_type.rs`
- Runtime module: `src/connector/src/<module>/mod.rs`
- Runtime exports/dispatch: `src/connector/src/lib.rs`, `src/connector/src/core.rs`
- Admin parse/validate: `src/admin-server/src/mqtt/connector.rs`
- Docs: zh/en API + zh/en Overview

## Output Format

Return concise delivery report:

1. Files changed.
2. Behavior and defaults.
3. Known limitations.
4. Suggested next verification command(s).

Use this completion template:

```markdown
Implemented `<type>` connector end-to-end.

- Changed files: ...
- Runtime behavior: ...
- Config defaults: ...
- Limitations: ...
- Verify: `cargo check -p metadata-struct -p connector -p admin-server`
```

## Guardrails

- Reuse existing connector patterns before inventing new abstractions.
- Keep changes minimal and localized.
- Preserve backward compatibility for serialized config where possible.
- If protocol/client constraints block implementation, stop and explain blocker with alternatives.

## Common Pitfalls

- Added new config file but forgot `ConnectorType` `FromStr` branch.
- Added `ConnectorType` variant but forgot runtime dispatch in `core.rs`.
- Added runtime module but forgot `pub mod` export in `lib.rs`.
- Added code but forgot admin `validate_connector_type` whitelist.
- Docs updated in one language only.
