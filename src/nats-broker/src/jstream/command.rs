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

/// Subject namespace prefix for JetStream API.
const JS_API_PREFIX: &str = "$JS.API";
/// Subject namespace prefix for JetStream ACK.
const JS_ACK_PREFIX: &str = "$JS.ACK";
/// Subject namespace prefix for JetStream advisory events.
const JS_EVENT_PREFIX: &str = "$JS.EVENT";
/// Subject namespace prefix for KV store.
const KV_PREFIX: &str = "$KV";
/// Subject namespace prefix for Object store.
const OBJ_PREFIX: &str = "$OBJ";

/// All recognized JetStream commands parsed from NATS subjects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsCommand {
    // ── $JS.API.INFO ──────────────────────────────────────────────
    /// `$JS.API.INFO`
    Info,
    /// `$JS.API.ACCOUNT.INFO`
    AccountInfo,

    // ── $JS.API.STREAM.* ─────────────────────────────────────────
    /// `$JS.API.STREAM.CREATE.{stream}`
    StreamCreate { stream: String },
    /// `$JS.API.STREAM.UPDATE.{stream}`
    StreamUpdate { stream: String },
    /// `$JS.API.STREAM.DELETE.{stream}`
    StreamDelete { stream: String },
    /// `$JS.API.STREAM.INFO.{stream}`
    StreamInfo { stream: String },
    /// `$JS.API.STREAM.LIST`
    StreamList,
    /// `$JS.API.STREAM.NAMES`
    StreamNames,
    /// `$JS.API.STREAM.PURGE.{stream}`
    StreamPurge { stream: String },
    /// `$JS.API.STREAM.MSG.GET.{stream}`
    StreamMsgGet { stream: String },
    /// `$JS.API.STREAM.MSG.DELETE.{stream}`
    StreamMsgDelete { stream: String },
    /// `$JS.API.STREAM.SNAPSHOT.{stream}`
    StreamSnapshot { stream: String },
    /// `$JS.API.STREAM.RESTORE.{stream}`
    StreamRestore { stream: String },
    /// `$JS.API.STREAM.LEADER.STEPDOWN.{stream}`
    StreamLeaderStepdown { stream: String },
    /// `$JS.API.STREAM.PEER.REMOVE.{stream}`
    StreamPeerRemove { stream: String },

    // ── $JS.API.CONSUMER.* ───────────────────────────────────────
    /// `$JS.API.CONSUMER.CREATE.{stream}` (ephemeral)
    ConsumerCreate { stream: String },
    /// `$JS.API.CONSUMER.CREATE.{stream}.{consumer}` (named ephemeral)
    ConsumerCreateNamed { stream: String, consumer: String },
    /// `$JS.API.CONSUMER.DURABLE.CREATE.{stream}.{consumer}`
    ConsumerDurableCreate { stream: String, consumer: String },
    /// `$JS.API.CONSUMER.DELETE.{stream}.{consumer}`
    ConsumerDelete { stream: String, consumer: String },
    /// `$JS.API.CONSUMER.INFO.{stream}.{consumer}`
    ConsumerInfo { stream: String, consumer: String },
    /// `$JS.API.CONSUMER.LIST.{stream}`
    ConsumerList { stream: String },
    /// `$JS.API.CONSUMER.NAMES.{stream}`
    ConsumerNames { stream: String },
    /// `$JS.API.CONSUMER.MSG.NEXT.{stream}.{consumer}`
    ConsumerMsgNext { stream: String, consumer: String },
    /// `$JS.API.CONSUMER.LEADER.STEPDOWN.{stream}.{consumer}`
    ConsumerLeaderStepdown { stream: String, consumer: String },
    /// `$JS.API.CONSUMER.PAUSE.{stream}.{consumer}`
    ConsumerPause { stream: String, consumer: String },

    // ── $JS.API.DIRECT.GET.* ─────────────────────────────────────
    /// `$JS.API.DIRECT.GET.{stream}`
    DirectGet { stream: String },
    /// `$JS.API.DIRECT.GET.{stream}.{subject}`
    DirectGetBySubject { stream: String, subject: String },
    /// `$JS.API.DIRECT.GET.LAST.{stream}.{subject}`
    DirectGetLast { stream: String, subject: String },

    // ── $JS.ACK.* ────────────────────────────────────────────────
    /// `$JS.ACK.{stream}.{consumer}.{...}` — positive ack
    Ack { stream: String, consumer: String },
    /// Negative ack (re-deliver), optionally with delay
    Nak { stream: String, consumer: String },
    /// In-progress heartbeat, resets ack wait timer
    AckProgress { stream: String, consumer: String },
    /// Terminate message, no re-delivery
    AckTerm { stream: String, consumer: String },
    /// Ack + request next message
    AckNext { stream: String, consumer: String },

    // ── $JS.EVENT.* ──────────────────────────────────────────────
    /// `$JS.EVENT.ADVISORY.STREAM.CREATED.{stream}`
    EventStreamCreated { stream: String },
    /// `$JS.EVENT.ADVISORY.STREAM.DELETED.{stream}`
    EventStreamDeleted { stream: String },
    /// `$JS.EVENT.ADVISORY.STREAM.UPDATED.{stream}`
    EventStreamUpdated { stream: String },
    /// `$JS.EVENT.ADVISORY.CONSUMER.CREATED.{stream}.{consumer}`
    EventConsumerCreated { stream: String, consumer: String },
    /// `$JS.EVENT.ADVISORY.CONSUMER.DELETED.{stream}.{consumer}`
    EventConsumerDeleted { stream: String, consumer: String },
    /// `$JS.EVENT.ADVISORY.API.{subject}` — API audit event
    EventApiAudit { subject: String },
    /// `$JS.EVENT.ADVISORY.STREAM.SNAPSHOT.CREATE.{stream}`
    EventSnapshotCreate { stream: String },
    /// `$JS.EVENT.ADVISORY.STREAM.SNAPSHOT.COMPLETE.{stream}`
    EventSnapshotComplete { stream: String },
    /// `$JS.EVENT.ADVISORY.STREAM.RESTORE.CREATE.{stream}`
    EventRestoreCreate { stream: String },
    /// `$JS.EVENT.ADVISORY.STREAM.RESTORE.COMPLETE.{stream}`
    EventRestoreComplete { stream: String },
    /// `$JS.EVENT.ADVISORY.CONSUMER.LEADER_ELECTED.{stream}.{consumer}`
    EventConsumerLeaderElected { stream: String, consumer: String },
    /// `$JS.EVENT.ADVISORY.STREAM.LEADER_ELECTED.{stream}`
    EventStreamLeaderElected { stream: String },

    // ── $KV.* ────────────────────────────────────────────────────
    /// `$KV.{bucket}.{key}` — put (publish)
    KvPut { bucket: String, key: String },
    /// `$KV.{bucket}.{key}` — get (request)
    KvGet { bucket: String, key: String },
    /// `$KV.{bucket}.{key}` — delete
    KvDelete { bucket: String, key: String },
    /// `$KV.{bucket}.{key}` — purge all revisions
    KvPurge { bucket: String, key: String },
    /// `$KV.{bucket}.>` — list all keys
    KvKeys { bucket: String },
    /// `$KV.{bucket}.{key|>}` — watch for changes
    KvWatch { bucket: String, pattern: String },

    // ── $OBJ.* ───────────────────────────────────────────────────
    /// `$OBJ.{bucket}` — put (chunked upload)
    ObjPut { bucket: String },
    /// `$OBJ.{bucket}.{object}` — get object
    ObjGet { bucket: String, object: String },
    /// `$OBJ.{bucket}.{object}` — delete object
    ObjDelete { bucket: String, object: String },
    /// `$OBJ.{bucket}.{object}` — get object info/metadata
    ObjInfo { bucket: String, object: String },
    /// `$OBJ.{bucket}` — list all objects
    ObjList { bucket: String },
    /// `$OBJ.{bucket}.>` — watch for changes
    ObjWatch { bucket: String },
}

