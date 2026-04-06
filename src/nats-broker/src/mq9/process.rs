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
use bytes::Bytes;
use mq9_core::subject::{Mq9Subject, Priority};
use protocol::nats::packet::NatsPacket;

pub async fn mq9_command(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Option<NatsPacket> {
    let parsed = match Mq9Subject::parse(subject) {
        Some(s) => s,
        None => {
            return Some(NatsPacket::Err(format!(
                "unrecognized mq9 subject: {}",
                subject
            )))
        }
    };

    let result = match parsed {
        Mq9Subject::MailboxCreate => process_create(ctx, reply_to, headers, payload).await,
        Mq9Subject::Mailbox { mail_id, priority } => {
            process_pub(ctx, &mail_id, &priority, reply_to, headers, payload).await
        }
        Mq9Subject::PublicList => process_public_list(ctx, reply_to, headers, payload).await,
    };

    match result {
        Ok(()) => None,
        Err(e) => Some(NatsPacket::Err(e.to_string())),
    }
}

async fn process_create(
    _ctx: &NatsProcessContext,
    _reply_to: Option<&str>,
    _headers: &Option<Bytes>,
    _payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    Ok(())
}

async fn process_pub(
    _ctx: &NatsProcessContext,
    _mail_id: &str,
    _priority: &Priority,
    _reply_to: Option<&str>,
    _headers: &Option<Bytes>,
    _payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    Ok(())
}

async fn process_public_list(
    _ctx: &NatsProcessContext,
    _reply_to: Option<&str>,
    _headers: &Option<Bytes>,
    _payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    Ok(())
}
