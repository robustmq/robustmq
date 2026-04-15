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
pub struct CreateMailboxReq {
    pub ttl: Option<u64>,
    #[serde(default)]
    pub public: bool,
    pub name: Option<String>,
    #[serde(default)]
    pub desc: String,
}

// ── Sub-reply items ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMailboxMsgItem {
    pub msg_id: u64,
    pub payload: String,
    pub priority: String,
    pub header: Option<Vec<u8>>,
    pub create_time: u64,
}

// ── Unified flat reply ────────────────────────────────────────────────────────

/// Flat JSON response for all mq9 commands.
///
/// All fields are `Option`; on success the relevant fields are populated and
/// `error` is `None`. On failure only `error` is set.
///
/// Examples:
/// - Create ok:  `{"mail_id":"mq9-…","is_new":true}`
/// - Publish ok: `{"msg_id":8}`
/// - List ok:    `{"mail_id":"mq9-…","messages":[…]}`
/// - Delete ok:  `{"deleted":true}`
/// - Error:      `{"error":"mailbox xxx does not exist"}`
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Mq9Reply {
    /// Always present. Empty string on success, error message on failure.
    pub error: String,

    // ── create fields ─────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_new: Option<bool>,

    // ── publish fields ────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,

    // ── list fields ───────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<ListMailboxMsgItem>>,

    // ── delete fields ─────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

impl Mq9Reply {
    pub fn ok_create(mail_id: String, is_new: bool) -> Self {
        Mq9Reply {
            mail_id: Some(mail_id),
            is_new: Some(is_new),
            ..Default::default()
        }
    }

    pub fn ok_publish(msg_id: u64) -> Self {
        Mq9Reply {
            msg_id: Some(msg_id),
            ..Default::default()
        }
    }

    pub fn ok_list(mail_id: String, messages: Vec<ListMailboxMsgItem>) -> Self {
        Mq9Reply {
            mail_id: Some(mail_id),
            messages: Some(messages),
            ..Default::default()
        }
    }

    pub fn ok_delete() -> Self {
        Mq9Reply {
            deleted: Some(true),
            ..Default::default()
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Mq9Reply {
            error: msg.into(),
            ..Default::default()
        }
    }

    pub fn is_error(&self) -> bool {
        !self.error.is_empty()
    }
}
