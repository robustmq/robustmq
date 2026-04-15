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
use crate::nats::subscribe::subject_message_tag;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use mq9_core::protocol::{ListMailboxMsgItem, Mq9Reply};
use std::collections::HashMap;

const BATCH_SIZE: u64 = 500;
const MAX_LIST_MSGS: usize = 1000;

pub async fn process_list(
    ctx: &NatsProcessContext,
    mail_id: &str,
) -> Result<Mq9Reply, NatsBrokerError> {
    let tenant = get_tenant();
    let tag = subject_message_tag(&tenant, mail_id);
    let read_config = AdapterReadConfig {
        max_record_num: BATCH_SIZE,
        max_size: 1024 * 1024 * 30,
    };

    let mut messages = Vec::new();
    // Track per-shard offsets to paginate through all records.
    let mut offsets: HashMap<String, u64> = HashMap::new();

    loop {
        if messages.len() >= MAX_LIST_MSGS {
            break;
        }

        let records = ctx
            .storage_driver_manager
            .read_by_tag(&tenant, mail_id, &tag, &offsets, &read_config)
            .await
            .map_err(NatsBrokerError::from)?;

        if records.is_empty() {
            break;
        }

        for record in &records {
            if messages.len() >= MAX_LIST_MSGS {
                break;
            }
            // Advance per-shard offset past this record.
            let shard = &record.metadata.shard;
            let next = record.metadata.offset + 1;
            let entry = offsets.entry(shard.clone()).or_insert(0);
            if next > *entry {
                *entry = next;
            }

            let (priority, header) = record
                .protocol_data
                .as_ref()
                .and_then(|pd| pd.mq9.as_ref())
                .map(|mq9| {
                    (
                        mq9.priority.clone(),
                        mq9.header.as_ref().map(|h| h.to_vec()),
                    )
                })
                .unwrap_or_default();

            messages.push(ListMailboxMsgItem {
                msg_id: record.metadata.offset,
                payload: String::from_utf8_lossy(&record.data).into_owned(),
                priority,
                header,
                create_time: record.metadata.create_t,
            });
        }
    }

    Ok(Mq9Reply::ok_list(mail_id.to_string(), messages))
}
