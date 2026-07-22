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

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::queue::{
    AMQPMethod, Bind, BindOk, Declare, DeclareOk as QueueDeclareOk, Delete, DeleteOk, Purge,
    PurgeOk, Unbind, UnbindOk,
};
use amq_protocol::protocol::AMQPClass;
use common_base::uuid::unique_id;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use metadata_struct::amqp::binding::{AmqpBinding, AmqpBindingDestinationType};
use metadata_struct::amqp::queue::AmqpQueue;
use metadata_struct::topic::{Topic, TopicConfig, TopicSource};
use storage_adapter::driver::StorageDriverManager;
use storage_adapter::topic::{create_topic_full, topic_replication_num};
use tracing::warn;

use crate::amqp::channel::channel_error_close;
use crate::amqp::route;
use crate::core::cache::AmqpCacheManager;
use crate::storage::binding::BindingStorage;
use crate::storage::queue::QueueStorage;

pub(crate) async fn process_queue_full(
    channel_id: u16,
    method: &AMQPMethod,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Declare(declare) => {
            process_queue_declare(
                channel_id,
                declare,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Delete(delete) => {
            process_queue_delete(
                channel_id,
                delete,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Bind(bind) => {
            process_queue_bind(
                channel_id,
                bind,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Unbind(unbind) => {
            process_queue_unbind(
                channel_id,
                unbind,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        AMQPMethod::Purge(purge) => {
            process_queue_purge(
                channel_id,
                purge,
                connection_id,
                amqp_cache,
                storage_driver_manager,
            )
            .await
        }
        _ => None,
    }
}

async fn process_queue_declare(
    channel_id: u16,
    declare: &Declare,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let queue_name = if declare.queue.as_str().is_empty() {
        format!("amqp-{}", unique_id())
    } else {
        declare.queue.to_string()
    };
    let tenant = amqp_cache.tenant_for(connection_id);

    // Passive: assert existence without declaring anything. A missing
    // queue is a channel exception (404 NOT_FOUND), not a silent create.
    if declare.passive {
        let exists = amqp_cache.get_queue(&tenant, &queue_name).is_some();
        if !exists {
            return Some(channel_error_close(channel_id, 404, "NOT_FOUND", 50, 10));
        }
        return if declare.nowait {
            None
        } else {
            Some(AMQPFrame::Method(
                channel_id,
                AMQPClass::Queue(AMQPMethod::DeclareOk(QueueDeclareOk {
                    queue: queue_name.into(),
                    message_count: 0,
                    consumer_count: 0,
                })),
            ))
        };
    }

    // The physical message shard: needed whether or not the queue's own
    // declare metadata is durable — something has to hold its messages
    // while it's alive.
    if declare_amqp_queue(storage_driver_manager, &tenant, &queue_name)
        .await
        .is_none()
    {
        warn!("AMQP Queue.Declare failed for queue={}", queue_name);
    }

    let arguments = route::field_table_to_map(&declare.arguments);
    let amqp_queue = AmqpQueue::new(
        &tenant,
        &queue_name,
        declare.durable,
        declare.exclusive,
        declare.auto_delete,
        arguments,
    );
    let storage = QueueStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    match storage.set_queue(&amqp_queue).await {
        Ok(()) => amqp_cache.set_queue(amqp_queue),
        Err(e) => warn!(
            "AMQP Queue.Declare metadata write failed for {}: {}",
            queue_name, e
        ),
    }

    if declare.nowait {
        return None;
    }
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Queue(AMQPMethod::DeclareOk(QueueDeclareOk {
            queue: queue_name.into(),
            message_count: 0,
            consumer_count: 0,
        })),
    ))
}

async fn process_queue_delete(
    channel_id: u16,
    delete: &Delete,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let queue_name = delete.queue.to_string();
    let tenant = amqp_cache.tenant_for(connection_id);
    let mut message_count: u64 = 0;

    if let Ok(resources) = storage_driver_manager
        .list_storage_resource(&tenant, &queue_name)
        .await
    {
        message_count = resources
            .values()
            .map(|d| d.offset.end_offset.saturating_sub(d.offset.start_offset))
            .sum();
    }
    if delete.if_empty && message_count > 0 {
        return Some(channel_error_close(
            channel_id,
            406,
            "PRECONDITION_FAILED",
            50,
            40,
        ));
    }

    let storage = QueueStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    match storage.delete_queue(&tenant, &queue_name).await {
        Ok(()) => amqp_cache.remove_queue(&tenant, &queue_name),
        Err(e) => warn!("AMQP Queue.Delete failed for {}: {}", queue_name, e),
    }
    // Metadata is gone either way at this point; also tear down the
    // underlying message shard so a later redeclare starts fresh
    // instead of silently resurrecting old messages.
    if let Err(e) = storage_driver_manager
        .delete_storage_resource(&tenant, &queue_name)
        .await
    {
        warn!(
            "AMQP Queue.Delete: failed to remove underlying storage for {}: {}",
            queue_name, e
        );
    }

    if delete.nowait {
        return None;
    }
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Queue(AMQPMethod::DeleteOk(DeleteOk {
            message_count: message_count as u32,
        })),
    ))
}

async fn process_queue_bind(
    channel_id: u16,
    bind: &Bind,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let tenant = amqp_cache.tenant_for(connection_id);
    if !bind.exchange.as_str().is_empty() {
        let exchange_exists = amqp_cache
            .get_exchange(&tenant, bind.exchange.as_str())
            .is_some();
        if !exchange_exists {
            return Some(channel_error_close(channel_id, 404, "NOT_FOUND", 50, 20));
        }
    }

    let arguments = route::field_table_to_map(&bind.arguments);
    let binding = AmqpBinding::new(
        &tenant,
        bind.exchange.as_str(),
        bind.queue.as_str(),
        AmqpBindingDestinationType::Queue,
        bind.routing_key.as_str(),
        arguments,
    );
    let storage = BindingStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    match storage.set_binding(&binding).await {
        Ok(()) => amqp_cache.set_binding(binding),
        Err(e) => warn!("AMQP Queue.Bind failed: {}", e),
    }

    if bind.nowait {
        return None;
    }
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Queue(AMQPMethod::BindOk(BindOk {})),
    ))
}

async fn process_queue_unbind(
    channel_id: u16,
    unbind: &Unbind,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let tenant = amqp_cache.tenant_for(connection_id);
    let storage = BindingStorage::new(
        storage_driver_manager
            .engine_storage_handler
            .client_pool
            .clone(),
    );
    let destination_type = AmqpBindingDestinationType::Queue;
    match storage
        .delete_binding(
            &tenant,
            unbind.exchange.as_str(),
            unbind.queue.as_str(),
            &destination_type,
            unbind.routing_key.as_str(),
        )
        .await
    {
        Ok(()) => {
            let key = format!(
                "{}/{}/{}/{}",
                unbind.exchange.as_str(),
                destination_type.as_str(),
                unbind.queue.as_str(),
                unbind.routing_key.as_str()
            );
            amqp_cache.remove_binding(&tenant, &key);
        }
        Err(e) => warn!("AMQP Queue.Unbind failed: {}", e),
    }

    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Queue(AMQPMethod::UnbindOk(UnbindOk {})),
    ))
}

