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
use crate::subscribe::{NatsSubscribeManager, NatsSubscriber};
use axum::extract::ws::Message;
use bytes::{Bytes, BytesMut};
use dashmap::DashMap;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use protocol::nats::codec::NatsCodec;
use protocol::nats::packet::NatsPacket;
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::consumer::GroupConsumer;
use storage_adapter::driver::StorageDriverManager;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tokio_util::codec::Encoder;
use tracing::{debug, error, info, warn};

const BATCH_SIZE: u64 = 500;
const IDLE_SLEEP_MS: u64 = 100;
const LOW_LOAD_SLEEP_MS: u64 = 50;
const HIGH_LOAD_SLEEP_MS: u64 = 10;
const LOW_LOAD_THRESHOLD: usize = 10;

pub struct DirectlyPushManager {
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    // group_name → GroupConsumer
    consumers: DashMap<String, GroupConsumer>,
}

impl DirectlyPushManager {
    pub fn new(
        subscribe_manager: Arc<NatsSubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
    ) -> Self {
        DirectlyPushManager {
            subscribe_manager,
            connection_manager,
            storage_driver_manager,
            consumers: DashMap::with_capacity(64),
        }
    }

    pub async fn start(&self, stop_sx: &broadcast::Sender<bool>) {
        let mut stop_rx = stop_sx.subscribe();
        loop {
            select! {
                val = stop_rx.recv() => {
                    match val {
                        Ok(true) => {
                            info!("NATS DirectlyPushManager stopped");
                            break;
                        }
                        Ok(false) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("NATS DirectlyPushManager stop channel closed");
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            debug!("NATS DirectlyPushManager stop channel lagged, skipped {}", n);
                        }
                    }
                }
                res = self.push_all() => {
                    match res {
                        Ok(count) => {
                            if count == 0 {
                                sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                            } else if count < LOW_LOAD_THRESHOLD {
                                sleep(Duration::from_millis(LOW_LOAD_SLEEP_MS)).await;
                            } else {
                                sleep(Duration::from_millis(HIGH_LOAD_SLEEP_MS)).await;
                            }
                        }
                        Err(e) => {
                            error!("NATS DirectlyPushManager push error: {}", e);
                            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                        }
                    }
                }
            }
        }
    }

    async fn push_all(&self) -> Result<usize, NatsBrokerError> {
        let mut total = 0;

        // Collect all (topic, seq, subscriber) to avoid holding DashMap locks across awaits.
        let entries: Vec<(String, u64, NatsSubscriber)> = self
            .subscribe_manager
            .directly_push
            .iter()
            .flat_map(|topic_entry| {
                let topic = topic_entry.key().clone();
                topic_entry
                    .value()
                    .iter()
                    .map(|sub_entry| (topic.clone(), *sub_entry.key(), sub_entry.value().clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut stale: Vec<(String, u64)> = Vec::new();

        for (topic_name, seq, subscriber) in entries {
            if !self
                .subscribe_manager
                .allow_push_client(subscriber.connect_id, &topic_name)
            {
                continue;
            }

            match self.push_subscriber(&subscriber).await {
                Ok(count) => total += count,
                Err(NatsBrokerError::ConnectionNotFound(_)) => {
                    warn!(
                        "NATS subscriber connection not found, removing: connect_id={} sid={}",
                        subscriber.connect_id, subscriber.sid
                    );
                    stale.push((topic_name, seq));
                }
                Err(e) => {
                    debug!(
                        "NATS push failed [connect_id={}, topic={}, sid={}]: {}",
                        subscriber.connect_id, topic_name, subscriber.sid, e
                    );
                    self.subscribe_manager
                        .add_not_push_client(subscriber.connect_id, &topic_name);
                }
            }
        }

        for (topic_name, seq) in stale {
            self.subscribe_manager
                .remove_directly_subscriber(&topic_name, seq);
        }

        Ok(total)
    }

    async fn push_subscriber(&self, subscriber: &NatsSubscriber) -> Result<usize, NatsBrokerError> {
        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        let tenant = get_tenant();

        let mut consumer_entry = self
            .consumers
            .entry(subscriber.group_name.clone())
            .or_insert_with(|| {
                GroupConsumer::new_manual(
                    self.storage_driver_manager.clone(),
                    subscriber.group_name.clone(),
                )
            });

        let records = consumer_entry
            .next_messages(&tenant, &subscriber.topic_name, &read_config)
            .await
            .map_err(NatsBrokerError::from)?;

        if records.is_empty() {
            return Ok(0);
        }

        let mut pushed = 0;
        for record in &records {
            match self.send_to_client(subscriber, record).await {
                Ok(true) => pushed += 1,
                Ok(false) => {}
                Err(e) => return Err(e),
            }
        }

        consumer_entry
            .commit()
            .await
            .map_err(NatsBrokerError::from)?;

        Ok(pushed)
    }

    async fn send_to_client(
        &self,
        subscriber: &NatsSubscriber,
        record: &StorageRecord,
    ) -> Result<bool, NatsBrokerError> {
        send_packet(&self.connection_manager, subscriber, record).await
    }
}

pub async fn send_packet(
    connection_manager: &Arc<ConnectionManager>,
    subscriber: &NatsSubscriber,
    record: &StorageRecord,
) -> Result<bool, NatsBrokerError> {
    let connect_id = subscriber.connect_id;

    if connection_manager.get_connect(connect_id).is_none() {
        return Err(NatsBrokerError::ConnectionNotFound(connect_id));
    }

    let (reply_to, headers) = extract_nats_meta(record);

    let packet = if let Some(headers) = headers {
        NatsPacket::HMsg {
            subject: subscriber.topic_name.clone(),
            sid: subscriber.sid.clone(),
            reply_to,
            headers,
            payload: Bytes::copy_from_slice(&record.data),
        }
    } else {
        NatsPacket::Msg {
            subject: subscriber.topic_name.clone(),
            sid: subscriber.sid.clone(),
            reply_to,
            payload: Bytes::copy_from_slice(&record.data),
        }
    };

    let wrapper = RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(packet.clone()),
    };

    if connection_manager.is_websocket(connect_id) {
        let mut codec = NatsCodec::new();
        let mut buf = BytesMut::new();
        codec
            .encode(packet, &mut buf)
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
        connection_manager
            .write_websocket_frame(connect_id, wrapper, Message::Binary(buf.to_vec().into()))
            .await
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
    } else {
        connection_manager
            .write_tcp_frame(connect_id, wrapper)
            .await
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
    }

    Ok(true)
}

fn extract_nats_meta(record: &StorageRecord) -> (Option<String>, Option<Bytes>) {
    let Some(proto) = &record.protocol_data else {
        return (None, None);
    };
    let Some(nats) = &proto.nats else {
        return (None, None);
    };
    let headers = nats.header.clone();
    (nats.reply_to.clone(), headers)
}
