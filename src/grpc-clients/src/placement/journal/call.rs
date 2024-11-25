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

use common_base::error::common::CommonError;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    ListShardReply, ListShardRequest, UpdateSegmentMetaReply, UpdateSegmentMetaRequest,
    UpdateSegmentStatusReply, UpdateSegmentStatusRequest,
};

use super::{JournalServiceReply, JournalServiceRequest};
use crate::placement::{retry_placement_center_call, PlacementCenterReply, PlacementCenterRequest};
use crate::pool::ClientPool;

macro_rules! generate_journal_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty, $variant:ident) => {
        pub async fn $fn_name(
            client_pool: Arc<ClientPool>,
            addrs: &[String],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            let request = PlacementCenterRequest::Journal(JournalServiceRequest::$variant(request));
            match retry_placement_center_call(&client_pool, addrs, request).await? {
                PlacementCenterReply::Journal(JournalServiceReply::$variant(reply)) => Ok(reply),
                _ => unreachable!("Reply type mismatch"),
            }
        }
    };
}

generate_journal_service_call!(list_shard, ListShardRequest, ListShardReply, ListShard);
generate_journal_service_call!(
    create_shard,
    CreateShardRequest,
    CreateShardReply,
    CreateShard
);
generate_journal_service_call!(
    delete_shard,
    DeleteShardRequest,
    DeleteShardReply,
    DeleteShard
);
generate_journal_service_call!(
    list_segment,
    ListSegmentRequest,
    ListSegmentReply,
    ListSegment
);
generate_journal_service_call!(
    create_next_segment,
    CreateNextSegmentRequest,
    CreateNextSegmentReply,
    CreateSegment
);
generate_journal_service_call!(
    delete_segment,
    DeleteSegmentRequest,
    DeleteSegmentReply,
    DeleteSegment
);
generate_journal_service_call!(
    update_segment_status,
    UpdateSegmentStatusRequest,
    UpdateSegmentStatusReply,
    UpdateSegmentStatus
);
generate_journal_service_call!(
    list_segment_meta,
    ListSegmentMetaRequest,
    ListSegmentMetaReply,
    ListSegmentMeta
);
generate_journal_service_call!(
    update_segment_meta,
    UpdateSegmentMetaRequest,
    UpdateSegmentMetaReply,
    UpdateSegmentMeta
);
