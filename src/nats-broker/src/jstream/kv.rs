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

use crate::core::error::NatsBrokerError;
use crate::handler::command::NatsProcessContext;
use crate::jstream::protocol::{KvGetHeaders, KvPutResponse};
use bytes::Bytes;

/// `$KV.<bucket>.<key>` — put value.
/// Returns a publish ACK confirming the message was written to the stream.
pub async fn process_kv_put(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
    _payload: Bytes,
) -> Result<KvPutResponse, NatsBrokerError> {
    todo!("KV PUT")
}

/// `$KV.<bucket>.<key>` — get latest value.
/// Returns headers describing the entry plus the raw value bytes.
pub async fn process_kv_get(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
) -> Result<(KvGetHeaders, Bytes), NatsBrokerError> {
    todo!("KV GET")
}

/// `$KV.<bucket>.<key>` — delete key (writes a DEL marker).
pub async fn process_kv_delete(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
) -> Result<KvPutResponse, NatsBrokerError> {
    todo!("KV DELETE")
}

/// `$KV.<bucket>.<key>` — purge all revisions of key (writes a PURGE marker).
pub async fn process_kv_purge(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _key: &str,
) -> Result<KvPutResponse, NatsBrokerError> {
    todo!("KV PURGE")
}

/// `$KV.<bucket>.>` — list all keys in bucket.
/// Returns the key names as strings.
pub async fn process_kv_keys(
    _ctx: &NatsProcessContext,
    _bucket: &str,
) -> Result<Vec<String>, NatsBrokerError> {
    todo!("KV KEYS")
}

/// `$KV.<bucket>.<key|>` — watch for changes.
/// Sets up a push consumer; individual change events are delivered as KV get responses.
pub async fn process_kv_watch(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _pattern: &str,
) -> Result<(), NatsBrokerError> {
    todo!("KV WATCH")
}
