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
use crate::core::subject::try_get_or_init_subject;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::nats::subscribe::subject_message_tag;
use crate::storage::message::MessageStorage;
use bytes::Bytes;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::mq9::Priority;
use metadata_struct::storage::record::{StorageRecordProtocolData, StorageRecordProtocolDataMq9};
use mq9_core::protocol::{Mq9Reply, PubMailboxReply};

pub async fn process_pub(
    ctx: &NatsProcessContext,
    mail_id: &str,
    priority: &Priority,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Result<Mq9Reply, NatsBrokerError> {
    let tenant = get_tenant();

    if ctx.cache_manager.get_email(&tenant, mail_id).is_none() {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} does not exist",
            mail_id
        )));
    }

    try_get_or_init_subject(
        &ctx.cache_manager,
        &ctx.storage_driver_manager,
        &ctx.client_pool,
        &ctx.subscribe_manager,
        &tenant,
        mail_id,
        true,
    )
    .await?;

    let record = AdapterWriteRecord::new(mail_id.to_string(), payload.clone())
        .with_tags(vec![subject_message_tag(&tenant, mail_id)])
        .with_protocol_data(Some(StorageRecordProtocolData {
            mq9: Some(StorageRecordProtocolDataMq9 {
                priority: priority.to_string(),
                header: headers.clone(),
            }),
            nats: None,
            mqtt: None,
        }));

    let offsets = MessageStorage::new(ctx.storage_driver_manager.clone())
        .write(&tenant, mail_id, vec![record])
        .await?;

    let offset = offsets.into_iter().next().unwrap_or(0);
    Ok(Mq9Reply::Pub(PubMailboxReply {
        mail_id: mail_id.to_string(),
        msg_id: offset,
    }))
}
