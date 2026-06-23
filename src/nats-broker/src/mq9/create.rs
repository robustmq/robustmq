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
use crate::core::subject::try_get_or_init_subject;
use crate::core::tenant::get_tenant;
use crate::handler::command::NatsProcessContext;
use crate::storage::mail::Mq9MailStorage;
use bytes::Bytes;
use common_base::{tools::now_second, uuid::unique_id};
use metadata_struct::mq9::mail::MQ9Mail;
use mq9_core::protocol::{MailboxCreateReply, MailboxCreateReq};
use std::sync::Arc;

fn build_mail_address(name: Option<String>) -> Result<String, NatsBrokerError> {
    match name {
        Some(n) => {
            validate_mail_name(&n)?;
            Ok(n)
        }
        None => Ok(unique_id()),
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

    Mq9MailStorage::new(ctx.client_pool.clone())
        .create(&mail)
        .await?;

    try_get_or_init_subject(
        &ctx.cache_manager,
        &ctx.storage_driver_manager,
        &ctx.client_pool,
        &ctx.subscribe_manager,
        &get_tenant(),
        &mail_address,
        true,
    )
    .await?;

    Ok(MailboxCreateReply {
        error: String::new(),
        mail_address,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_mail_address, validate_mail_name};

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
            "alice"
        );
        assert!(build_mail_address(Some(".abc".to_string())).is_err());
        let auto = build_mail_address(None).unwrap();
        assert!(!auto.is_empty());
    }
}
