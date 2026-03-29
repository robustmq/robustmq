# new-broker

在 RobustMQ 中实现一个全新协议 Broker 的完整步骤。

## 架构理解

每个 Broker 的职责是：**协议解析 + 通用基础组件的编排**，不自己实现存储、路由、集群通信。

```
协议包 → broker handler → 调用通用组件（storage-adapter / broker-core / node-call / rate-limit）
```

---

## 完整实施步骤

### Step 1：协议层 — `src/protocol/src/<proto>/`

1. `packet.rs` — 定义所有协议包的数据结构（参考 NATS 的 `ServerInfo` / `ClientConnect` / `NatsPacket`）
2. `codec.rs` — 实现 `tokio_util::codec::Decoder` + `Encoder<Packet>`，需要 `#[derive(Clone)]`
3. `mod.rs` — `pub mod codec; pub mod packet;`
4. 在 `src/protocol/src/lib.rs` 加 `pub mod <proto>;`

**Codec 注意点：**
- 需要 `#[derive(Clone)]` 在 codec struct 及其所有内部 enum 上
- 内部 enum 大小差异大时加 `Box<>` 包大 variant，避免 `clippy::large_enum_variant` 报错

### Step 2：协议注册 — `src/protocol/src/robust.rs`

在以下位置各加一个 variant：

| 位置 | 加什么 |
|------|--------|
| `RobustMQProtocol` enum | `PROTO` variant，`to_u8()` 分配唯一 u8，`to_str()` 返回名字，`from_u8()` 加分支，加 `is_proto()` 方法 |
| `XxxWrapperExtend` struct | 新建 `pub struct ProtoWrapperExtend {}` |
| `RobustMQWrapperExtend` enum | `PROTO(ProtoWrapperExtend)` variant，`to_mqtt_protocol()` 加分支 |
| `RobustMQPacket` enum | `PROTO(ProtoPacket)` variant，加 `get_proto_packet()` 方法 |

### Step 3：Codec 注册 — `src/protocol/src/codec.rs`

1. `RobustMQCodecWrapper` enum 加 `PROTO(ProtoPacket)` variant，`Display` impl 加分支
2. `RobustMQCodecEnum` enum 加 `PROTO(ProtoCodec)` variant
3. `PROTOCOL_PROBE_ORDER` 常量数组加 `RobustMQProtocol::PROTO`
4. `RobustMQCodec` struct 加 `proto_codec: ProtoCodec` 字段，`new()` 初始化
5. `decode_data()` 的 `match self.protocol` 加 `Some(RobustMQProtocol::PROTO)` arm
6. `encode_data()` 的 match 加 `RobustMQCodecWrapper::PROTO(pkt)` arm

### Step 4：network-server 穷举修复

以下文件的 match 都是穷举的，每处加 `PROTO` 分支（搜 `RobustMQPacket::StorageEngine` 定位）：

- `src/common/network-server/src/common/handler.rs` — `write_response()` 和 `write_websocket_response()` 两处
- `src/common/network-server/src/common/write.rs` — `write_tcp_frame()` 和 `write_quic_frame()` 两处
- `src/common/network-server/src/common/tcp_acceptor.rs`
- `src/common/network-server/src/common/tls_acceptor.rs`
- `src/common/network-server/src/quic/acceptor.rs`
- `src/common/network-server/src/websocket/server.rs`

### Step 5：新建 Broker crate — `src/<proto>-broker/`

```
src/<proto>-broker/
├── Cargo.toml
└── src/
    ├── lib.rs              — pub mod broker/handler/nats/server
    ├── broker.rs           — XxxBrokerServerParams + XxxBrokerServer（DEFAULT_PORT）
    ├── server/mod.rs       — XxxServer，TcpServer + handler_process
    ├── handler/
    │   ├── mod.rs
    │   └── command.rs      — XxxHandlerCommand impl Command，match 调用 nats/ 各函数
    └── <proto>/
        ├── mod.rs
        ├── connect.rs      — process_connect()
        ├── publish.rs      — process_pub()
        ├── subscribe.rs    — process_sub() / process_unsub()
        └── ping.rs         — process_ping() / process_pong()（如协议有心跳）
```

**`<proto>/` 目录设计原则：**
- 每个文件对应一组语义相关的命令（不是一命令一文件）
- 函数签名接收协议包字段，返回 `Option<ProtoPacket>`
- 函数体只写 TODO 注释说明要调用哪些通用组件，业务逻辑后续填入

**`command.rs` 模式：**
```rust
let resp_packet = match &packet {
    ProtoPacket::Connect(req) => connect::process_connect(req),
    ProtoPacket::Pub { subject, payload, .. } => publish::process_pub(subject, payload),
    // ...
}?;
Some(ResponsePackage::new(connection_id, RobustMQPacket::PROTO(resp_packet)))
```

### Step 6：配置 — `src/common/config/src/`

1. `config.rs` — 新建 `pub struct ProtoRuntime { pub network: Network }`，加 `Default impl`，在 `BrokerConfig` 加字段，在 `BrokerConfig::default()` 初始化
2. `default.rs` — 加 `pub fn default_proto_runtime() -> ProtoRuntime`，在 use 里导入 `ProtoRuntime`

### Step 7：Workspace 注册

1. 根 `Cargo.toml` — `[workspace] members` 加 `"src/<proto>-broker"`，`[workspace.dependencies]` 加 `<proto>-broker = { path = "src/<proto>-broker" }`
2. `src/broker-server/Cargo.toml` — 加 `<proto>-broker.workspace = true`

### Step 8：broker-server 接入 — `src/broker-server/src/lib.rs`

1. use 加 `use <proto>_broker::broker::{XxxBrokerServer, XxxBrokerServerParams};`
2. `BrokerServer` struct 加 `proto_params: XxxBrokerServerParams` 字段
3. `new()` 中构建 `proto_params`（参考 kafka_params / amqp_params）
4. `start()` Phase 7 加 `self.start_proto_broker(app_stop.clone());`
5. 实现 `start_proto_broker()` 方法（参考 `start_kafka_broker`）

### Step 9：验证

```bash
cargo build --workspace
```

编译通过即骨架完成，业务逻辑后续在 `<proto>/` 目录各函数中填入。

---

## 常见坑

| 问题 | 原因 | 解法 |
|------|------|------|
| `Clone` 编译报错 | Codec struct 内部 enum 没加 `#[derive(Clone)]` | 给 codec 内所有 enum 加 Clone |
| `large_enum_variant` clippy 报错 | enum variant 大小差异超阈值 | 大 variant 用 `Box<>` 包裹 |
| `non-exhaustive patterns` | network-server 里的 match 没加新分支 | 搜 `StorageEngine` 定位所有 match，逐一补分支 |
| `unresolved module metadata_struct` | Cargo.toml 缺依赖 | 加 `metadata-struct.workspace = true` |
| PR 模板 CLA 链接失效 | 相对路径在 GitHub 渲染时不生效 | 改为 `https://github.com/robustmq/robustmq/blob/main/CLA.md` |
