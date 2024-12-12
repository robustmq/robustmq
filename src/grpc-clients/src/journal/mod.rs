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

use common_base::error::common::CommonError;
use protocol::journal_server::journal_admin::journal_server_admin_service_client::JournalServerAdminServiceClient;
use protocol::journal_server::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use protocol::journal_server::journal_inner::journal_server_inner_service_client::JournalServerInnerServiceClient;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};
use tonic::transport::Channel;

use crate::pool::ClientPool;
use crate::utils::RetriableRequest;

pub mod admin;
pub mod inner;

impl_retriable_request!(
    UpdateJournalCacheRequest,
    JournalServerInnerServiceClient<Channel>,
    UpdateJournalCacheReply,
    journal_inner_services_client,
    update_cache
);

impl_retriable_request!(
    DeleteShardFileRequest,
    JournalServerInnerServiceClient<Channel>,
    DeleteShardFileReply,
    journal_inner_services_client,
    delete_shard_file
);

impl_retriable_request!(
    GetShardDeleteStatusRequest,
    JournalServerInnerServiceClient<Channel>,
    GetShardDeleteStatusReply,
    journal_inner_services_client,
    get_shard_delete_status
);

impl_retriable_request!(
    DeleteSegmentFileRequest,
    JournalServerInnerServiceClient<Channel>,
    DeleteSegmentFileReply,
    journal_inner_services_client,
    delete_segment_file
);

impl_retriable_request!(
    GetSegmentDeleteStatusRequest,
    JournalServerInnerServiceClient<Channel>,
    GetSegmentDeleteStatusReply,
    journal_inner_services_client,
    get_segment_delete_status
);

impl_retriable_request!(
    ListShardRequest,
    JournalServerAdminServiceClient<Channel>,
    ListShardReply,
    journal_admin_services_client,
    list_shard
);

impl_retriable_request!(
    ListSegmentRequest,
    JournalServerAdminServiceClient<Channel>,
    ListSegmentReply,
    journal_admin_services_client,
    list_segment
);
