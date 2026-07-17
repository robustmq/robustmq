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

use std::sync::Arc;

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::queue::{AMQPMethod, PurgeOk};
use amq_protocol::protocol::AMQPClass;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use metadata_struct::tenant::DEFAULT_TENANT;
use metadata_struct::topic::{Topic, TopicConfig, TopicSource};
use storage_adapter::driver::StorageDriverManager;
use storage_adapter::topic::{create_topic_full, topic_replication_num};
use tracing::warn;

// Queue.Declare/Delete/Bind/Unbind need storage access, so they are handled
// in command.rs. Everything else in this file is a plain protocol ack.
pub fn process_queue(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Purge(_) => process_purge(channel_id),
        _ => None,
    }
}

/// Creates (or fetches, if already declared) the topic backing an AMQP queue.
/// Reused by Queue.Declare and by Basic.Publish when it targets a queue that was
/// never explicitly declared.
pub(crate) async fn declare_amqp_queue(
    sdm: &Arc<StorageDriverManager>,
    queue_name: &str,
) -> Option<Topic> {
    if let Some(topic) = sdm
        .broker_cache
        .get_topic_by_name(DEFAULT_TENANT, queue_name)
    {
        return Some(topic);
    }

    let conf = broker_config();
    let topic = Topic::new(DEFAULT_TENANT, queue_name, StorageType::EngineRocksDB)
        .with_source(TopicSource::AMQP)
        .with_partition(conf.runtime.default_topic_partition_num)
        .with_replication(topic_replication_num(
            conf.runtime.default_topic_replica_num,
        ))
        .with_config(TopicConfig::default());

    match create_topic_full(
        &sdm.broker_cache,
        sdm,
        &sdm.engine_storage_handler.client_pool,
        &topic,
    )
    .await
    {
        Ok(()) => sdm
            .broker_cache
            .get_topic_by_name(DEFAULT_TENANT, queue_name),
        Err(e) => {
            warn!("AMQP queue declare failed for {}: {}", queue_name, e);
            None
        }
    }
}

fn process_purge(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Queue(AMQPMethod::PurgeOk(PurgeOk { message_count: 0 })),
    ))
}
