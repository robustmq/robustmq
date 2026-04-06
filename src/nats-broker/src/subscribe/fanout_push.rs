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
use crate::nats::subscribe::subject_message_tag;
use crate::subscribe::NatsSubscribeManager;
use crate::subscribe::NatsSubscriber;
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
use storage_adapter::consumer::{GroupConsumer, StartOffsetStrategy};
use storage_adapter::driver::StorageDriverManager;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tokio_util::codec::Encoder;
use tracing::{debug, error, info, warn};

pub const BATCH_SIZE: u64 = 500;
pub const IDLE_SLEEP_MS: u64 = 100;
pub const LOW_LOAD_SLEEP_MS: u64 = 3000;
pub const HIGH_LOAD_SLEEP_MS: u64 = 3000;
pub const LOW_LOAD_THRESHOLD: usize = 3000;

pub struct FanoutPushManager {
    subscribe_manager: Arc<NatsSubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    bucket_id: String,
    consumers: DashMap<String, Arc<GroupConsumer>>,
}

impl FanoutPushManager {
    pub fn new(
        subscribe_manager: Arc<NatsSubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
        storage_driver_manager: Arc<StorageDriverManager>,
        bucket_id: String,
    ) -> Self {
        FanoutPushManager {
            subscribe_manager,
            connection_manager,
            storage_driver_manager,
            bucket_id,
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
                            info!("NATS FanoutPushManager[{}] stopped", self.bucket_id);
                            break;
                        }
                        Ok(false) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("NATS FanoutPushManager[{}] stop channel closed", self.bucket_id);
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            debug!(
                                "NATS FanoutPushManager[{}] stop channel lagged, skipped {}",
                                self.bucket_id, n
                            );
                        }
                    }
                }
                res = self.send_messages() => {
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
                            error!(
                                "NATS FanoutPushManager[{}] error: {}",
                                self.bucket_id, e
                            );
                            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                        }
                    }
                }
            }
        }
    }

    async fn send_messages(&self) -> Result<usize, NatsBrokerError> {
        let mut processed = 0;
        let mut stale: Vec<(u64, String, String)> = Vec::new();

        let subscribers: Vec<NatsSubscriber> = self
            .subscribe_manager
            .fanout_push
            .buckets_data_list
            .get(&self.bucket_id)
            .map(|bucket| bucket.iter().map(|e| e.value().clone()).collect())
            .unwrap_or_default();

        for subscriber in subscribers {
            if !self
                .subscribe_manager
                .allow_push_client(subscriber.connect_id)
            {
                continue;
            }

            match self.process_subscriber(&subscriber).await {
                Ok(count) => processed += count,
                Err(NatsBrokerError::ConnectionNotFound(_)) => {
                    warn!(
                        "NATS subscriber gone, removing: connect_id={} sid={}",
                        subscriber.connect_id, subscriber.sid
                    );
                    stale.push((
                        subscriber.connect_id,
                        subscriber.sid.clone(),
                        subscriber.uniq_id.clone(),
                    ));
                }
                Err(e) => {
                    debug!(
                        "NATS push failed [connect_id={}, topic={}, sid={}]: {}",
                        subscriber.connect_id, subscriber.subject, subscriber.sid, e
                    );
                    self.subscribe_manager
                        .add_not_push_client(subscriber.connect_id);
                }
            }
        }

        for (connect_id, sid, uniq_id) in stale {
            self.subscribe_manager.remove_push_by_sid(connect_id, &sid);
            self.consumers.remove(&uniq_id);
        }

        Ok(processed)
    }

    async fn get_or_create_consumer(&self, subscriber: &NatsSubscriber) -> Arc<GroupConsumer> {
        if let Some(consumer) = self.consumers.get(&subscriber.uniq_id) {
            return consumer.clone();
        }
        let consumer = Arc::new(GroupConsumer::new_manual(
            self.storage_driver_manager.clone(),
            subscriber.uniq_id.clone(),
        ));
        consumer
            .set_start_offset_strategy(StartOffsetStrategy::Latest)
            .await;
        self.consumers
            .insert(subscriber.uniq_id.clone(), consumer.clone());
        consumer
    }

    async fn process_subscriber(
        &self,
        subscriber: &NatsSubscriber,
    ) -> Result<usize, NatsBrokerError> {
        let read_config = AdapterReadConfig {
            max_record_num: BATCH_SIZE,
            max_size: 1024 * 1024 * 30,
        };

        let consumer = self.get_or_create_consumer(subscriber).await;
        let tag = subject_message_tag(&subscriber.tenant, &subscriber.subject);
        let records = match consumer
            .next_messages_by_tags(&subscriber.tenant, &subscriber.subject, &tag, &read_config)
            .await
        {
            Err(e) => return Err(NatsBrokerError::from(e)),
            Ok(r) if r.is_empty() => return Ok(0),
            Ok(r) => r,
        };
        println!("records:{:?}", records);
        let mut pushed = 0;
        for record in &records {
            match send_packet(&self.connection_manager, subscriber, record).await {
                Ok(true) => pushed += 1,
                Ok(false) => {}
                Err(NatsBrokerError::ConnectionNotFound(_)) => {
                    return Err(NatsBrokerError::ConnectionNotFound(subscriber.connect_id));
                }
                Err(e) => {
                    debug!(
                        "NATS send failed [connect_id={}, sid={}]: {}",
                        subscriber.connect_id, subscriber.sid, e
                    );
                }
            }
        }

        consumer.advance();
        Ok(pushed)
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
            subject: subscriber.subject.clone(),
            sid: subscriber.sid.clone(),
            reply_to,
            headers,
            payload: Bytes::copy_from_slice(&record.data),
        }
    } else {
        NatsPacket::Msg {
            subject: subscriber.subject.clone(),
            sid: subscriber.sid.clone(),
            reply_to,
            payload: Bytes::copy_from_slice(&record.data),
        }
    };

    if connection_manager.is_websocket(connect_id) {
        let mut codec = NatsCodec::new();
        let mut buf = BytesMut::new();
        codec
            .encode(packet.clone(), &mut buf)
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
        let wrapper = RobustMQPacketWrapper {
            protocol: RobustMQProtocol::NATS,
            extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
            packet: RobustMQPacket::NATS(packet),
        };
        connection_manager
            .write_websocket_frame(connect_id, wrapper, Message::Binary(buf.to_vec().into()))
            .await
            .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
    } else {
        let wrapper = RobustMQPacketWrapper {
            protocol: RobustMQProtocol::NATS,
            extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
            packet: RobustMQPacket::NATS(packet),
        };
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
    (nats.reply_to.clone(), nats.header.clone())
}
