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

use std::sync::atomic::Ordering;
use std::sync::Arc;

use amq_protocol::protocol::basic::{AMQPMethod, Deliver};
use amq_protocol::protocol::AMQPClass;
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::broker::common::call::broker_send_share_group_message;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::share_group::{ShareGroupMember, ShareGroupParams};
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use protocol::broker::broker::{
    send_share_group_message_request::Detail, AmqpShareGroupDetail, SendShareGroupMessageRequest,
};
use protocol::robust::{
    AmqpWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::broadcast;
use tracing::{error, warn};

use crate::amqp::basic::properties_from_record;
use crate::amqp::queue::declare_amqp_queue;
use crate::core::cache::{AmqpCacheManager, UnackedEntry};
use crate::core::frame::build_basic_content_frames;
use crate::core::unacked_index;
use crate::push::common::{adaptive_sleep, should_stop};
use crate::push::manager::AmqpPushManager;
use crate::storage::offset::OffsetStorage;

pub struct AmqpQueuePushParams {
    pub connection_manager: Arc<ConnectionManager>,
    pub storage_driver_manager: Arc<StorageDriverManager>,
    pub amqp_cache: Arc<AmqpCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub push_manager: Arc<AmqpPushManager>,
    pub tenant: String,
    pub queue: String,
}

/// Drives Basic.Consume delivery for one queue on the node that leads it.
pub struct AmqpQueuePush {
    params: AmqpQueuePushParams,
    round_robin: u64,
}

impl AmqpQueuePush {
    pub fn new(params: AmqpQueuePushParams) -> Self {
        AmqpQueuePush {
            params,
            round_robin: 0,
        }
    }

    pub async fn start(&mut self, stop_sx: &broadcast::Sender<bool>) {
        let mut stop_rx = stop_sx.subscribe();
        let label = format!(
            "AMQP QueuePush[{}/{}]",
            self.params.tenant, self.params.queue
        );
        loop {
            tokio::select! {
                val = stop_rx.recv() => {
                    if should_stop(val, &label) { break; }
                }
                res = self.send_messages() => {
                    match res {
                        Ok(count) => adaptive_sleep(count).await,
                        Err(e) => {
                            error!("{} error: {}", label, e);
                            adaptive_sleep(0).await;
                        }
                    }
                }
            }
        }
    }

    async fn send_messages(&mut self) -> Result<usize, CommonError> {
        let members = self
            .params
            .storage_driver_manager
            .broker_cache
            .get_share_group_members(&self.params.tenant, &self.params.queue);
        if members.is_empty() {
            return Ok(0);
        }

        let Some(topic) = declare_amqp_queue(
            &self.params.storage_driver_manager,
            &self.params.tenant,
            &self.params.queue,
        )
        .await
        else {
            return Ok(0);
        };
        let Some(shard_name) = topic.storage_name_list.get(&0).cloned() else {
            return Ok(0);
        };

        let offset_storage = OffsetStorage::new(
            self.params
                .storage_driver_manager
                .engine_storage_handler
                .client_pool
                .clone(),
        );

        let mut pushed = 0;
        loop {
            if !members.iter().any(|m| match &m.params {
                ShareGroupParams::AMQP(detail) => self.member_ready(m, detail),
                _ => false,
            }) {
                // No member currently has room (Qos prefetch exhausted) or is
                // flow-paused; stop claiming so the message stays unclaimed
                // instead of being consumed with nowhere to go.
                break;
            }
            let claimed = offset_storage
                .claim_next_record(
                    &self.params.push_manager,
                    &self.params.storage_driver_manager,
                    &self.params.tenant,
                    &self.params.queue,
                    &shard_name,
                )
                .await?;
            let Some((record, msg_offset)) = claimed else {
                break;
            };

            let start_idx = self.round_robin as usize % members.len();
            self.round_robin = self.round_robin.wrapping_add(1);
            if self
                .deliver(&members, start_idx, &record, msg_offset)
                .await?
            {
                pushed += 1;
            } else {
                warn!(
                    "AMQP queue [{}/{}]: claimed offset {} but no consumer could take it",
                    self.params.tenant, self.params.queue, msg_offset
                );
            }
        }
        Ok(pushed)
    }

    /// Whether `member` can currently accept a push: gated on Channel.Flow
    /// and (for ack-required consumers) Basic.Qos prefetch_count.
    ///
    /// Qos/Flow state lives on the consumer's own connection, which may be a
    /// different node than the one driving this queue's push loop (the
    /// leader). We can only see that state for members local to this node;
    /// remote members are never gated here (best-effort, not enforced
    /// cluster-wide).
    fn member_ready(
        &self,
        member: &ShareGroupMember,
        detail: &metadata_struct::mqtt::share_group::ShareGroupParamsAmqp,
    ) -> bool {
        if member.broker_id != broker_config().broker_id {
            return true;
        }
        let Some(channel) = self
            .params
            .amqp_cache
            .get_channel(member.connect_id, detail.channel_id)
        else {
            return false;
        };
        if !channel.flow_active.load(Ordering::SeqCst) {
            return false;
        }
        if detail.no_ack {
            return true;
        }
        let limit = channel.prefetch_count.load(Ordering::SeqCst);
        limit == 0
            || self
                .params
                .amqp_cache
                .unacked_count(member.connect_id, detail.channel_id)
                < limit as usize
    }

    /// Tries each member round-robin from `start_idx` until one accepts.
    async fn deliver(
        &self,
        members: &[ShareGroupMember],
        start_idx: usize,
        record: &StorageRecord,
        msg_offset: u64,
    ) -> Result<bool, CommonError> {
        let self_broker_id = broker_config().broker_id;

        for i in 0..members.len() {
            let member = &members[(start_idx + i) % members.len()];
            let ShareGroupParams::AMQP(detail) = &member.params else {
                continue;
            };
            if !self.member_ready(member, detail) {
                continue;
            }

            let index_offset = if detail.no_ack {
                None
            } else {
                Some(
                    unacked_index::write_entry(
                        &self.params.storage_driver_manager,
                        &self.params.tenant,
                        &self.params.queue,
                        msg_offset,
                        member.connect_id,
                        detail.channel_id,
                        member.broker_id,
                    )
                    .await?,
                )
            };

            let delivered = if member.broker_id == self_broker_id {
                self.deliver_local(member, detail, record, msg_offset, index_offset)
                    .await
            } else {
                self.deliver_remote(member, detail, record, msg_offset, index_offset)
                    .await
            };

            match delivered {
                Ok(true) => return Ok(true),
                Ok(false) => {
                    if let Some(index_offset) = index_offset {
                        if let Err(e) = unacked_index::delete_entry(
                            &self.params.storage_driver_manager,
                            index_offset,
                        )
                        .await
                        {
                            warn!(
                                "AMQP queue push: failed to clean up stale index entry: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "AMQP queue push: delivery attempt failed for connect_id={}: {}",
                        member.connect_id, e
                    );
                    if let Some(index_offset) = index_offset {
                        if let Err(e) = unacked_index::delete_entry(
                            &self.params.storage_driver_manager,
                            index_offset,
                        )
                        .await
                        {
                            warn!(
                                "AMQP queue push: failed to clean up stale index entry: {}",
                                e
                            );
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    async fn deliver_local(
        &self,
        member: &ShareGroupMember,
        detail: &metadata_struct::mqtt::share_group::ShareGroupParamsAmqp,
        record: &StorageRecord,
        msg_offset: u64,
        index_offset: Option<u64>,
    ) -> Result<bool, CommonError> {
        deliver_to_local_connection(
            &self.params.connection_manager,
            &self.params.amqp_cache,
            member.connect_id,
            detail.channel_id,
            &detail.consumer_tag,
            &self.params.tenant,
            &self.params.queue,
            record,
            msg_offset,
            index_offset,
        )
        .await
    }

    async fn deliver_remote(
        &self,
        member: &ShareGroupMember,
        detail: &metadata_struct::mqtt::share_group::ShareGroupParamsAmqp,
        record: &StorageRecord,
        msg_offset: u64,
        index_offset: Option<u64>,
    ) -> Result<bool, CommonError> {
        let Some(node) = self
            .params
            .storage_driver_manager
            .broker_cache
            .node_lists
            .get(&member.broker_id)
        else {
            return Ok(false);
        };
        let addr = node.grpc_addr.clone();
        drop(node);

        let request = SendShareGroupMessageRequest {
            connect_id: member.connect_id,
            record: record.encode()?,
            detail: Some(Detail::Amqp(AmqpShareGroupDetail {
                channel_id: detail.channel_id as u32,
                consumer_tag: detail.consumer_tag.clone(),
                tenant: self.params.tenant.clone(),
                queue: self.params.queue.clone(),
                offset: msg_offset,
                index_offset,
            })),
        };

        match broker_send_share_group_message(&self.params.client_pool, &[addr], request).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!(
                    "AMQP queue push: remote delivery failed [broker_id={}, connect_id={}]: {}",
                    member.broker_id, member.connect_id, e
                );
                Ok(false)
            }
        }
    }
}

/// Writes a Basic.Deliver to a local consumer connection; shared by the
/// leader's own local delivery and the `SendShareGroupMessage` gRPC handler.
#[allow(clippy::too_many_arguments)]
pub async fn deliver_to_local_connection(
    connection_manager: &Arc<ConnectionManager>,
    amqp_cache: &Arc<AmqpCacheManager>,
    connect_id: u64,
    channel_id: u16,
    consumer_tag: &str,
    tenant: &str,
    queue: &str,
    record: &StorageRecord,
    msg_offset: u64,
    index_offset: Option<u64>,
) -> Result<bool, CommonError> {
    let Some(channel) = amqp_cache.get_channel(connect_id, channel_id) else {
        return Ok(false);
    };

    let delivery_tag = channel.next_delivery_tag.fetch_add(1, Ordering::SeqCst);
    let body = record.data.to_vec();
    let deliver_frame = amq_protocol::frame::AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::Deliver(Deliver {
            consumer_tag: consumer_tag.into(),
            delivery_tag,
            redelivered: false,
            exchange: "".into(),
            routing_key: queue.into(),
        })),
    );
    let frames = build_basic_content_frames(
        channel_id,
        deliver_frame,
        body,
        properties_from_record(record),
    );

    let wrapper = RobustMQPacketWrapper {
        protocol: RobustMQProtocol::AMQP,
        extend: RobustMQWrapperExtend::AMQP(AmqpWrapperExtend {}),
        packet: RobustMQPacket::AMQP(frames),
    };
    connection_manager
        .write_tcp_frame(connect_id, wrapper)
        .await
        .map_err(|e| CommonError::CommonError(e.to_string()))?;

    if let Some(index_offset) = index_offset {
        amqp_cache.unacked().insert(
            (connect_id, channel_id, delivery_tag),
            UnackedEntry {
                tenant: tenant.to_string(),
                queue: queue.to_string(),
                offset: msg_offset,
                index_offset,
            },
        );
    }
    Ok(true)
}
