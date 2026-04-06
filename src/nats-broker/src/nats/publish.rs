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

/// Returns `Ok(None)` for normal commands (verbose handled by caller),
/// `Ok(Some(packet))` for server-initiated replies (e.g. mq9 reply-to MSG),
/// `Err(packet)` for protocol errors.
pub async fn process_pub(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Result<Option<NatsPacket>, NatsPacket> {
    let connection = ctx.cache_manager.get_connection(ctx.connect_id);

    if broker_config().nats_runtime.auth_required {
        let is_login = connection.as_ref().map(|c| c.is_login).unwrap_or(false);
        if !is_login {
            return Err(NatsPacket::Err(
                NatsProtocolError::AuthorizationViolation.message(),
            ));
        }
    }

    if Mq9Command::is_mq9_subject(subject) {
        // mq9 commands send their own replies; return as server-initiated packet
        let pkt = mq9_command(ctx, subject, reply_to, headers, payload).await;
        return Ok(pkt);
    }

    process_pub0(ctx, subject, reply_to, payload, headers)
        .await
        .map_err(|e| NatsPacket::Err(e.to_string()))?;

    Ok(None)
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
        false,
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
            mq9: None,
        }));
    message.write(&tenant, subject, vec![record]).await?;
    Ok(())
}
