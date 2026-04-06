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
use crate::mq9::create::process_create;
use crate::mq9::public_list::process_public_list;
use crate::mq9::publish::process_pub;
use bytes::Bytes;
use mq9_core::command::Mq9Command;
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

    let result = match parsed {
        Mq9Command::MailboxCreate => process_create(ctx, payload).await,
        Mq9Command::Mailbox { mail_id, priority } => {
            process_pub(ctx, &mail_id, &priority, headers, payload).await
        }
        Mq9Command::PublicList => process_public_list(ctx, payload).await,
    };

    if let Some(reply_subject) = reply_to {
        let response = match result {
            Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
            Err(e) => e.to_string(),
        };
        let _ = reply_nats_packet(ctx, reply_subject, Bytes::from(response)).await;
    }

    None
}

async fn reply_nats_packet(
    ctx: &NatsProcessContext,
    subject: &str,
    payload: Bytes,
) -> Result<(), NatsBrokerError> {
    let packet = NatsPacket::Msg {
        subject: subject.to_string(),
        sid: "0".to_string(),
        reply_to: None,
        payload,
    };
    write_nats_packet(&ctx.connection_manager, ctx.connect_id, packet).await
}
