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

use crate::core::topic::{
    try_get_or_init_mq9_subject, try_get_or_init_nats_core_subject, NatSubject,
};
use crate::core::{cache::NatsCacheManager, error::NatsBrokerError};
use crate::push::{
    manager::NatsSubscribeManager,
    parse::{ParseAction, ParseSubscribeData},
};
use grpc_clients::pool::ClientPool;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
pub const INBOX_PREFIX: &str = "_INBOX.";

pub fn is_inbox_subject(subject: &str) -> bool {
    subject.starts_with(INBOX_PREFIX)
}

pub async fn try_get_or_init_subject(
    cache_manager: &Arc<NatsCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    tenant: &str,
    subject: &str,
    is_mq9: bool,
) -> Result<NatSubject, NatsBrokerError> {
    if tenant.is_empty() {
        return Err(NatsBrokerError::CommonError(
            "Tenant cannot be empty".to_string(),
        ));
    }

    if let Some(topic) = cache_manager.node_cache.get_topic_by_name(tenant, subject) {
        return Ok(topic);
    }

    let topic = if is_mq9 {
        try_get_or_init_mq9_subject(
            cache_manager,
            storage_driver_manager,
            client_pool,
            tenant,
            subject,
        )
        .await?
    } else {
        try_get_or_init_nats_core_subject(
            cache_manager,
            storage_driver_manager,
            client_pool,
            tenant,
            subject,
        )
        .await?
    };

    let data = ParseSubscribeData::new_topic(ParseAction::Add, topic.clone());
    subscribe_manager.send_parse_event(data).await;

    Ok(topic)
}
