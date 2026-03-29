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

use super::packet::{ClientConnect, NatsPacket, ServerInfo};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt;
use tokio_util::codec;

#[derive(Debug)]
pub enum NatsCodecError {
    Io(std::io::Error),
    InvalidUtf8(std::str::Utf8Error),
    InvalidJson(serde_json::Error),
    UnknownCommand(String),
    MalformedCommand(String),
    PayloadTooLarge(usize),
}

impl fmt::Display for NatsCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NatsCodecError::Io(e) => write!(f, "I/O error: {}", e),
            NatsCodecError::InvalidUtf8(e) => write!(f, "Invalid UTF-8: {}", e),
            NatsCodecError::InvalidJson(e) => write!(f, "Invalid JSON: {}", e),
            NatsCodecError::UnknownCommand(s) => write!(f, "Unknown command: {}", s),
            NatsCodecError::MalformedCommand(s) => write!(f, "Malformed command: {}", s),
            NatsCodecError::PayloadTooLarge(n) => write!(f, "Payload too large: {} bytes", n),
        }
    }
}

impl std::error::Error for NatsCodecError {}

impl From<std::io::Error> for NatsCodecError {
    fn from(e: std::io::Error) -> Self {
        NatsCodecError::Io(e)
    }
}

#[derive(Debug, Clone)]
enum DecodeState {
    /// Waiting for the first `\r\n`-terminated control line.
    ReadingControl,
    /// Control line parsed; waiting for `payload_len + 2` more bytes.
    ReadingPayload {
        packet: PartialPacket,
        payload_len: usize,
    },
}

/// Holds the parsed control-line fields for packets that carry a payload.
#[derive(Debug, Clone)]
enum PartialPacket {
    Pub {
        subject: String,
        reply_to: Option<String>,
    },
    Msg {
        subject: String,
        sid: String,
        reply_to: Option<String>,
    },
}

const MAX_PAYLOAD: usize = 1024 * 1024 * 8; // 8 MiB default

#[derive(Clone)]
pub struct NatsCodec {
    state: DecodeState,
    max_payload: usize,
}

impl Default for NatsCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl NatsCodec {
    pub fn new() -> Self {
        NatsCodec {
            state: DecodeState::ReadingControl,
            max_payload: MAX_PAYLOAD,
        }
    }

    pub fn with_max_payload(max_payload: usize) -> Self {
        NatsCodec {
            state: DecodeState::ReadingControl,
            max_payload,
        }
    }
}

impl codec::Decoder for NatsCodec {
    type Item = NatsPacket;
    type Error = NatsCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match &self.state {
                DecodeState::ReadingControl => {
                    // Find \r\n
                    let pos = match find_crlf(src) {
                        Some(p) => p,
                        None => return Ok(None),
                    };

                    let line_bytes = src.split_to(pos);
                    src.advance(2); // consume \r\n

                    let line = std::str::from_utf8(&line_bytes)
                        .map_err(NatsCodecError::InvalidUtf8)?
                        .trim();
                    if line.is_empty() {
                        continue;
                    }

                    match parse_control_line(line)? {
                        // Packets with payload → transition state
                        ParsedControl::Pub {
                            subject,
                            reply_to,
                            payload_len,
                        } => {
                            if payload_len > self.max_payload {
                                return Err(NatsCodecError::PayloadTooLarge(payload_len));
                            }
                            if payload_len == 0 {
                                return Ok(Some(NatsPacket::Pub {
                                    subject,
                                    reply_to,
                                    payload: Bytes::new(),
                                }));
                            }
                            self.state = DecodeState::ReadingPayload {
                                packet: PartialPacket::Pub { subject, reply_to },
                                payload_len,
                            };
                        }
                        ParsedControl::Msg {
                            subject,
                            sid,
                            reply_to,
                            payload_len,
                        } => {
                            if payload_len > self.max_payload {
                                return Err(NatsCodecError::PayloadTooLarge(payload_len));
                            }
                            if payload_len == 0 {
                                return Ok(Some(NatsPacket::Msg {
                                    subject,
                                    sid,
                                    reply_to,
                                    payload: Bytes::new(),
                                }));
                            }
                            self.state = DecodeState::ReadingPayload {
                                packet: PartialPacket::Msg {
                                    subject,
                                    sid,
                                    reply_to,
                                },
                                payload_len,
                            };
                        }
                        // All other packets are complete after the control line
                        ParsedControl::Complete(packet) => return Ok(Some(*packet)),
                    }
                }

                DecodeState::ReadingPayload {
                    payload_len,
                    packet: _,
                } => {
                    let needed = *payload_len + 2; // payload + \r\n
                    if src.len() < needed {
                        src.reserve(needed - src.len());
                        return Ok(None);
                    }

                    // Swap state out so we can consume `packet` by value
                    let prev = std::mem::replace(&mut self.state, DecodeState::ReadingControl);
                    let (partial, len) = match prev {
                        DecodeState::ReadingPayload {
                            packet,
                            payload_len,
                        } => (packet, payload_len),
                        _ => unreachable!(),
                    };

                    let payload = src.split_to(len).freeze();
                    src.advance(2); // consume trailing \r\n

                    let nats_packet = match partial {
                        PartialPacket::Pub { subject, reply_to } => NatsPacket::Pub {
                            subject,
                            reply_to,
                            payload,
                        },
                        PartialPacket::Msg {
                            subject,
                            sid,
                            reply_to,
                        } => NatsPacket::Msg {
                            subject,
                            sid,
                            reply_to,
                            payload,
                        },
                    };

                    return Ok(Some(nats_packet));
                }
            }
        }
    }
}

