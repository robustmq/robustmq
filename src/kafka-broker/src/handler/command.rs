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

use axum::async_trait;
use kafka_protocol::messages::api_versions_response::ApiVersion;
use kafka_protocol::messages::{ApiKey, ApiVersionsResponse, ProduceRequest, ProduceResponse};
use kafka_protocol::messages::produce_response::{PartitionProduceResponse, TopicProduceResponse};
use kafka_protocol::records::{RecordBatchDecoder, RecordBatchEncoder};
use metadata_struct::connection::NetworkConnection;
use network_server::{command::Command, common::packet::ResponsePackage};
use protocol::kafka::packet::KafkaPacket;
use protocol::robust::RobustMQPacket;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::log::{info, warn};
use storage_adapter::storage::ArcStorageAdapter;
use crate::{
    common::error::Result,
    storage::message::{Message, TopicPartition},
};
use crate::manager::offset::OffsetManager;
use crate::storage::log_reader::Reader;
use crate::storage::log_writer::Writer;

pub struct KafkaCommand {
    pub writer: Writer,
    pub reader: Reader,
    pub offset_manager: OffsetManager,
}

impl KafkaCommand {
    pub fn new(storage_adapter: ArcStorageAdapter) -> Self {
        Self {
            writer: Writer::new(storage_adapter.clone()),
            reader: Reader::new(storage_adapter.clone()),
            offset_manager: OffsetManager::new(),
        }
    }

    fn handle_api_versions() -> Result<KafkaPacket> {
        let api_versions: Vec<ApiVersion> = ApiKey::iter()
            .map(|k| {
                let range = ApiKey::valid_versions(&k);
                ApiVersion::default()
                    .with_api_key(k as i16)
                    .with_min_version(range.min)
                    .with_max_version(range.max)
            })
            .collect();
        let res = ApiVersionsResponse::default()
            .with_error_code(0)
            .with_api_keys(api_versions)
            .with_throttle_time_ms(0);
        Ok(KafkaPacket::ApiVersionResponse(res))
    }

    async fn handle_produce(
        &self,
        request: ProduceRequest,
    ) -> Result<KafkaPacket> {
        let mut topic_responses = vec![];
        let mut message_batch = vec![];

        for topic_data in &request.topic_data {
            let topic_name = topic_data.name.as_str();
            let mut partition_responses = vec![];
            for partition_data in &topic_data.partition_data {
                let partition_index = partition_data.index;
                let mut base_offset = None;
                let mut partition_error_code = 0i16;
                if let Some(records_bytes) = &partition_data.records {
                    let mut records_buf = records_bytes.clone();
                    let result = RecordBatchDecoder::decode(&mut records_buf);
                    match result {
                        Ok(record_set) => {
                            let mut log_start_offset = 0i64;
                            if record_set.records.len() > 0 && base_offset.is_none() {
                                log_start_offset = self.offset_manager.next_offset(topic_name, partition_index)
                                    .await
                                    .unwrap_or(0);
                                base_offset = Some(log_start_offset);
                            }
                            for record in &record_set.records {
                                log_start_offset += 1;
                                message_batch.push(Message {
                                    topic_partition: TopicPartition {
                                        topic: topic_name.to_string(),
                                        partition: partition_index,
                                    },
                                    offset: log_start_offset,
                                    record: Message::encode(record.clone()),
                                });
                            }
                        }
                        Err(e) => {
                            warn!("Failed to decode record batch: {:?}", e);
                            partition_error_code = 1;
                        }
                    }
                }

                let partition_resp = PartitionProduceResponse::default()
                    .with_index(partition_index)
                    .with_error_code(partition_error_code)
                    .with_base_offset(base_offset.unwrap_or_default())
                    .with_log_append_time_ms(-1) // message.timestamp.type
                    .with_log_start_offset(0);

                partition_responses.push(partition_resp);
            }

            let topic_resp = TopicProduceResponse::default()
                .with_name(topic_data.name.clone())
                .with_partition_responses(partition_responses);

            topic_responses.push(topic_resp);
        }

        for msg in message_batch.into_iter() {
            self.writer.write(&msg).await?;
        }

        let response = ProduceResponse::default()
            .with_responses(topic_responses)
            .with_throttle_time_ms(0);
        Ok(KafkaPacket::ProduceResponse(response))
    }
}

#[async_trait]
impl Command for KafkaCommand {
    async fn apply(
        &self,
        tcp_connection: &NetworkConnection,
        _addr: &SocketAddr,
        robust_packet: &RobustMQPacket,
    ) -> Option<ResponsePackage> {
        let packet = match robust_packet.get_kafka_packet() {
            Some(p) => p,
            None => {
                warn!("No Kafka packet found in RobustMQPacket");
                return None;
            }
        };
        let kafka_response = match packet {
            KafkaPacket::ApiVersionReq(_) => match Self::handle_api_versions() {
                Ok(resp) => resp,
                Err(e) => {
                    warn!("Failed to build ApiVersionsResponse: {:?}", e);
                    return None;
                }
            },
            KafkaPacket::ProduceReq(req) => match self.handle_produce(req).await {
                Ok(resp) => resp,
                Err(e) => {
                    warn!("Produce handler failed: {:?}", e);
                    return None;
                }
            },
            KafkaPacket::FetchReq(_) => {
                warn!("Received Fetch request but Fetch handling is not implemented");
                return None;
            }
            other => {
                warn!("Unsupported or unhandled Kafka packet: {:?}", other);
                return None;
            }
        };

        Some(ResponsePackage::build(
            tcp_connection.connection_id,
            RobustMQPacket::KAFKA(kafka_response),
        ))
    }
}

pub fn create_command(message_storage_adapter: ArcStorageAdapter) -> Arc<Box<dyn Command + Send + Sync>> {
    let storage: Box<dyn Command + Send + Sync> = Box::new(KafkaCommand::new(message_storage_adapter));
    Arc::new(storage)
}
