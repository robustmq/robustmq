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

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// INFO is sent by the server immediately after a client connects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerInfo {
    pub server_id: String,
    pub server_name: String,
    pub version: String,
    pub proto: u8,
    pub host: String,
    pub port: u16,
    pub headers: bool,
    pub auth_required: bool,
    pub tls_required: bool,
    pub tls_verify: bool,
    pub tls_available: bool,
    pub max_payload: u64,
    pub jetstream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_dynamic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_connect_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldm: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xkey: Option<String>,
}

/// CONNECT is sent by the client after receiving INFO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientConnect {
    pub verbose: bool,
    pub pedantic: bool,
    pub tls_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,
    pub name: String,
    pub lang: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_responders: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nkey: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NatsPacket {
    // Server → Client
    Info(ServerInfo),
    Msg {
        subject: String,
        sid: String,
        reply_to: Option<String>,
        payload: Bytes,
    },
    /// HMSG: server delivers a message with custom headers.
    /// `headers` is the raw header block: `NATS/1.0[status]\r\n[Key: Value\r\n...]\r\n`.
    HMsg {
        subject: String,
        sid: String,
        reply_to: Option<String>,
        headers: Bytes,
        payload: Bytes,
    },
    Ping,
    Pong,
    Ok,
    Err(String),

    // Client → Server
    Connect(ClientConnect),
    Pub {
        subject: String,
        reply_to: Option<String>,
        payload: Bytes,
    },
    /// HPUB: publish with custom headers.
    /// `headers` is the raw header block: `NATS/1.0\r\n[Key: Value\r\n...]\r\n`.
    HPub {
        subject: String,
        reply_to: Option<String>,
        headers: Bytes,
        payload: Bytes,
    },
    Sub {
        subject: String,
        queue_group: Option<String>,
        sid: String,
    },
    Unsub {
        sid: String,
        max_msgs: Option<u32>,
    },
}
