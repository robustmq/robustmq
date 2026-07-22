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

use std::collections::HashMap;
use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use storage_adapter::driver::StorageDriverManager;
use tracing::warn;

use crate::amqp::unacked_index;

pub(crate) async fn requeue_message(
    sdm: &Arc<StorageDriverManager>,
    tenant: &str,
    queue: &str,
    offset: u64,
    index_offset: u64,
) -> Result<(), CommonError> {
    let Some(topic) = sdm.broker_cache.get_topic_by_name(tenant, queue) else {
        warn!(
            "AMQP requeue: queue {} no longer exists, dropping index entry",
            queue
        );
        return unacked_index::delete_entry(sdm, index_offset).await;
    };
    let Some(shard_name) = topic.storage_name_list.get(&0).cloned() else {
        warn!(
            "AMQP requeue: queue {} has no shard, dropping index entry",
            queue
        );
        return unacked_index::delete_entry(sdm, index_offset).await;
    };

    let read_config = AdapterReadConfig::new();
    let mut offsets = HashMap::new();
    offsets.insert(shard_name, offset);
    let records = sdm
        .read_by_offset(tenant, queue, &offsets, &read_config)
        .await?;
    let Some(record) = records.into_iter().next() else {
        return unacked_index::delete_entry(sdm, index_offset).await;
    };

    let mut protocol_data = record.protocol_data.unwrap_or_default();
    let mut amqp = protocol_data.amqp.unwrap_or_default();
    amqp.redelivered = true;
    protocol_data.amqp = Some(amqp);

    let new_record = AdapterWriteRecord::new(queue.to_string(), record.data.to_vec())
        .with_protocol_data(Some(protocol_data));
    sdm.write(tenant, queue, std::slice::from_ref(&new_record), 1)
        .await?;

    sdm.delete_by_offsets(tenant, queue, &[offset]).await?;

    unacked_index::delete_entry(sdm, index_offset).await
}
