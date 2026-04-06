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

use crate::core::error::{NatsBrokerError, NatsProtocolError};
use crate::core::subject::try_get_or_init_subject;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::mq9::process::mq9_command;
use crate::nats::subscribe::subject_message_tag;
use crate::storage::message::MessageStorage;
use bytes::Bytes;
use common_config::broker::broker_config;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::record::{StorageRecordProtocolData, StorageRecordProtocolDataNats};
use mq9_core::command::Mq9Command;
use protocol::nats::packet::NatsPacket;

pub async fn process_pub(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Option<NatsPacket> {
    let connection = if let Some(connection) = ctx.cache_manager.get_connection(ctx.connect_id) {
        connection.clone()
    } else {
        return None;
    };

    if broker_config().nats_runtime.auth_required && !connection.is_login {
        return Some(NatsPacket::Err(
            NatsProtocolError::AuthorizationViolation.message(),
        ));
    }

    if Mq9Command::is_mq9_subject(subject) {
        return mq9_command(ctx, subject, reply_to, headers, payload).await;
    }

    if let Err(e) = process_pub0(ctx, subject, reply_to, payload, headers).await {
        if connection.verbose {
            return Some(NatsPacket::Err(e.to_string()));
        } else {
            return None;
        }
    }

    if connection.verbose {
        return Some(NatsPacket::Ok);
    }
    None
}

async fn process_pub0(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    payload: &Bytes,
    header: &Option<Bytes>,
) -> Result<(), NatsBrokerError> {
    let tenant = get_tenant();

    try_get_or_init_subject(
        &ctx.cache_manager,
        &ctx.storage_driver_manager,
        &ctx.client_pool,
        &ctx.subscribe_manager,
        &tenant,
        subject,
    )
    .await?;

    let message = MessageStorage::new(ctx.storage_driver_manager.clone());
    let reply_to_string = reply_to.map(|rt| rt.to_string());

    let record = AdapterWriteRecord::new(subject.to_string(), payload.clone())
        .with_tags(vec![subject_message_tag(&tenant, subject)])
        .with_protocol_data(Some(StorageRecordProtocolData {
            nats: Some(StorageRecordProtocolDataNats {
                reply_to: reply_to_string,
                header: header.clone(),
            }),
            mqtt: None,
        }));
    message.write(&tenant, subject, vec![record]).await?;
    Ok(())
}