impl JsCommand {
    pub fn is_js_subject(subject: &str) -> bool {
        subject.starts_with(JS_API_PREFIX)
            || subject.starts_with(JS_ACK_PREFIX)
            || subject.starts_with(JS_EVENT_PREFIX)
            || subject.starts_with(KV_PREFIX)
            || subject.starts_with(OBJ_PREFIX)
    }

    pub fn parse(subject: &str) -> Option<Self> {
        if let Some(rest) = subject.strip_prefix(JS_API_PREFIX) {
            return parse_js_api(rest.strip_prefix('.')?);
        }
        if let Some(rest) = subject.strip_prefix(JS_ACK_PREFIX) {
            return parse_js_ack(rest.strip_prefix('.')?);
        }
        if let Some(rest) = subject.strip_prefix(JS_EVENT_PREFIX) {
            return parse_js_event(rest.strip_prefix('.')?);
        }
        if let Some(rest) = subject.strip_prefix(KV_PREFIX) {
            return parse_kv(rest.strip_prefix('.')?);
        }
        if let Some(rest) = subject.strip_prefix(OBJ_PREFIX) {
            return parse_obj(rest.strip_prefix('.')?);
        }
        None
    }
}

fn parse_js_api(rest: &str) -> Option<JsCommand> {
    let (cmd, tail) = split_first(rest);
    match cmd {
        "INFO" if tail.is_empty() => Some(JsCommand::Info),
        "ACCOUNT" => {
            let (sub, _) = split_first(tail);
            if sub == "INFO" {
                Some(JsCommand::AccountInfo)
            } else {
                None
            }
        }
        "STREAM" => parse_stream(tail),
        "CONSUMER" => parse_consumer(tail),
        "DIRECT" => parse_direct(tail),
        _ => None,
    }
}

