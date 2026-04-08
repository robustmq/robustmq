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
use crate::storage::message::MessageStorage;
use bytes::Bytes;
use common_base::{tools::now_second, uuid::unique_id};
use common_config::broker::broker_config;
use metadata_struct::mq9::email::MQ9Email;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use mq9_core::protocol::{CreateMailboxReply, CreateMailboxReq, Mq9Reply};
use mq9_core::public::{is_system_mailbox, StoragePublicData, MQ9_SYSTEM_PUBLIC_MAIL};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

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
        format!("email-{}-{}", unique_id(), unique_id())
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

pub async fn process_create(
    ctx: &NatsProcessContext,
    payload: &Bytes,
) -> Result<Mq9Reply, NatsBrokerError> {
    let email = build_email(payload)?;
    let mail_id = email.mail_id.clone();

    if is_system_mailbox(&mail_id) {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox '{}' is reserved and cannot be created by clients",
            mail_id
        )));
    }

    let is_new = ctx
        .cache_manager
        .get_email(&email.tenant, &email.mail_id)
        .is_none();

    if is_new {
        Mq9EmailStorage::new(ctx.client_pool.clone())
            .create(&email)
            .await?;
    }

    if email.public {
        save_public_data(
            &ctx.storage_driver_manager,
            &email.mail_id,
            &email.desc,
            email.ttl,
        )
        .await?;
    }

    Ok(Mq9Reply::Create(CreateMailboxReply { mail_id, is_new }))
}

pub async fn save_public_data(
    storage_driver_manager: &Arc<StorageDriverManager>,
    mail_id: &str,
    desc: &str,
    ttl: u64,
) -> Result<(), NatsBrokerError> {
    let data = StoragePublicData {
        mail_id: mail_id.to_string(),
        ttl,
        desc: desc.to_string(),
        create_at: now_second(),
    };
    let payload = serde_json::to_string(&data)?;
    let record = AdapterWriteRecord::new(MQ9_SYSTEM_PUBLIC_MAIL.to_string(), payload.clone())
        .with_key(mail_id);
    let _offsets = MessageStorage::new(storage_driver_manager.clone())
        .write(DEFAULT_TENANT, MQ9_SYSTEM_PUBLIC_MAIL, vec![record])
        .await?;
    Ok(())
}
