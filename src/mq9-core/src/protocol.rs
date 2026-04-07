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

#[derive(Debug, Deserialize)]
pub struct CreateMailboxReq {
    pub ttl: Option<u64>,
    #[serde(default)]
    pub public: bool,
    pub name: Option<String>,
    #[serde(default)]
    pub desc: String,
}

// ── Replies ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CreateMailboxReply {
    pub mail_id: String,
    pub is_new: bool,
}

#[derive(Debug, Serialize)]
pub struct PubMailboxReply {
    pub msg_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ListMailboxMsgItem {
    pub msg_id: u64,
    pub payload: String,
    pub priority: String,
    pub header: Option<Vec<u8>>,
    pub create_time: u64,
}

#[derive(Debug, Serialize)]
pub struct ListMailboxMsgReply {
    pub mail_id: String,
    pub messages: Vec<ListMailboxMsgItem>,
}

#[derive(Debug, Serialize)]
pub struct DeleteMailboxMsgReply {
    pub deleted: bool,
}

// ── Reply enum ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Mq9Reply {
    Create(CreateMailboxReply),
    Pub(PubMailboxReply),
    List(ListMailboxMsgReply),
    Delete(DeleteMailboxMsgReply),
}
