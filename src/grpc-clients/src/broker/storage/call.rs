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

use crate::pool::ClientPool;
use common_base::error::common::CommonError;
use protocol::broker::broker_storage::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};

macro_rules! generate_journal_inner_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty, $variant:ident) => {
        pub async fn $fn_name(
            client_pool: &ClientPool,
            addrs: &[impl AsRef<str>],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            $crate::utils::retry_call(client_pool, addrs, request).await
        }
    };
}

generate_journal_inner_service_call!(
    journal_inner_update_cache,
    UpdateJournalCacheRequest,
    UpdateJournalCacheReply,
    UpdateJournalCache
);

generate_journal_inner_service_call!(
    journal_inner_delete_shard_file,
    DeleteShardFileRequest,
    DeleteShardFileReply,
    DeleteShardFile
);

generate_journal_inner_service_call!(
    journal_inner_get_shard_delete_status,
    GetShardDeleteStatusRequest,
    GetShardDeleteStatusReply,
    GetShardDeleteStatus
);

generate_journal_inner_service_call!(
    journal_inner_delete_segment_file,
    DeleteSegmentFileRequest,
    DeleteSegmentFileReply,
    DeleteSegmentFile
);

generate_journal_inner_service_call!(
    journal_inner_get_segment_delete_status,
    GetSegmentDeleteStatusRequest,
    GetSegmentDeleteStatusReply,
    GetSegmentDeleteStatus
);
