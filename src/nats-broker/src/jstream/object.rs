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
use crate::jstream::protocol::{
    ObjectDeleteResponse, ObjectInfo, ObjectListResponse, ObjectMeta, ObjectRequest,
};
use bytes::Bytes;

/// `$OBJ.<bucket>.info.<object>` + `$OBJ.<bucket>.chunks.<nonce>` — chunked upload.
/// Caller publishes metadata first, then data chunks to the chunks subject.
/// This handler processes the metadata message that initiates the upload.
pub async fn process_obj_put(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _meta: ObjectMeta,
) -> Result<ObjectInfo, NatsBrokerError> {
    todo!("OBJ PUT")
}

/// `$OBJ.<bucket>` with `{"name": "...", "deliver_subject": "..."}` — stream object chunks.
/// Server pushes chunks to the deliver_subject; caller must SUB first.
pub async fn process_obj_get(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _req: ObjectRequest,
) -> Result<(), NatsBrokerError> {
    todo!("OBJ GET")
}

/// `$OBJ.<bucket>` with `{"name": "..."}` — delete object.
pub async fn process_obj_delete(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _req: ObjectRequest,
) -> Result<ObjectDeleteResponse, NatsBrokerError> {
    todo!("OBJ DELETE")
}

/// `$OBJ.<bucket>` with `{"name": "..."}` — get object metadata.
pub async fn process_obj_info(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _req: ObjectRequest,
) -> Result<ObjectInfo, NatsBrokerError> {
    todo!("OBJ INFO")
}

/// `$OBJ.<bucket>` with empty body — list all objects in bucket.
pub async fn process_obj_list(
    _ctx: &NatsProcessContext,
    _bucket: &str,
) -> Result<ObjectListResponse, NatsBrokerError> {
    todo!("OBJ LIST")
}

/// `$OBJ.<bucket>.>` — watch for object changes (upload / delete).
/// Sets up a push consumer on the info subject prefix.
pub async fn process_obj_watch(
    _ctx: &NatsProcessContext,
    _bucket: &str,
) -> Result<(), NatsBrokerError> {
    todo!("OBJ WATCH")
}

/// Handle a single chunk published to `$OBJ.<bucket>.chunks.<nonce>`.
/// Called for each chunk during an active upload session.
pub async fn process_obj_chunk(
    _ctx: &NatsProcessContext,
    _bucket: &str,
    _nonce: &str,
    _chunk: Bytes,
) -> Result<(), NatsBrokerError> {
    todo!("OBJ CHUNK")
}
