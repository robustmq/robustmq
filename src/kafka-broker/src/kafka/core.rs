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

use kafka_protocol::messages::list_offsets_response::{
    ListOffsetsPartitionResponse, ListOffsetsResponse, ListOffsetsTopicResponse,
};
use kafka_protocol::messages::{FetchRequest, ListOffsetsRequest, MetadataRequest, ProduceRequest};
use protocol::kafka::packet::KafkaPacket;

pub fn process_produce(_req: &ProduceRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_fetch(_req: &FetchRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_list_offsets(req: &ListOffsetsRequest) -> Option<KafkaPacket> {
    // Return offset=0 for earliest (-2) and the end for latest (-1).
    // We always report offset 0 as both earliest and latest to keep it simple.
    let topics = req
        .topics
        .iter()
        .map(|t| {
            let partitions = t
                .partitions
                .iter()
                .map(|p| {
                    ListOffsetsPartitionResponse::default()
                        .with_partition_index(p.partition_index)
                        .with_error_code(0)
                        .with_offset(0)
                })
                .collect();
            ListOffsetsTopicResponse::default()
                .with_name(t.name.clone())
                .with_partitions(partitions)
        })
        .collect();

    Some(KafkaPacket::ListOffsetsResponse(
        ListOffsetsResponse::default().with_topics(topics),
    ))
}

pub fn process_metadata(_req: &MetadataRequest) -> Option<KafkaPacket> {
    None
}
