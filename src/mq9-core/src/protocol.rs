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

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRegisterReq {
    pub name: String,
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentReportReq {
    pub name: String,
    pub report_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MailboxCreateReq {
    pub name: Option<String>,
    pub ttl: Option<u64>,
    pub desc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MsgQueryReq {
    pub key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub since: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverPolicy {
    Earliest,
    Latest,
    FromTime,
    FromId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgAckReq {
    pub group_name: String,
    pub mail_address: String,
    pub msg_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgFetchConfig {
    pub num_msgs: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgFetchReq {
    pub group_name: String,
    pub deliver: DeliverPolicy,
    pub from_time: Option<u64>,
    pub from_id: Option<u64>,
    pub force_deliver: Option<bool>,
    pub config: Option<MsgFetchConfig>,
}

// ── Reply item ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgItem {
    pub msg_id: u64,
    pub payload: String,
    pub priority: String,
    pub header: Option<Vec<u8>>,
    pub create_time: u64,
}

// ── Per-command reply types ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MailboxCreateReply {
    pub error: String,
    pub mail_address: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MsgSendReply {
    pub error: String,
    #[serde(default)]
    pub msg_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MsgFetchReply {
    pub error: String,
    pub messages: Vec<MsgItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MsgAckReply {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MsgQueryReply {
    pub error: String,
    pub messages: Vec<MsgItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MsgDeleteReply {
    pub error: String,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AgentRegisterReply {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AgentUnregisterReply {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AgentReportReply {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AgentDiscoverReply {
    pub error: String,
    pub agents: Vec<serde_json::Value>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn err_reply(error: impl Into<String>) -> String {
    #[derive(Serialize)]
    struct ErrOnly {
        error: String,
    }
    serde_json::to_string(&ErrOnly {
        error: error.into(),
    })
    .unwrap_or_default()
}
