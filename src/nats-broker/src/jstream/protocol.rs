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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// Common types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
    pub code: u16,
    pub err_code: u32,
    pub description: String,
}

/// Shared pagination cursor used by LIST / NAMES requests.
#[derive(Debug, Default, Deserialize)]
pub struct PageRequest {
    pub offset: Option<u64>,
}

/// Shared pagination info returned by LIST / NAMES responses.
#[derive(Debug, Serialize)]
pub struct PageInfo {
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Info
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct JsApiLimits {
    pub max_memory: i64,
    pub max_storage: i64,
    pub max_streams: i64,
    pub max_consumers: i64,
    pub max_ack_pending: i64,
    pub memory_max_stream_bytes: i64,
    pub storage_max_stream_bytes: i64,
    pub duplicate_window_max: i64,
    pub max_bytes_required: bool,
}

#[derive(Debug, Serialize)]
pub struct JsAccountStats {
    pub memory: u64,
    pub storage: u64,
    pub reserved_memory: u64,
    pub reserved_storage: u64,
    pub streams: u64,
    pub consumers: u64,
    pub limits: JsApiLimits,
    pub tiers: Option<HashMap<String, serde_json::Value>>,
}

/// Response for `$JS.API.INFO` and `$JS.API.ACCOUNT.INFO`.
#[derive(Debug, Serialize)]
pub struct JsInfoResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub stats: JsAccountStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stream
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub name: String,
    #[serde(default)]
    pub subjects: Vec<String>,
    #[serde(default = "default_storage")]
    pub storage: String,
    #[serde(default = "default_retention")]
    pub retention: String,
    #[serde(default = "neg_one_i64")]
    pub max_msgs: i64,
    #[serde(default = "neg_one_i64")]
    pub max_bytes: i64,
    #[serde(default)]
    pub max_age: u64,
    #[serde(default = "default_replicas")]
    pub num_replicas: u32,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub deny_delete: bool,
    #[serde(default)]
    pub deny_purge: bool,
    #[serde(default)]
    pub allow_rollup_hdrs: bool,
    #[serde(default = "neg_one_i64")]
    pub max_msgs_per_subject: i64,
    #[serde(default = "neg_one_i64")]
    pub max_msg_size: i64,
    #[serde(default)]
    pub discard: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_window: Option<u64>,
}

fn default_storage() -> String {
    "file".into()
}
fn default_retention() -> String {
    "limits".into()
}
fn default_replicas() -> u32 {
    1
}
fn neg_one_i64() -> i64 {
    -1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamState {
    pub messages: u64,
    pub bytes: u64,
    pub first_seq: u64,
    pub last_seq: u64,
    pub consumer_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamInfo {
    pub config: StreamConfig,
    pub state: StreamState,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<ClusterInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub name: Option<String>,
    pub leader: Option<String>,
    pub replicas: Option<Vec<PeerInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub name: String,
    pub current: bool,
    pub active: u64,
    pub offline: Option<bool>,
    pub lag: Option<u64>,
}

// ── Stream requests ───────────────────────────────────────────────────────────

/// `$JS.API.STREAM.CREATE.<stream>` / `$JS.API.STREAM.UPDATE.<stream>`
pub type StreamCreateRequest = StreamConfig;

/// `$JS.API.STREAM.LIST` / `$JS.API.STREAM.NAMES`
pub type StreamListRequest = PageRequest;

/// `$JS.API.STREAM.PURGE.<stream>`
#[derive(Debug, Default, Deserialize)]
pub struct StreamPurgeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep: Option<u64>,
}

/// `$JS.API.STREAM.MSG.GET.<stream>`
#[derive(Debug, Deserialize)]
pub struct StreamMsgGetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_by_subj: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_by_subj: Option<String>,
}

/// `$JS.API.STREAM.MSG.DELETE.<stream>`
#[derive(Debug, Deserialize)]
pub struct StreamMsgDeleteRequest {
    pub seq: u64,
    #[serde(default)]
    pub no_erase: bool,
}

