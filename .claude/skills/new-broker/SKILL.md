---
name: new-broker
description: Complete step-by-step guide for implementing a new protocol Broker in RobustMQ. Use when the user asks to add a new broker, implement a new protocol, or scaffold a new broker crate.
---

# new-broker

Complete steps for implementing a brand-new protocol Broker in RobustMQ.

## Architecture

Each Broker's responsibility is: **protocol parsing + orchestration of shared infrastructure components**. It does NOT implement storage, routing, or cluster communication itself.

```
Protocol packet → broker handler → shared components (storage-adapter / broker-core / node-call / rate-limit)
```

---

## Implementation Steps

### Step 1: Protocol layer — `src/protocol/src/<proto>/`

1. `packet.rs` — define all protocol packet data structures (refer to NATS `ServerInfo` / `ClientConnect` / `NatsPacket`)
2. `codec.rs` — implement `tokio_util::codec::Decoder` + `Encoder<Packet>`, requires `#[derive(Clone)]`
3. `mod.rs` — `pub mod codec; pub mod packet;`
4. Add `pub mod <proto>;` to `src/protocol/src/lib.rs`

**Codec notes:**
- `#[derive(Clone)]` required on the codec struct and all internal enums
- When enum variant sizes differ greatly, wrap large variants in `Box<>` to avoid `clippy::large_enum_variant`

### Step 2: Protocol registration — `src/protocol/src/robust.rs`

Add one variant in each location:

| Location | What to add |
|----------|-------------|
| `RobustMQProtocol` enum | `PROTO` variant; `to_u8()` assigns a unique u8; `to_str()` returns the name; `from_u8()` adds a branch; add `is_proto()` method |
| `XxxWrapperExtend` struct | Create `pub struct ProtoWrapperExtend {}` |
| `RobustMQWrapperExtend` enum | `PROTO(ProtoWrapperExtend)` variant; add branch in `to_mqtt_protocol()` |
| `RobustMQPacket` enum | `PROTO(ProtoPacket)` variant; add `get_proto_packet()` method |

### Step 3: Codec registration — `src/protocol/src/codec.rs`

1. Add `PROTO(ProtoPacket)` variant to `RobustMQCodecWrapper` enum; add branch in `Display` impl
2. Add `PROTO(ProtoCodec)` variant to `RobustMQCodecEnum` enum
3. Add `RobustMQProtocol::PROTO` to `PROTOCOL_PROBE_ORDER` constant array
4. Add `proto_codec: ProtoCodec` field to `RobustMQCodec` struct; initialize in `new()`
5. Add `Some(RobustMQProtocol::PROTO)` arm in `decode_data()` match on `self.protocol`
6. Add `RobustMQCodecWrapper::PROTO(pkt)` arm in `encode_data()` match

### Step 4: Fix exhaustive matches in network-server

The following files have exhaustive matches — add a `PROTO` branch in each (search for `RobustMQPacket::StorageEngine` to locate them):

- `src/common/network-server/src/common/handler.rs` — two places: `write_response()` and `write_websocket_response()`
- `src/common/network-server/src/common/write.rs` — two places: `write_tcp_frame()` and `write_quic_frame()`
- `src/common/network-server/src/common/tcp_acceptor.rs`
- `src/common/network-server/src/common/tls_acceptor.rs`
- `src/common/network-server/src/quic/acceptor.rs`
- `src/common/network-server/src/websocket/server.rs`

### Step 5: New broker crate — `src/<proto>-broker/`

```
src/<proto>-broker/
├── Cargo.toml
└── src/
    ├── lib.rs              — pub mod broker/handler/nats/server
    ├── broker.rs           — XxxBrokerServerParams + XxxBrokerServer (DEFAULT_PORT)
    ├── server/mod.rs       — XxxServer, TcpServer + handler_process
    ├── handler/
    │   ├── mod.rs
    │   └── command.rs      — XxxHandlerCommand impl Command, match dispatches to <proto>/ functions
    └── <proto>/
        ├── mod.rs
        ├── connect.rs      — process_connect()
        ├── publish.rs      — process_pub()
        ├── subscribe.rs    — process_sub() / process_unsub()
        └── ping.rs         — process_ping() / process_pong() (if the protocol has heartbeats)
```

**`<proto>/` directory design principles:**
- One file per semantically related group of commands (not one file per command)
- Function signatures accept protocol packet fields, return `Option<ProtoPacket>`
- Function bodies contain only TODO comments describing which shared components to call; business logic filled in later

**`command.rs` pattern:**
```rust
let resp_packet = match &packet {
    ProtoPacket::Connect(req) => connect::process_connect(req),
    ProtoPacket::Pub { subject, payload, .. } => publish::process_pub(subject, payload),
    // ...
}?;
Some(ResponsePackage::new(connection_id, RobustMQPacket::PROTO(resp_packet)))
```

### Step 6: Configuration — `src/common/config/src/`

1. `config.rs` — create `pub struct ProtoRuntime { pub network: Network }` with `Default impl`; add field to `BrokerConfig`; initialize in `BrokerConfig::default()`
2. `default.rs` — add `pub fn default_proto_runtime() -> ProtoRuntime`; add `ProtoRuntime` to use imports

### Step 7: Workspace registration

1. Root `Cargo.toml` — add `"src/<proto>-broker"` to `[workspace] members`; add `<proto>-broker = { path = "src/<proto>-broker" }` to `[workspace.dependencies]`
2. `src/broker-server/Cargo.toml` — add `<proto>-broker.workspace = true`

### Step 8: broker-server integration — `src/broker-server/src/lib.rs`

1. Add `use <proto>_broker::broker::{XxxBrokerServer, XxxBrokerServerParams};`
2. Add `proto_params: XxxBrokerServerParams` field to `BrokerServer` struct
3. Build `proto_params` in `new()` (refer to kafka_params / amqp_params)
4. Add `self.start_proto_broker(app_stop.clone());` in `start()` Phase 7
5. Implement `start_proto_broker()` method (refer to `start_kafka_broker`)

### Step 9: Verify

```bash
cargo build --workspace
```

Successful compilation confirms the scaffold is complete. Fill in business logic in the `<proto>/` directory functions afterward.

---

## Common Pitfalls

| Problem | Cause | Fix |
|---------|-------|-----|
| `Clone` compile error | Internal enum in codec struct missing `#[derive(Clone)]` | Add Clone to all enums inside the codec |
| `large_enum_variant` clippy error | Enum variant size difference exceeds threshold | Wrap large variants in `Box<>` |
| `non-exhaustive patterns` | match in network-server missing new branch | Search `StorageEngine` to locate all matches, add branch to each |
| `unresolved module metadata_struct` | Missing Cargo.toml dependency | Add `metadata-struct.workspace = true` |
| PR template CLA link broken | Relative path doesn't render correctly on GitHub | Change to `https://github.com/robustmq/robustmq/blob/main/CLA.md` |