/// Purges all messages currently in the queue's shard(s) while leaving the
/// queue itself declared, by deleting every record up to each partition's
/// current end_offset.
async fn process_queue_purge(
    channel_id: u16,
    purge: &Purge,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Option<AMQPFrame> {
    let queue_name = purge.queue.to_string();
    let tenant = amqp_cache.tenant_for(connection_id);
    let mut message_count: u64 = 0;

    match storage_driver_manager
        .list_storage_resource(&tenant, &queue_name)
        .await
    {
        Ok(resources) => {
            let targets: HashMap<u32, u64> = resources
                .iter()
                .map(|(partition, detail)| (*partition, detail.offset.end_offset))
                .collect();
            message_count = resources
                .values()
                .map(|d| d.offset.end_offset.saturating_sub(d.offset.start_offset))
                .sum();
            if let Err(e) = storage_driver_manager
                .delete_records_before(&tenant, &queue_name, &targets)
                .await
            {
                warn!("AMQP Queue.Purge failed for {}: {}", queue_name, e);
                message_count = 0;
            }
        }
        Err(e) => warn!("AMQP Queue.Purge: failed to inspect {}: {}", queue_name, e),
    }

    if purge.nowait {
        return None;
    }
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Queue(AMQPMethod::PurgeOk(PurgeOk {
            message_count: message_count as u32,
        })),
    ))
}

/// Creates (or fetches, if already declared) the topic backing an AMQP queue.
/// Reused by Queue.Declare and by Basic.Publish when it targets a queue that was
/// never explicitly declared.
pub(crate) async fn declare_amqp_queue(
    sdm: &Arc<StorageDriverManager>,
    tenant: &str,
    queue_name: &str,
) -> Option<Topic> {
    if let Some(topic) = sdm.broker_cache.get_topic_by_name(tenant, queue_name) {
        return Some(topic);
    }

    let conf = broker_config();
    let topic = Topic::new(tenant, queue_name, StorageType::EngineRocksDB)
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
        Ok(()) => sdm.broker_cache.get_topic_by_name(tenant, queue_name),
        Err(e) => {
            warn!("AMQP queue declare failed for {}: {}", queue_name, e);
            None
        }
    }
}
