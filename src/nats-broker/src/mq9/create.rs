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
use crate::handler::command::NatsProcessContext;
use crate::storage::email::Mq9EmailStorage;
use axum::extract::ws::Message;
use bytes::{Bytes, BytesMut};
use common_base::tools::now_second;
use metadata_struct::mq9::email::MQ9Email;
use network_server::common::connection_manager::ConnectionManager;
use protocol::nats::codec::NatsCodec;
use protocol::nats::packet::NatsPacket;
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::codec::Encoder;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CreateMailboxReply {
    pub mail_id: String,
}

#[derive(Deserialize)]
pub struct CreateEmailPayload {
    pub ttl: Option<u64>,
    #[serde(default)]
    pub public: bool,
    pub name: Option<String>,
    #[serde(default)]
    pub desc: String,
}

fn build_email(payload: &Bytes) -> Result<MQ9Email, NatsBrokerError> {
    let params: CreateEmailPayload = serde_json::from_slice(payload).map_err(|e| {
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
        ttl: params.ttl.unwrap_or(3600),
        create_time: now_second(),
    })
}

async fn reply_nats_packet(
    connection_manager: &Arc<ConnectionManager>,
    connect_id: u64,
    subject: &str,
    payload: Bytes,
) -> Result<(), NatsBrokerError> {
    let packet = NatsPacket::Msg {
        subject: subject.to_string(),
        sid: "0".to_string(),
        reply_to: None,
        payload,
    };
    let wrapper = RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(packet.clone()),
    };

    if connection_manager.is_websocket(connect_id) {
        let mut codec = NatsCodec::new();
        let mut buf = BytesMut::new();
        codec
            .encode(packet, &mut buf)
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
        connection_manager
            .write_websocket_frame(connect_id, wrapper, Message::Binary(buf.to_vec().into()))
            .await
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
    } else {
        connection_manager
            .write_tcp_frame(connect_id, wrapper)
            .await
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
    }

    Ok(())
}

pub async fn process_create(
    ctx: &NatsProcessContext,
    reply_to: Option<&str>,
    _headers: &Option<Bytes>,
    payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    let email = build_email(payload)?;
    let mail_id = email.mail_id.clone();

    if ctx
        .cache_manager
        .get_email(&email.tenant, &email.mail_id)
        .is_some()
    {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} already exists",
            mail_id
        )));
    }

    Mq9EmailStorage::new(ctx.client_pool.clone())
        .create(&email)
        .await?;

    if let Some(reply_subject) = reply_to {
        let response = serde_json::to_string(&CreateMailboxReply {
            mail_id: mail_id.clone(),
        })
        .unwrap_or_default();
        reply_nats_packet(
            &ctx.connection_manager,
            ctx.connect_id,
            reply_subject,
            Bytes::from(response),
        )
        .await?;
    }

    Ok(())
}