impl codec::Encoder<NatsPacket> for NatsCodec {
    type Error = NatsCodecError;

    fn encode(&mut self, packet: NatsPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match packet {
            NatsPacket::Info(info) => {
                let json = serde_json::to_string(&info).map_err(NatsCodecError::InvalidJson)?;
                dst.reserve(6 + json.len() + 2);
                dst.put_slice(b"INFO ");
                dst.put_slice(json.as_bytes());
                dst.put_slice(b"\r\n");
            }

            NatsPacket::Msg {
                subject,
                sid,
                reply_to,
                payload,
            } => {
                let len = payload.len();
                let ctrl = match reply_to {
                    Some(rt) => format!("MSG {} {} {} {}\r\n", subject, sid, rt, len),
                    None => format!("MSG {} {} {}\r\n", subject, sid, len),
                };
                dst.reserve(ctrl.len() + len + 2);
                dst.put_slice(ctrl.as_bytes());
                dst.put_slice(&payload);
                dst.put_slice(b"\r\n");
            }

            NatsPacket::Ping => dst.put_slice(b"PING\r\n"),
            NatsPacket::Pong => dst.put_slice(b"PONG\r\n"),
            NatsPacket::Ok => dst.put_slice(b"+OK\r\n"),

            NatsPacket::Err(msg) => {
                let line = format!("-ERR '{}'\r\n", msg);
                dst.put_slice(line.as_bytes());
            }

            NatsPacket::Connect(connect) => {
                let json = serde_json::to_string(&connect).map_err(NatsCodecError::InvalidJson)?;
                dst.reserve(8 + json.len() + 2);
                dst.put_slice(b"CONNECT ");
                dst.put_slice(json.as_bytes());
                dst.put_slice(b"\r\n");
            }

            NatsPacket::Pub {
                subject,
                reply_to,
                payload,
            } => {
                let len = payload.len();
                let ctrl = match reply_to {
                    Some(rt) => format!("PUB {} {} {}\r\n", subject, rt, len),
                    None => format!("PUB {} {}\r\n", subject, len),
                };
                dst.reserve(ctrl.len() + len + 2);
                dst.put_slice(ctrl.as_bytes());
                dst.put_slice(&payload);
                dst.put_slice(b"\r\n");
            }

            NatsPacket::Sub {
                subject,
                queue_group,
                sid,
            } => {
                let line = match queue_group {
                    Some(qg) => format!("SUB {} {} {}\r\n", subject, qg, sid),
                    None => format!("SUB {} {}\r\n", subject, sid),
                };
                dst.put_slice(line.as_bytes());
            }

            NatsPacket::Unsub { sid, max_msgs } => {
                let line = match max_msgs {
                    Some(n) => format!("UNSUB {} {}\r\n", sid, n),
                    None => format!("UNSUB {}\r\n", sid),
                };
                dst.put_slice(line.as_bytes());
            }
        }
        Ok(())
    }
}

enum ParsedControl {
    Complete(Box<NatsPacket>),
    Pub {
        subject: String,
        reply_to: Option<String>,
        payload_len: usize,
    },
    Msg {
        subject: String,
        sid: String,
        reply_to: Option<String>,
        payload_len: usize,
    },
}

