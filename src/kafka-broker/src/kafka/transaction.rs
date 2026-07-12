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

use kafka_protocol::error::ResponseError;
use kafka_protocol::messages::{
    AddOffsetsToTxnRequest, AddPartitionsToTxnRequest, DescribeTransactionsRequest, EndTxnRequest,
    InitProducerIdRequest, InitProducerIdResponse, ListTransactionsRequest, ProducerId,
    TxnOffsetCommitRequest,
};
use protocol::kafka::packet::KafkaPacket;

use crate::core::cache::KafkaCacheManager;

/// Allocate a producer id for an **idempotent** producer (KIP-98 idempotence
/// only — transactions are not implemented). A request carrying a
/// `transactional_id` is rejected, since honoring it would promise
/// transactional semantics the broker can't provide. A pure idempotent producer
/// (null transactional id) gets a fresh producer id at epoch 0.
pub fn process_init_producer_id(
    cache: &Arc<KafkaCacheManager>,
    req: &InitProducerIdRequest,
) -> Option<KafkaPacket> {
    if req.transactional_id.is_some() {
        return Some(KafkaPacket::InitProducerIdResponse(
            InitProducerIdResponse::default()
                .with_error_code(ResponseError::TransactionalIdAuthorizationFailed.code())
                .with_producer_id(ProducerId(-1))
                .with_producer_epoch(-1),
        ));
    }

    Some(KafkaPacket::InitProducerIdResponse(
        InitProducerIdResponse::default()
            .with_error_code(0)
            .with_producer_id(ProducerId(cache.next_producer_id()))
            .with_producer_epoch(0),
    ))
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
