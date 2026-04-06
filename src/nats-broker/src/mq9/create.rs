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
use crate::core::tenant::get_tenant;
use crate::core::write_client::write_nats_packet;
use crate::handler::command::NatsProcessContext;
use crate::storage::email::Mq9EmailStorage;
use bytes::Bytes;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use metadata_struct::mq9::email::MQ9Email;
use mq9_core::protocol::{CreateMailboxReply, CreateMailboxReq};
use protocol::nats::packet::NatsPacket;
use uuid::Uuid;

fn build_email(payload: &Bytes) -> Result<MQ9Email, NatsBrokerError> {
    let params: CreateMailboxReq = serde_json::from_slice(payload).map_err(|e| {
        NatsBrokerError::CommonError(format!("invalid MAILBOX.CREATE payload: {}", e))
    })?;

    let tenant = get_tenant();

    let mail_id = if params.public {
        params.name.ok_or_else(|| {
            NatsBrokerError::CommonError("public mailbox requires a 'name' field".to_string())
        })?
    } else {
        Uuid::new_v4().to_string()
    };

    Ok(MQ9Email {
        mail_id,
        tenant,
        desc: params.desc,
        public: params.public,
        ttl: params
            .ttl
            .unwrap_or_else(|| broker_config().nats_runtime.mq9_mailbox_ttl),
        create_time: now_second(),
    })
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

pub async fn process_create(
    ctx: &NatsProcessContext,
    reply_to: Option<&str>,
    _headers: &Option<Bytes>,
    payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    let email = build_email(payload)?;
    let mail_id = email.mail_id.clone();

    let is_new = ctx
        .cache_manager
        .get_email(&email.tenant, &email.mail_id)
        .is_none();

    if is_new {
        Mq9EmailStorage::new(ctx.client_pool.clone())
            .create(&email)
            .await?;
    }

    if let Some(reply_subject) = reply_to {
        let response = serde_json::to_string(&CreateMailboxReply {
            mail_id: mail_id.clone(),
            is_new,
        })
        .unwrap_or_default();
        reply_nats_packet(ctx, reply_subject, Bytes::from(response)).await?;
    }

    Ok(())
}