fn parse_control_line(line: &str) -> Result<ParsedControl, NatsCodecError> {
    // Split on first whitespace to get the verb
    let (verb, rest) = split_verb(line);

    match verb.to_uppercase().as_str() {
        "INFO" => {
            let info: ServerInfo =
                serde_json::from_str(rest.trim()).map_err(NatsCodecError::InvalidJson)?;
            Ok(ParsedControl::Complete(Box::new(NatsPacket::Info(info))))
        }

        "CONNECT" => {
            let connect: ClientConnect =
                serde_json::from_str(rest.trim()).map_err(NatsCodecError::InvalidJson)?;
            Ok(ParsedControl::Complete(Box::new(NatsPacket::Connect(
                connect,
            ))))
        }

        "PUB" => {
            // PUB <subject> [reply-to] <#bytes>
            let parts: Vec<&str> = rest.split_whitespace().collect();
            match parts.len() {
                2 => {
                    let subject = parts[0].to_string();
                    let payload_len = parse_usize(parts[1], "PUB")?;
                    Ok(ParsedControl::Pub {
                        subject,
                        reply_to: None,
                        payload_len,
                    })
                }
                3 => {
                    let subject = parts[0].to_string();
                    let reply_to = parts[1].to_string();
                    let payload_len = parse_usize(parts[2], "PUB")?;
                    Ok(ParsedControl::Pub {
                        subject,
                        reply_to: Some(reply_to),
                        payload_len,
                    })
                }
                _ => Err(NatsCodecError::MalformedCommand(format!(
                    "PUB expects 2 or 3 args, got: {}",
                    line
                ))),
            }
        }

        "SUB" => {
            // SUB <subject> [queue group] <sid>
            let parts: Vec<&str> = rest.split_whitespace().collect();
            match parts.len() {
                2 => Ok(ParsedControl::Complete(Box::new(NatsPacket::Sub {
                    subject: parts[0].to_string(),
                    queue_group: None,
                    sid: parts[1].to_string(),
                }))),
                3 => Ok(ParsedControl::Complete(Box::new(NatsPacket::Sub {
                    subject: parts[0].to_string(),
                    queue_group: Some(parts[1].to_string()),
                    sid: parts[2].to_string(),
                }))),
                _ => Err(NatsCodecError::MalformedCommand(format!(
                    "SUB expects 2 or 3 args, got: {}",
                    line
                ))),
            }
        }

        "UNSUB" => {
            // UNSUB <sid> [max_msgs]
            let parts: Vec<&str> = rest.split_whitespace().collect();
            match parts.len() {
                1 => Ok(ParsedControl::Complete(Box::new(NatsPacket::Unsub {
                    sid: parts[0].to_string(),
                    max_msgs: None,
                }))),
                2 => {
                    let max_msgs = parse_u32(parts[1], "UNSUB")?;
                    Ok(ParsedControl::Complete(Box::new(NatsPacket::Unsub {
                        sid: parts[0].to_string(),
                        max_msgs: Some(max_msgs),
                    })))
                }
                _ => Err(NatsCodecError::MalformedCommand(format!(
                    "UNSUB expects 1 or 2 args, got: {}",
                    line
                ))),
            }
        }

        "MSG" => {
            // MSG <subject> <sid> [reply-to] <#bytes>
            let parts: Vec<&str> = rest.split_whitespace().collect();
            match parts.len() {
                3 => {
                    let subject = parts[0].to_string();
                    let sid = parts[1].to_string();
                    let payload_len = parse_usize(parts[2], "MSG")?;
                    Ok(ParsedControl::Msg {
                        subject,
                        sid,
                        reply_to: None,
                        payload_len,
                    })
                }
                4 => {
                    let subject = parts[0].to_string();
                    let sid = parts[1].to_string();
                    let reply_to = parts[2].to_string();
                    let payload_len = parse_usize(parts[3], "MSG")?;
                    Ok(ParsedControl::Msg {
                        subject,
                        sid,
                        reply_to: Some(reply_to),
                        payload_len,
                    })
                }
                _ => Err(NatsCodecError::MalformedCommand(format!(
                    "MSG expects 3 or 4 args, got: {}",
                    line
                ))),
            }
        }

        "PING" => Ok(ParsedControl::Complete(Box::new(NatsPacket::Ping))),
        "PONG" => Ok(ParsedControl::Complete(Box::new(NatsPacket::Pong))),
        "+OK" => Ok(ParsedControl::Complete(Box::new(NatsPacket::Ok))),

        "-ERR" => {
            // -ERR 'message'  — strip surrounding quotes if present
            let msg = rest.trim().trim_matches('\'').to_string();
            Ok(ParsedControl::Complete(Box::new(NatsPacket::Err(msg))))
        }

        other => Err(NatsCodecError::UnknownCommand(other.to_string())),
    }
}

fn find_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\r\n")
}

fn split_verb(line: &str) -> (&str, &str) {
    match line.find(|c: char| c.is_whitespace()) {
        Some(pos) => (&line[..pos], &line[pos..]),
        None => (line, ""),
    }
}

