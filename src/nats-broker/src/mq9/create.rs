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

use crate::core::cache::NatsCacheManager;
use crate::core::error::NatsBrokerError;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::storage::mail::Mq9MailStorage;
use crate::storage::message::MessageStorage;
use bytes::Bytes;
use common_base::{tools::now_second, uuid::unique_id};
use metadata_struct::mq9::mail::MQ9Mail;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::tenant::DEFAULT_TENANT;
use mq9_core::protocol::{MailboxCreateReply, MailboxCreateReq};
use mq9_core::public::{is_system_mailbox, StoragePublicData, MQ9_SYSTEM_PUBLIC_MAIL};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;

const MQ9_MAIL_SUFFIX: &str = "@mq9";

fn build_mail_address(name: Option<String>) -> Result<String, NatsBrokerError> {
    match name {
        Some(n) => {
            validate_mail_name(&n)?;
            Ok(format!("{}{}", n, MQ9_MAIL_SUFFIX))
        }
        None => Ok(format!("{}{}", unique_id(), MQ9_MAIL_SUFFIX)),
    }
}

fn validate_mail_name(name: &str) -> Result<(), NatsBrokerError> {
    if name.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "prefix must not be empty".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')
    {
        return Err(NatsBrokerError::CommonError(
            "name may only contain lowercase letters, digits, and dots".to_string(),
        ));
    }
    let is_alphanumeric = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    if !name.starts_with(is_alphanumeric) || !name.ends_with(is_alphanumeric) {
        return Err(NatsBrokerError::CommonError(
            "prefix must start and end with a lowercase letter or digit".to_string(),
        ));
    }
    Ok(())
}

fn build_mail(
    cache_manager: &Arc<NatsCacheManager>,
    payload: &Bytes,
) -> Result<MQ9Mail, NatsBrokerError> {
    let create_req: MailboxCreateReq = if payload.is_empty() {
        MailboxCreateReq {
            name: None,
            ttl: None,
            desc: None,
        }
    } else {
        serde_json::from_slice(payload).map_err(|e| {
            NatsBrokerError::CommonError(format!("invalid MAILBOX.CREATE payload: {}", e))
        })?
    };

    let tenant = get_tenant();
    let mail_address = build_mail_address(create_req.name)?;

    Ok(MQ9Mail {
        mail_address,
        tenant,
        desc: create_req.desc.unwrap_or_default(),
        ttl: create_req.ttl.unwrap_or_else(|| {
            cache_manager
                .node_cache
                .get_cluster_config()
                .nats_runtime
                .mq9_mailbox_default_ttl
        }),
        create_time: now_second(),
    })
}

pub async fn process_create(
    ctx: &NatsProcessContext,
    payload: &Bytes,
) -> Result<MailboxCreateReply, NatsBrokerError> {
    let mail = build_mail(&ctx.cache_manager, payload)?;
    let mail_address = mail.mail_address.clone();

    if ctx
        .cache_manager
        .get_mail(&mail.tenant, &mail.mail_address)
        .is_some()
    {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} already exists",
            mail_address
        )));
    }

    if is_system_mailbox(&mail_address) {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox '{}' is reserved and cannot be created by clients",
            mail_address
        )));
    }

    Mq9MailStorage::new(ctx.client_pool.clone())
        .create(&mail)
        .await?;

    Ok(MailboxCreateReply {
        error: String::new(),
        mail_address,
    })
}

pub async fn save_public_data(
    storage_driver_manager: &Arc<StorageDriverManager>,
    mail_address: &str,
    desc: &str,
    ttl: u64,
) -> Result<(), NatsBrokerError> {
    let data = StoragePublicData {
        mail_address: mail_address.to_string(),
        ttl,
        desc: desc.to_string(),
        create_at: now_second(),
    };
    let payload = serde_json::to_string(&data)?;
    let record = AdapterWriteRecord::new(MQ9_SYSTEM_PUBLIC_MAIL.to_string(), payload.clone())
        .with_key(mail_address);
    let _offsets = MessageStorage::new(storage_driver_manager.clone())
        .write(DEFAULT_TENANT, MQ9_SYSTEM_PUBLIC_MAIL, vec![record])
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_mail_address, validate_mail_name, MQ9_MAIL_SUFFIX};

    #[test]
    fn test_validate_mail_name() {
        assert!(validate_mail_name("a1.b2.c3").is_ok());
        assert!(validate_mail_name("").is_err());
        assert!(validate_mail_name("Abc").is_err());
        assert!(validate_mail_name("a-b").is_err());
        assert!(validate_mail_name(".leading").is_err());
        assert!(validate_mail_name("trailing.").is_err());
    }

    #[test]
    fn test_build_mail_address() {
        assert_eq!(
            build_mail_address(Some("alice".to_string())).unwrap(),
            format!("alice{}", MQ9_MAIL_SUFFIX)
        );
        assert!(build_mail_address(Some(".abc".to_string())).is_err());
        let auto = build_mail_address(None).unwrap();
        assert!(auto.ends_with(MQ9_MAIL_SUFFIX));
    }
}
