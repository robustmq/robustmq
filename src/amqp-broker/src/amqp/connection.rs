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

use std::sync::Arc;

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::connection::{
    AMQPMethod, Close, CloseOk, Open, OpenOk, Start, StartOk, Tune, UpdateSecretOk,
};
use amq_protocol::protocol::AMQPClass;
use amq_protocol::types::{FieldTable, LongString};
use common_security::login::password::password_check_by_login;
use common_security::manager::SecurityManager;
use metadata_struct::tenant::DEFAULT_TENANT;
use tracing::warn;

use crate::core::cache::AmqpCacheManager;
use crate::core::connection::{AmqpConnection, AmqpConnectionState};

/// Handles the Connection class. StartOk/Open/Close/CloseOk carry the
/// login+tenant handshake and connection lifecycle; everything else is a
/// plain protocol ack handled by `process_connection` below.
pub(crate) async fn process_connection_full(
    channel_id: u16,
    method: &AMQPMethod,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    security_manager: &Arc<SecurityManager>,
) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::StartOk(start_ok) => {
            process_connection_start_ok(start_ok, connection_id, amqp_cache)
        }
        AMQPMethod::Open(open) => {
            process_connection_open(open, connection_id, amqp_cache, security_manager).await
        }
        AMQPMethod::Close(_) => process_connection_close(connection_id, amqp_cache),
        AMQPMethod::CloseOk(_) => process_connection_close_ok(connection_id, amqp_cache),
        _ => process_connection(channel_id, method),
    }
}

/// Parses a SASL PLAIN response per RFC 4616: [authzid] NUL authcid NUL passwd.
fn parse_sasl_plain(response: &[u8]) -> Option<(String, String)> {
    let parts: Vec<&[u8]> = response.split(|b| *b == 0).collect();
    if parts.len() != 3 {
        return None;
    }
    let username = String::from_utf8(parts[1].to_vec()).ok()?;
    let password = String::from_utf8(parts[2].to_vec()).ok()?;
    Some((username, password))
}

/// Captures the SASL PLAIN credentials for verification once Open reveals
/// which tenant (vhost) they should be checked against, then replies Tune.
fn process_connection_start_ok(
    start_ok: &StartOk,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
) -> Option<AMQPFrame> {
    if let Some((username, password)) = parse_sasl_plain(start_ok.response.as_bytes()) {
        amqp_cache.set_pending_login(connection_id, username, password);
    } else {
        warn!("AMQP Connection.StartOk: unsupported SASL response format");
    }
    Some(tune_frame())
}

async fn process_connection_open(
    open: &Open,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    security_manager: &Arc<SecurityManager>,
) -> Option<AMQPFrame> {
    let tenant = if open.virtual_host.as_str().is_empty() {
        DEFAULT_TENANT.to_string()
    } else {
        open.virtual_host.to_string()
    };

    let login = amqp_cache.take_pending_login(connection_id);
    let authenticated = match &login {
        Some((username, password)) => {
            password_check_by_login(security_manager, &tenant, username, password)
        }
        None => false,
    };

    if !authenticated {
        warn!(
            connection_id,
            "AMQP Connection.Open authentication failed for vhost={}", tenant
        );
        return Some(close_frame(530, "NOT_ALLOWED", 10, 40));
    }

    let mut conn = amqp_cache
        .get_connection(connection_id)
        .unwrap_or_else(|| AmqpConnection::new(connection_id));
    conn.tenant = tenant;
    if let Some((username, _)) = login {
        conn.username = username;
    }
    conn.state = AmqpConnectionState::Open;
    amqp_cache.set_connection(conn);

    Some(open_ok_frame())
}

fn process_connection_close(
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
) -> Option<AMQPFrame> {
    amqp_cache.remove_connection(connection_id);
    Some(close_ok_frame())
}

fn process_connection_close_ok(
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
) -> Option<AMQPFrame> {
    amqp_cache.remove_connection(connection_id);
    None
}

pub fn process_protocol_header() -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        0,
        AMQPClass::Connection(AMQPMethod::Start(Start {
            version_major: 0,
            version_minor: 9,
            server_properties: FieldTable::default(),
            mechanisms: LongString::from("PLAIN"),
            locales: LongString::from("en_US"),
        })),
    ))
}

pub fn process_heartbeat(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Heartbeat(channel_id))
}

// StartOk/Open/Close/CloseOk need AmqpCacheManager/SecurityManager access (login,
// connection-state tracking) and are handled in command.rs. Everything else here
// is a plain protocol ack.
pub fn process_connection(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::SecureOk(_) => process_secure_ok(channel_id),
        AMQPMethod::TuneOk(_) => process_tune_ok(channel_id),
        AMQPMethod::Blocked(_) => process_blocked(channel_id),
        AMQPMethod::Unblocked(_) => process_unblocked(channel_id),
        AMQPMethod::UpdateSecret(_) => process_update_secret(channel_id),
        _ => None,
    }
}

pub(crate) fn tune_frame() -> AMQPFrame {
    AMQPFrame::Method(
        0,
        AMQPClass::Connection(AMQPMethod::Tune(Tune {
            channel_max: 2047,
            frame_max: 131072,
            heartbeat: 60,
        })),
    )
}

pub(crate) fn open_ok_frame() -> AMQPFrame {
    AMQPFrame::Method(0, AMQPClass::Connection(AMQPMethod::OpenOk(OpenOk {})))
}

pub(crate) fn close_ok_frame() -> AMQPFrame {
    AMQPFrame::Method(0, AMQPClass::Connection(AMQPMethod::CloseOk(CloseOk {})))
}

pub(crate) fn close_frame(
    reply_code: u16,
    reply_text: &str,
    class_id: u16,
    method_id: u16,
) -> AMQPFrame {
    AMQPFrame::Method(
        0,
        AMQPClass::Connection(AMQPMethod::Close(Close {
            reply_code,
            reply_text: reply_text.into(),
            class_id,
            method_id,
        })),
    )
}

fn process_secure_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_tune_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_blocked(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_unblocked(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_update_secret(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Connection(AMQPMethod::UpdateSecretOk(UpdateSecretOk {})),
    ))
}
