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

//! MCP handlers for mq9 mailbox operations.
//!
//! | Tool                  | Backed by                          |
//! |-----------------------|------------------------------------|
//! | mq9_create_mailbox    | Mq9EmailStorage::create            |
//! | mq9_list_mailboxes    | Mq9EmailStorage::list              |
//! | mq9_delete_mailbox    | Mq9EmailStorage::delete            |
//!
//! `list` and `delete` are admin-only capabilities that bypass the mq9
//! protocol layer and call the storage layer directly.

use crate::state::NatsContext;
use common_base::error::common::CommonError;
use nats_broker::storage::email::Mq9EmailStorage;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct CreateMailboxArgs {
    pub tenant: String,
    pub mail_id: String,
    /// Optional description.
    pub desc: Option<String>,
    /// Whether the mailbox is public (visible in the system public mailbox list).
    pub public: Option<bool>,
    /// TTL in seconds. Falls back to broker config default when absent.
    pub ttl: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ListMailboxesArgs {
    pub tenant: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMailboxArgs {
    pub tenant: String,
    pub mail_id: String,
}

/// Create a new mailbox via Mq9EmailStorage.
pub async fn create_mailbox(
    ctx: &NatsContext,
    args: CreateMailboxArgs,
) -> Result<Value, CommonError> {
    use common_base::tools::now_second;
    use common_config::broker::broker_config;
    use metadata_struct::mq9::email::MQ9Email;

    let email = MQ9Email {
        mail_id: args.mail_id.clone(),
        tenant: args.tenant.clone(),
        desc: args.desc.unwrap_or_default(),
        public: args.public.unwrap_or(false),
        ttl: args
            .ttl
            .unwrap_or_else(|| broker_config().nats_runtime.mq9_mailbox_ttl),
        create_time: now_second(),
    };

    Mq9EmailStorage::new(ctx.cache_manager.client_pool.clone())
        .create(&email)
        .await?;

    Ok(json!({ "mail_id": args.mail_id, "created": true }))
}

/// List all mailboxes belonging to a tenant via Mq9EmailStorage.
pub async fn list_mailboxes(
    ctx: &NatsContext,
    args: ListMailboxesArgs,
) -> Result<Value, CommonError> {
    let emails = Mq9EmailStorage::new(ctx.cache_manager.client_pool.clone())
        .list(&args.tenant)
        .await?;

    let items: Vec<Value> = emails
        .into_iter()
        .map(|e| {
            json!({
                "mail_id":     e.mail_id,
                "tenant":      e.tenant,
                "desc":        e.desc,
                "public":      e.public,
                "ttl":         e.ttl,
                "create_time": e.create_time,
            })
        })
        .collect();

    Ok(json!({ "mailboxes": items }))
}

/// Delete a mailbox via Mq9EmailStorage.
pub async fn delete_mailbox(
    ctx: &NatsContext,
    args: DeleteMailboxArgs,
) -> Result<Value, CommonError> {
    Mq9EmailStorage::new(ctx.cache_manager.client_pool.clone())
        .delete(&args.tenant, &args.mail_id)
        .await?;

    Ok(json!({ "mail_id": args.mail_id, "deleted": true }))
}