/// `$JS.API.STREAM.SNAPSHOT.<stream>`
#[derive(Debug, Deserialize)]
pub struct StreamSnapshotRequest {
    pub deliver_subject: String,
    #[serde(default)]
    pub no_consumers: bool,
    #[serde(default)]
    pub check_msgs: bool,
}

/// `$JS.API.STREAM.PEER.REMOVE.<stream>`
#[derive(Debug, Deserialize)]
pub struct StreamPeerRemoveRequest {
    pub peer: String,
}

// ── Stream responses ──────────────────────────────────────────────────────────

/// `$JS.API.STREAM.CREATE` / `UPDATE` / `INFO`
#[derive(Debug, Serialize)]
pub struct StreamInfoResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub info: StreamInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.LIST`
#[derive(Debug, Serialize)]
pub struct StreamListResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub page: PageInfo,
    pub streams: Vec<StreamInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.NAMES`
#[derive(Debug, Serialize)]
pub struct StreamNamesResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub page: PageInfo,
    pub streams: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.DELETE`
#[derive(Debug, Serialize)]
pub struct StreamDeleteResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.PURGE`
#[derive(Debug, Serialize)]
pub struct StreamPurgeResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub success: bool,
    pub purged: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.MSG.GET`
#[derive(Debug, Serialize)]
pub struct RawMessage {
    pub subject: String,
    pub seq: u64,
    pub data: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct StreamMsgGetResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub message: RawMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.MSG.DELETE`
#[derive(Debug, Serialize)]
pub struct StreamMsgDeleteResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.STREAM.LEADER.STEPDOWN` / `PEER.REMOVE`
#[derive(Debug, Serialize)]
pub struct StreamLeaderResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Consumer
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub durable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deliver_subject: Option<String>,
    #[serde(default = "default_deliver_policy")]
    pub deliver_policy: String,
    #[serde(default = "default_ack_policy")]
    pub ack_policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_wait: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_deliver: Option<i64>,
    #[serde(default = "default_replay_policy")]
    pub replay_policy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_subject: Option<String>,
    #[serde(default)]
    pub filter_subjects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_start_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_waiting: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ack_pending: Option<i64>,
    #[serde(default)]
    pub flow_control: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_heartbeat: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_until: Option<String>,
}

fn default_deliver_policy() -> String {
    "all".into()
}
fn default_ack_policy() -> String {
    "explicit".into()
}
fn default_replay_policy() -> String {
    "instant".into()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsumerInfo {
    pub stream_name: String,
    pub name: String,
    pub config: ConsumerConfig,
    pub created: String,
    pub delivered: SequenceInfo,
    pub ack_floor: SequenceInfo,
    pub num_ack_pending: u64,
    pub num_redelivered: u64,
    pub num_waiting: u64,
    pub num_pending: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<ClusterInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_remaining: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceInfo {
    pub consumer_seq: u64,
    pub stream_seq: u64,
}

// ── Consumer requests ─────────────────────────────────────────────────────────

/// `$JS.API.CONSUMER.CREATE.*` / `CONSUMER.DURABLE.CREATE.*`
#[derive(Debug, Deserialize)]
pub struct ConsumerCreateRequest {
    pub stream_name: String,
    pub config: ConsumerConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// `$JS.API.CONSUMER.LIST.<stream>` / `CONSUMER.NAMES.<stream>`
pub type ConsumerListRequest = PageRequest;

/// `$JS.API.CONSUMER.MSG.NEXT.<stream>.<consumer>`
#[derive(Debug, Default, Deserialize)]
pub struct ConsumerMsgNextRequest {
    #[serde(default = "default_batch")]
    pub batch: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,
    #[serde(default)]
    pub no_wait: bool,
    #[serde(default)]
    pub idle_heartbeat: Option<u64>,
}

fn default_batch() -> u64 {
    1
}

/// `$JS.API.CONSUMER.PAUSE.<stream>.<consumer>`
#[derive(Debug, Deserialize)]
pub struct ConsumerPauseRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_until: Option<String>,
}

// ── Consumer responses ────────────────────────────────────────────────────────

/// `$JS.API.CONSUMER.CREATE` / `DURABLE.CREATE` / `INFO`
#[derive(Debug, Serialize)]
pub struct ConsumerInfoResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub info: ConsumerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.CONSUMER.LIST`
#[derive(Debug, Serialize)]
pub struct ConsumerListResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub page: PageInfo,
    pub consumers: Vec<ConsumerInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.CONSUMER.NAMES`
#[derive(Debug, Serialize)]
pub struct ConsumerNamesResponse {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub page: PageInfo,
    pub consumers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.CONSUMER.DELETE`
