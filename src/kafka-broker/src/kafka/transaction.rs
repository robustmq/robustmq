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

use kafka_protocol::messages::{
    AddOffsetsToTxnRequest, AddPartitionsToTxnRequest, DescribeTransactionsRequest, EndTxnRequest,
    InitProducerIdRequest, ListTransactionsRequest, TxnOffsetCommitRequest,
};
use protocol::kafka::packet::KafkaPacket;

pub fn process_init_producer_id(_req: &InitProducerIdRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_add_partitions_to_txn(_req: &AddPartitionsToTxnRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_add_offsets_to_txn(_req: &AddOffsetsToTxnRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_end_txn(_req: &EndTxnRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_txn_offset_commit(_req: &TxnOffsetCommitRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_describe_transactions(_req: &DescribeTransactionsRequest) -> Option<KafkaPacket> {
    None
}

pub fn process_list_transactions(_req: &ListTransactionsRequest) -> Option<KafkaPacket> {
    None
}