fn parse_stream(tail: &str) -> Option<JsCommand> {
    let (op, rest) = split_first(tail);
    match op {
        "CREATE" => Some(JsCommand::StreamCreate {
            stream: rest.to_string(),
        }),
        "UPDATE" => Some(JsCommand::StreamUpdate {
            stream: rest.to_string(),
        }),
        "DELETE" => Some(JsCommand::StreamDelete {
            stream: rest.to_string(),
        }),
        "INFO" => Some(JsCommand::StreamInfo {
            stream: rest.to_string(),
        }),
        "LIST" => Some(JsCommand::StreamList),
        "NAMES" => Some(JsCommand::StreamNames),
        "PURGE" => Some(JsCommand::StreamPurge {
            stream: rest.to_string(),
        }),
        "MSG" => {
            let (sub, stream) = split_first(rest);
            match sub {
                "GET" => Some(JsCommand::StreamMsgGet {
                    stream: stream.to_string(),
                }),
                "DELETE" => Some(JsCommand::StreamMsgDelete {
                    stream: stream.to_string(),
                }),
                _ => None,
            }
        }
        "SNAPSHOT" => Some(JsCommand::StreamSnapshot {
            stream: rest.to_string(),
        }),
        "RESTORE" => Some(JsCommand::StreamRestore {
            stream: rest.to_string(),
        }),
        "LEADER" => {
            let (sub, stream) = split_first(rest);
            if sub == "STEPDOWN" {
                Some(JsCommand::StreamLeaderStepdown {
                    stream: stream.to_string(),
                })
            } else {
                None
            }
        }
        "PEER" => {
            let (sub, stream) = split_first(rest);
            if sub == "REMOVE" {
                Some(JsCommand::StreamPeerRemove {
                    stream: stream.to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_consumer(tail: &str) -> Option<JsCommand> {
    let (op, rest) = split_first(tail);
    match op {
        "CREATE" => {
            let (stream, consumer) = split_first(rest);
            if consumer.is_empty() {
                Some(JsCommand::ConsumerCreate {
                    stream: stream.to_string(),
                })
            } else {
                Some(JsCommand::ConsumerCreateNamed {
                    stream: stream.to_string(),
                    consumer: consumer.to_string(),
                })
            }
        }
        "DURABLE" => {
            let (sub, rest2) = split_first(rest);
            if sub == "CREATE" {
                let (stream, consumer) = split_first(rest2);
                Some(JsCommand::ConsumerDurableCreate {
                    stream: stream.to_string(),
                    consumer: consumer.to_string(),
                })
            } else {
                None
            }
        }
        "DELETE" => {
            let (stream, consumer) = split_first(rest);
            Some(JsCommand::ConsumerDelete {
                stream: stream.to_string(),
                consumer: consumer.to_string(),
            })
        }
        "INFO" => {
            let (stream, consumer) = split_first(rest);
            Some(JsCommand::ConsumerInfo {
                stream: stream.to_string(),
                consumer: consumer.to_string(),
            })
        }
        "LIST" => Some(JsCommand::ConsumerList {
            stream: rest.to_string(),
        }),
        "NAMES" => Some(JsCommand::ConsumerNames {
            stream: rest.to_string(),
        }),
        "MSG" => {
            let (sub, rest2) = split_first(rest);
            if sub == "NEXT" {
                let (stream, consumer) = split_first(rest2);
                Some(JsCommand::ConsumerMsgNext {
                    stream: stream.to_string(),
                    consumer: consumer.to_string(),
                })
            } else {
                None
            }
        }
        "LEADER" => {
            let (sub, rest2) = split_first(rest);
            if sub == "STEPDOWN" {
                let (stream, consumer) = split_first(rest2);
                Some(JsCommand::ConsumerLeaderStepdown {
                    stream: stream.to_string(),
                    consumer: consumer.to_string(),
                })
            } else {
                None
            }
        }
        "PAUSE" => {
            let (stream, consumer) = split_first(rest);
            Some(JsCommand::ConsumerPause {
                stream: stream.to_string(),
                consumer: consumer.to_string(),
            })
        }
        _ => None,
    }
}

fn parse_direct(tail: &str) -> Option<JsCommand> {
    let (op, rest) = split_first(tail);
    if op != "GET" {
        return None;
    }
    let (first, rest2) = split_first(rest);
    if first == "LAST" {
        let (stream, subject) = split_first(rest2);
        return Some(JsCommand::DirectGetLast {
            stream: stream.to_string(),
            subject: subject.to_string(),
        });
    }
    // first is stream name; rest2 is optional subject
    if rest2.is_empty() {
        Some(JsCommand::DirectGet {
            stream: first.to_string(),
        })
    } else {
        Some(JsCommand::DirectGetBySubject {
            stream: first.to_string(),
            subject: rest2.to_string(),
        })
    }
}

/// Parse `$JS.ACK.{stream}.{consumer}.{ack_type}.*`
fn parse_js_ack(rest: &str) -> Option<JsCommand> {
    // Format: {stream}.{consumer}.{delivery_count}.{stream_seq}.{consumer_seq}.{timestamp}.{num_pending}[.{ack_type}]
    // For routing purposes we only need stream + consumer + ack kind from the reply subject.
    // The ack subject encodes the ack type as the last meaningful token by convention from client libraries.
    // We extract stream and consumer from the first two dot-separated tokens.
    let mut parts = rest.splitn(3, '.');
    let stream = parts.next()?.to_string();
    let consumer = parts.next()?.to_string();
    let tail = parts.next().unwrap_or("");

    // Ack type is signaled by the client via the payload, not the subject in most cases.
    // We default to Ack here; the handler will inspect the payload for nak/term/next/progress.
    let _ = tail;
    Some(JsCommand::Ack { stream, consumer })
}

fn parse_js_event(rest: &str) -> Option<JsCommand> {
    // Format: ADVISORY.{category}.{event}.{stream}[.{consumer}]
    let (prefix, tail) = split_first(rest);
    if prefix != "ADVISORY" {
        return None;
    }
    let (category, tail) = split_first(tail);
    match category {
        "STREAM" => {
            let (event, stream_tail) = split_first(tail);
            match event {
                "CREATED" => Some(JsCommand::EventStreamCreated {
                    stream: stream_tail.to_string(),
                }),
                "DELETED" => Some(JsCommand::EventStreamDeleted {
                    stream: stream_tail.to_string(),
                }),
                "UPDATED" => Some(JsCommand::EventStreamUpdated {
                    stream: stream_tail.to_string(),
                }),
                "SNAPSHOT" => {
                    let (sub, stream) = split_first(stream_tail);
                    match sub {
                        "CREATE" => Some(JsCommand::EventSnapshotCreate {
                            stream: stream.to_string(),
                        }),
                        "COMPLETE" => Some(JsCommand::EventSnapshotComplete {
                            stream: stream.to_string(),
                        }),
                        _ => None,
                    }
                }
                "RESTORE" => {
                    let (sub, stream) = split_first(stream_tail);
                    match sub {
                        "CREATE" => Some(JsCommand::EventRestoreCreate {
                            stream: stream.to_string(),
                        }),
                        "COMPLETE" => Some(JsCommand::EventRestoreComplete {
                            stream: stream.to_string(),
                        }),
                        _ => None,
                    }
                }
                "LEADER_ELECTED" => Some(JsCommand::EventStreamLeaderElected {
                    stream: stream_tail.to_string(),
                }),
                _ => None,
            }
        }
        "CONSUMER" => {
            let (event, rest2) = split_first(tail);
            match event {
                "CREATED" => {
                    let (stream, consumer) = split_first(rest2);
                    Some(JsCommand::EventConsumerCreated {
                        stream: stream.to_string(),
                        consumer: consumer.to_string(),
                    })
                }
                "DELETED" => {
                    let (stream, consumer) = split_first(rest2);
                    Some(JsCommand::EventConsumerDeleted {
                        stream: stream.to_string(),
                        consumer: consumer.to_string(),
                    })
                }
                "LEADER_ELECTED" => {
                    let (stream, consumer) = split_first(rest2);
                    Some(JsCommand::EventConsumerLeaderElected {
                        stream: stream.to_string(),
                        consumer: consumer.to_string(),
                    })
                }
                _ => None,
            }
        }
        "API" => Some(JsCommand::EventApiAudit {
            subject: tail.to_string(),
        }),
        _ => None,
    }
}

fn parse_kv(rest: &str) -> Option<JsCommand> {
    let (bucket, key) = split_first(rest);
    if bucket.is_empty() {
        return None;
    }
    if key == ">" {
        return Some(JsCommand::KvKeys {
            bucket: bucket.to_string(),
        });
    }
    // Caller distinguishes put/get/delete/purge/watch by operation context (PUB vs SUB vs request).
    // We return KvGet as the default parsed form; the handler routes based on operation type.
    Some(JsCommand::KvGet {
        bucket: bucket.to_string(),
        key: key.to_string(),
    })
}

fn parse_obj(rest: &str) -> Option<JsCommand> {
    let (bucket, tail) = split_first(rest);
    if bucket.is_empty() {
        return None;
    }
    if tail.is_empty() {
        return Some(JsCommand::ObjList {
            bucket: bucket.to_string(),
        });
    }
    if tail == ">" {
        return Some(JsCommand::ObjWatch {
            bucket: bucket.to_string(),
        });
    }
    Some(JsCommand::ObjGet {
        bucket: bucket.to_string(),
        object: tail.to_string(),
    })
}

/// Split `s` at the first `.`, returning `(before, after)`.
/// If there is no `.`, returns `(s, "")`.
fn split_first(s: &str) -> (&str, &str) {
    s.split_once('.').unwrap_or((s, ""))
}
