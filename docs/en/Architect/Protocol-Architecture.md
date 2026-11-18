# Protocol Codec Architecture

This document describes the internal `src/protocol` crate. It is an implementation reference for contributors; user-facing protocol behavior belongs in the MQTT, Kafka, NATS, and AMQP documentation.

## Responsibilities

The crate owns wire-level data types and codecs for:

- MQTT 3.1, 3.1.1, and 5.0
- Kafka requests and responses
- AMQP 0-9-1 frames
- NATS Core commands and messages
- the internal Storage Engine protocol

Protocol business behavior remains in the corresponding broker crates. The codec crate does not perform topic routing, authentication decisions, subscription management, or storage operations.

## Unified Dispatch

`RobustMQCodec` in `src/protocol/src/codec.rs` wraps the protocol-specific codecs and exposes one `tokio_util::codec::{Decoder, Encoder}` interface. Decoded values are returned as `RobustMQCodecWrapper` variants:

| Variant | Payload |
|---|---|
| `MQTT` | `MqttPacketWrapper`, including the negotiated MQTT version |
| `KAFKA` | `KafkaPacketWrapper` |
| `AMQP` | one `AMQPFrame` |
| `NATS` | `NatsPacket` |
| `StorageEngine` | `StorageEnginePacket` |

Network acceptors for a known protocol construct the codec with `RobustMQCodec::new_with_protocol`. MQTT listeners use a codec that starts without a version so the first `CONNECT` packet can select MQTT 3, 4, or 5. A codec created with no preset protocol does not perform general-purpose detection for Kafka, AMQP, NATS, or Storage Engine traffic.

## Protocol Identifiers

`RobustMQProtocol` is an internal routing identifier, not a wire-protocol registry. Its numeric values are used by connection metadata and internal packet wrappers:

| Protocol | Internal value |
|---|---:|
| MQTT3 / MQTT4 / MQTT5 | 3 / 4 / 5 |
| Kafka | 0 |
| Storage Engine | 10 |
| AMQP | 11 |
| NATS | 12 |

`from_u8` is intentionally a compatibility fallback: it recognizes MQTT4, MQTT5, and NATS, and maps all other values to MQTT3. It must not be treated as a reversible conversion for every value returned by `to_u8`.

## Framing and Partial Reads

All codecs preserve incomplete bytes in the input buffer and return `Ok(None)` until a complete frame or command is available. AMQP and Kafka use length-bearing binary frames. NATS uses CRLF-terminated control lines followed by a payload and trailing CRLF for `PUB`, `HPUB`, `MSG`, and `HMSG`. MQTT uses its fixed header and variable-length remaining-length field.

## Error Handling

Protocol-specific codecs return their own errors when used directly. Through `RobustMQCodec`, MQTT, Kafka, AMQP, and Storage Engine decode failures are currently treated as no decoded packet in the dispatch path, while NATS decode failures are converted to `CommonError` and propagated so the connection can be closed instead of waiting indefinitely for more bytes. Callers that need precise protocol errors should use the protocol-specific codec directly.

## Scope Boundary

This crate defines wire representations and serialization. The broker crates decide what a decoded packet means and whether it is accepted, rejected, persisted, acknowledged, or delivered. Changes to protocol semantics should update the relevant protocol compatibility documentation and tests in addition to this internal reference.