#[derive(Debug, Serialize)]
pub struct ConsumerDeleteResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.CONSUMER.LEADER.STEPDOWN`
#[derive(Debug, Serialize)]
pub struct ConsumerLeaderResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

/// `$JS.API.CONSUMER.PAUSE`
#[derive(Debug, Serialize)]
pub struct ConsumerPauseResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub paused: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsError>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Direct Get
// ═══════════════════════════════════════════════════════════════════════════════

/// `$JS.API.DIRECT.GET.<stream>` request body
#[derive(Debug, Deserialize)]
pub struct DirectGetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_by_subj: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_by_subj: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
}

/// Direct Get responses are raw HMSG with metadata in headers — no JSON body.
/// This struct represents the headers the server sets on the response.
#[derive(Debug, Serialize)]
pub struct DirectGetHeaders {
    /// `Nats-Stream`
    pub stream: String,
    /// `Nats-Sequence`
    pub sequence: u64,
    /// `Nats-Subject`
    pub subject: String,
    /// `Nats-Time-Stamp`
    pub timestamp: String,
    /// `Nats-Num-Pending` (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_pending: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACK
// ═══════════════════════════════════════════════════════════════════════════════

/// Optional body for `-NAK {"delay": N}` — delay in nanoseconds.
#[derive(Debug, Default, Deserialize)]
pub struct NakRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u64>,
}

/// Optional body for `-NXT` — request next batch.
#[derive(Debug, Default, Deserialize)]
pub struct AckNextRequest {
    #[serde(default = "default_batch")]
    pub batch: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Advisory Events
// ═══════════════════════════════════════════════════════════════════════════════

/// Common envelope for all `$JS.EVENT.ADVISORY.*` payloads.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdvisoryEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    pub timestamp: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// KV Store
// ═══════════════════════════════════════════════════════════════════════════════

/// Publish ACK returned after a KV put (same as JetStream publish ACK).
#[derive(Debug, Serialize, Deserialize)]
pub struct KvPutResponse {
    pub stream: String,
    pub seq: u64,
    pub duplicate: bool,
}

/// Metadata returned in HMSG headers for a KV get response.
#[derive(Debug, Serialize)]
pub struct KvGetHeaders {
    /// `Nats-Stream`
    pub stream: String,
    /// `Nats-Subject`   e.g. `$KV.<bucket>.<key>`
    pub subject: String,
    /// `Nats-Sequence`
    pub sequence: u64,
    /// `Nats-Time-Stamp`
    pub timestamp: String,
    /// `KV-Operation`  empty = PUT, "DEL" = delete, "PURGE" = purge
    pub operation: String,
    /// `Nats-Num-Pending`  remaining revisions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_pending: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Object Store
// ═══════════════════════════════════════════════════════════════════════════════

/// Metadata published to `$OBJ.<bucket>.info.<object>` before chunk upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub nonce: String,
    pub bucket: String,
    pub chunks: u64,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

/// Response returned by get-info and after successful upload.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub name: String,
    pub description: String,
    pub nonce: String,
    pub bucket: String,
    pub chunks: u64,
    pub size: u64,
    pub digest: String,
    pub deleted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// `$OBJ.<bucket>` — get / delete / info / list request body.
#[derive(Debug, Deserialize)]
pub struct ObjectRequest {
    pub name: String,
    /// Only used for get: subject to stream chunks to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deliver_subject: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ObjectDeleteResponse {
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct ObjectListResponse {
    pub bucket: String,
    pub objects: Vec<ObjectInfo>,
}
