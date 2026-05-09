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
use crate::core::write_client::write_nats_packet;
use crate::handler::command::NatsProcessContext;
use crate::mq9::agent::{
    process_agent_discover, process_agent_register, process_agent_report, process_agent_unregister,
};
use crate::mq9::create::process_create;
use crate::mq9::delete::process_delete;
use crate::mq9::fetch::{process_ack, process_fetch};
use crate::mq9::query::process_query;
use crate::mq9::send::process_send;
use bytes::Bytes;
use mq9_core::command::Mq9Command;
use mq9_core::protocol::err_reply;
use protocol::nats::packet::NatsPacket;

pub async fn mq9_command(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Option<NatsPacket> {
    let parsed = match Mq9Command::parse(subject) {
        Some(s) => s,
        None => {
            return Some(NatsPacket::Err(format!(
                "unrecognized mq9 subject: {}",
                subject
            )))
        }
    };

    let response_json = match parsed {
        Mq9Command::MailboxCreate => match process_create(ctx, payload).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::MsgSend {
            mail_address,
            priority,
        } => match process_send(ctx, &mail_address, &priority, headers, reply_to, payload).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::MsgFetch { mail_address } => {
            match process_fetch(ctx, &mail_address, reply_to, payload).await {
                Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
                Err(e) => err_reply(e.to_string()),
            }
        }
        Mq9Command::MsgAck { mail_address } => match process_ack(ctx, &mail_address, payload).await
        {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::MsgQuery { mail_address } => {
            match process_query(ctx, &mail_address, payload).await {
                Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
                Err(e) => err_reply(e.to_string()),
            }
        }
        Mq9Command::MsgDelete {
            mail_address,
            msg_id,
        } => match process_delete(ctx, &mail_address, &msg_id).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::AgentRegister => match process_agent_register(ctx, payload).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::AgentUnregister => match process_agent_unregister(ctx, payload).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::AgentReport => match process_agent_report(ctx, payload).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
        Mq9Command::AgentDiscover => match process_agent_discover(ctx, payload).await {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => err_reply(e.to_string()),
        },
    };

    if let Some(reply_subject) = reply_to {
        let _ = reply_nats_packet(ctx, reply_subject, Bytes::from(response_json)).await;
    }

    None
}

async fn reply_nats_packet(
    ctx: &NatsProcessContext,
    subject: &str,
    payload: Bytes,
) -> Result<(), NatsBrokerError> {
    let sid = ctx
        .cache_manager
        .get_inbox_sid(subject)
        .unwrap_or_else(|| "0".to_string());
    let packet = NatsPacket::Msg {
        subject: subject.to_string(),
        sid,
        reply_to: None,
        payload,
    };
    write_nats_packet(&ctx.connection_manager, ctx.connect_id, packet).await
}
