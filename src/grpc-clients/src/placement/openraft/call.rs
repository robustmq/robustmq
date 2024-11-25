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
use protocol::placement_center::placement_center_openraft::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};

use crate::placement::openraft::{OpenRaftServiceReply, OpenRaftServiceRequest};
use crate::placement::{retry_placement_center_call, PlacementCenterReply, PlacementCenterRequest};
use crate::pool::ClientPool;

macro_rules! generate_openraft_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty, $variant:ident) => {
        pub async fn $fn_name(
            client_pool: Arc<ClientPool>,
            addrs: &[String],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            let request =
                PlacementCenterRequest::OpenRaft(OpenRaftServiceRequest::$variant(request));
            match retry_placement_center_call(&client_pool, addrs, request).await? {
                PlacementCenterReply::OpenRaft(OpenRaftServiceReply::$variant(reply)) => Ok(reply),
                _ => unreachable!("Reply type mismatch"),
            }
        }
    };
}

generate_openraft_service_call!(placement_openraft_vote, VoteRequest, VoteReply, Vote);
generate_openraft_service_call!(
    placement_openraft_append,
    AppendRequest,
    AppendReply,
    Append
);
generate_openraft_service_call!(
    placement_openraft_snapshot,
    SnapshotRequest,
    SnapshotReply,
    Snapshot
);
generate_openraft_service_call!(
    placement_openraft_add_learner,
    AddLearnerRequest,
    AddLearnerReply,
    AddLearner
);
generate_openraft_service_call!(
    placement_openraft_change_membership,
    ChangeMembershipRequest,
    ChangeMembershipReply,
    ChangeMembership
);
