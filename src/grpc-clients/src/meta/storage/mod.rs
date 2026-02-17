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

use protocol::meta::meta_service_journal::engine_service_client::EngineServiceClient;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, SealUpSegmentReply, SealUpSegmentRequest,
    UpdateStartTimeBySegmentMetaReply, UpdateStartTimeBySegmentMetaRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    ListShardRequest,
    EngineServiceClient<Channel>,
    ListShardReply,
    list_shard,
    "EngineService",
    "ListShard",
    true
);

impl_retriable_request!(
    CreateShardRequest,
    EngineServiceClient<Channel>,
    CreateShardReply,
    create_shard,
    "EngineService",
    "CreateShard",
    true
);

impl_retriable_request!(
    DeleteShardRequest,
    EngineServiceClient<Channel>,
    DeleteShardReply,
    delete_shard,
    "EngineService",
    "DeleteShard",
    true
);

impl_retriable_request!(
    ListSegmentRequest,
    EngineServiceClient<Channel>,
    ListSegmentReply,
    list_segment,
    "EngineService",
    "ListSegment",
    true
);

impl_retriable_request!(
    CreateNextSegmentRequest,
    EngineServiceClient<Channel>,
    CreateNextSegmentReply,
    create_next_segment,
    "EngineService",
    "CreateNextSegment",
    true
);

impl_retriable_request!(
    DeleteSegmentRequest,
    EngineServiceClient<Channel>,
    DeleteSegmentReply,
    delete_segment,
    "EngineService",
    "DeleteSegment",
    true
);

impl_retriable_request!(
    SealUpSegmentRequest,
    EngineServiceClient<Channel>,
    SealUpSegmentReply,
    seal_up_segment,
    "EngineService",
    "SealUpSegmentRequest",
    true
);

impl_retriable_request!(
    ListSegmentMetaRequest,
    EngineServiceClient<Channel>,
    ListSegmentMetaReply,
    list_segment_meta,
    "EngineService",
    "ListSegmentMeta",
    true
);

impl_retriable_request!(
    UpdateStartTimeBySegmentMetaRequest,
    EngineServiceClient<Channel>,
    UpdateStartTimeBySegmentMetaReply,
    update_start_time_by_segment_meta,
    "EngineService",
    "UpdateStartTimeBySegmentMeta",
    true
);