fn parse_usize(s: &str, cmd: &str) -> Result<usize, NatsCodecError> {
    s.parse::<usize>().map_err(|_| {
        NatsCodecError::MalformedCommand(format!("{}: expected integer, got '{}'", cmd, s))
    })
}

fn parse_u32(s: &str, cmd: &str) -> Result<u32, NatsCodecError> {
    s.parse::<u32>().map_err(|_| {
        NatsCodecError::MalformedCommand(format!("{}: expected integer, got '{}'", cmd, s))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::codec::{Decoder, Encoder};

    fn encode(packet: NatsPacket) -> BytesMut {
        let mut codec = NatsCodec::new();
        let mut buf = BytesMut::new();
        codec.encode(packet, &mut buf).unwrap();
        buf
    }

    fn decode(buf: &mut BytesMut) -> NatsPacket {
        let mut codec = NatsCodec::new();
        codec.decode(buf).unwrap().unwrap()
    }

    #[test]
    fn ping_roundtrip() {
        let mut buf = encode(NatsPacket::Ping);
        assert_eq!(decode(&mut buf), NatsPacket::Ping);
    }

    #[test]
    fn pong_roundtrip() {
        let mut buf = encode(NatsPacket::Pong);
        assert_eq!(decode(&mut buf), NatsPacket::Pong);
    }

    #[test]
    fn ok_roundtrip() {
        let mut buf = encode(NatsPacket::Ok);
        assert_eq!(decode(&mut buf), NatsPacket::Ok);
    }

    #[test]
    fn err_roundtrip() {
        let mut buf = encode(NatsPacket::Err("Unknown Protocol Operation".to_string()));
        assert_eq!(
            decode(&mut buf),
            NatsPacket::Err("Unknown Protocol Operation".to_string())
        );
    }

    #[test]
    fn pub_no_reply_roundtrip() {
        let packet = NatsPacket::Pub {
            subject: "foo.bar".to_string(),
            reply_to: None,
            payload: Bytes::from("hello nats"),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn pub_with_reply_roundtrip() {
        let packet = NatsPacket::Pub {
            subject: "foo".to_string(),
            reply_to: Some("INBOX.42".to_string()),
            payload: Bytes::from("knock knock"),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn pub_empty_payload_roundtrip() {
        let packet = NatsPacket::Pub {
            subject: "notify".to_string(),
            reply_to: None,
            payload: Bytes::new(),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn sub_no_queue_roundtrip() {
        let packet = NatsPacket::Sub {
            subject: "foo".to_string(),
            queue_group: None,
            sid: "1".to_string(),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn sub_with_queue_roundtrip() {
        let packet = NatsPacket::Sub {
            subject: "bar".to_string(),
            queue_group: Some("workers".to_string()),
            sid: "42".to_string(),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn unsub_no_max_roundtrip() {
        let packet = NatsPacket::Unsub {
            sid: "1".to_string(),
            max_msgs: None,
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn unsub_with_max_roundtrip() {
        let packet = NatsPacket::Unsub {
            sid: "1".to_string(),
            max_msgs: Some(5),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn msg_no_reply_roundtrip() {
        let packet = NatsPacket::Msg {
            subject: "foo.bar".to_string(),
            sid: "9".to_string(),
            reply_to: None,
            payload: Bytes::from("Hello World"),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn msg_with_reply_roundtrip() {
        let packet = NatsPacket::Msg {
            subject: "foo.bar".to_string(),
            sid: "9".to_string(),
            reply_to: Some("GREETING.34".to_string()),
            payload: Bytes::from("Hello World"),
        };
        let mut buf = encode(packet.clone());
        assert_eq!(decode(&mut buf), packet);
    }

    #[test]
    fn multi_packet_in_buffer() {
        let mut codec = NatsCodec::new();
        let mut buf = BytesMut::new();

        // Two packets in one buffer
        codec.encode(NatsPacket::Ping, &mut buf).unwrap();
        codec.encode(NatsPacket::Pong, &mut buf).unwrap();

        assert_eq!(codec.decode(&mut buf).unwrap().unwrap(), NatsPacket::Ping);
        assert_eq!(codec.decode(&mut buf).unwrap().unwrap(), NatsPacket::Pong);
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn partial_buffer_returns_none() {
        let mut codec = NatsCodec::new();
        // Only partial control line, no \r\n yet
        let mut buf = BytesMut::from(&b"PING"[..]);
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn partial_payload_returns_none() {
        let mut codec = NatsCodec::new();
        // Control line complete but payload not fully arrived
        let mut buf = BytesMut::from(&b"PUB foo 10\r\nhello"[..]);
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }
}
