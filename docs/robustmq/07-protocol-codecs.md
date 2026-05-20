# 7. Protocol Codecs — `src/protocol`

Crate: [src/protocol](../../src/protocol)

This crate isolates **all wire-format work** (parsing, serialization, gRPC
protobuf-generated types) from broker logic. It builds against the `.proto` files via
[`build.rs`](../../src/protocol/build.rs).

## Layout

```
src/protocol/src
├── lib.rs
├── codec.rs    # generic framed-codec helpers (length-prefixed, varint)
├── robust.rs   # internal RobustMQ control protocol types
├── mqtt/       # MQTT 3.1 / 3.1.1 / 5.0 packet codecs
├── kafka/      # Kafka request/response codecs (API keys, versions)
├── nats/       # NATS line protocol + headers
├── amqp/       # AMQP 0-9-1 frame codecs
├── meta/       # gRPC types for Meta Service (Tenant, Topic, ACL, Cluster, ...)
├── broker/     # gRPC types between brokers and other components
└── storage/    # gRPC types for engine ↔ adapter and engine ↔ engine
```

## Two codec families

1. **External wire protocols** (`mqtt/`, `kafka/`, `nats/`, `amqp/`):
   - Implemented as `tokio_util::codec::{Encoder, Decoder}` so they slot into
     `common/network-server` listeners.
   - Stateless decoders where possible; per-version dispatch (e.g. MQTT 3 vs 5,
     Kafka API version) handled by small enums.
2. **Internal gRPC** (`meta/`, `broker/`, `storage/`):
   - Generated from `.proto` definitions via `tonic-build` in `build.rs`.
   - Consumed by `src/grpc-clients` (client side) and each crate’s `server/` module
     (server side).

## Why this separation matters

- Brokers never `unsafe`-parse bytes; they get already-typed packet enums.
- Adding a new protocol means **one new subdirectory here + one new `*-broker` crate**.
- Re-using `tokio_util::codec` makes TLS/WebSocket/QUIC composition free: the same codec
  runs on top of any `AsyncRead + AsyncWrite` transport in
  [common/network-server](../../src/common/network-server).

Continue to [Supporting Services](08-supporting-services.md).
