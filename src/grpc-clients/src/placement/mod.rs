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

use std::borrow::Cow;
use std::collections::HashSet;
use std::time::Duration;

use common_base::error::common::CommonError;
use inner::{PlacementServiceReply, PlacementServiceRequest};
use journal::{JournalServiceReply, JournalServiceRequest};
use kv::{KvServiceReply, KvServiceRequest};
use lazy_static::lazy_static;
use log::debug;
use mqtt::{MqttServiceReply, MqttServiceRequest};
use tokio::time::sleep;

use self::openraft::{OpenRaftServiceReply, OpenRaftServiceRequest};
use crate::pool::ClientPool;
use crate::{retry_sleep_time, retry_times};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlacementCenterInterface {
    // kv interface
    Set,
    Get,
    Delete,
    Exists,

    // placement inner interface
    ClusterStatus,
    ListNode,
    RegisterNode,
    UnRegisterNode,
    Heartbeat,
    SendRaftMessage,
    SendRaftConfChange,

    // journal service interface
    ListShard,
    CreateShard,
    DeleteShard,
    ListSegment,
    CreateSegment,
    DeleteSegment,
    UpdateSegmentStatus,
    ListSegmentMeta,
    UpdateSegmentMeta,

    // mqtt service interface
    GetShareSubLeader,
    CreateUser,
    DeleteUser,
    ListUser,
    CreateTopic,
    DeleteTopic,
    ListTopic,
    SetTopicRetainMessage,
    SetNXExclusiveTopic,
    DeleteExclusiveTopic,
    CreateSession,
    DeleteSession,
    ListSession,
    UpdateSession,
    SaveLastWillMessage,
    SetResourceConfig,
    GetResourceConfig,
    DeleteResourceConfig,
    SetIdempotentData,
    ExistsIdempotentData,
    DeleteIdempotentData,
    CreateAcl,
    DeleteAcl,
    ListAcl,
    CreateBlackList,
    DeleteBlackList,
    ListBlackList,

    // Open Raft
    Vote,
    Append,
    Snapshot,
    AddLearner,
    ChangeMembership,
}

/// Enum wrapper for all possible requests to the placement center
#[derive(Debug, Clone)]
pub enum PlacementCenterRequest {
    Kv(KvServiceRequest),
    Placement(PlacementServiceRequest),
    Journal(JournalServiceRequest),
    Mqtt(MqttServiceRequest),
    OpenRaft(OpenRaftServiceRequest),
}

/// Enum wrapper for all possible replies from the placement center
#[derive(Debug, Clone)]
pub enum PlacementCenterReply {
    Kv(KvServiceReply),
    Placement(PlacementServiceReply),
    Journal(JournalServiceReply),
    Mqtt(MqttServiceReply),
    OpenRaft(OpenRaftServiceReply),
}

impl PlacementCenterInterface {
    pub fn should_forward_to_leader(&self) -> bool {
        lazy_static! {
            static ref FORWARD_SET: HashSet<PlacementCenterInterface> = {
                let mut set = HashSet::new();
                // mqtt service interface
                set.insert(PlacementCenterInterface::CreateUser);
                set.insert(PlacementCenterInterface::DeleteUser);
                set.insert(PlacementCenterInterface::CreateTopic);
                set.insert(PlacementCenterInterface::DeleteTopic);
                set.insert(PlacementCenterInterface::CreateSession);
                set.insert(PlacementCenterInterface::DeleteSession);
                set.insert(PlacementCenterInterface::UpdateSession);
                set.insert(PlacementCenterInterface::DeleteSession);
                set.insert(PlacementCenterInterface::CreateAcl);
                set.insert(PlacementCenterInterface::DeleteAcl);
                set.insert(PlacementCenterInterface::CreateBlackList);
                set.insert(PlacementCenterInterface::DeleteBlackList);

                // placement inner interface
                set.insert(PlacementCenterInterface::RegisterNode);
                set.insert(PlacementCenterInterface::UnRegisterNode);
                set.insert(PlacementCenterInterface::Heartbeat);
                set.insert(PlacementCenterInterface::SendRaftMessage);
                set.insert(PlacementCenterInterface::SendRaftConfChange);
                set.insert(PlacementCenterInterface::SetResourceConfig);
                set.insert(PlacementCenterInterface::DeleteResourceConfig);
                set.insert(PlacementCenterInterface::SetIdempotentData);
                set.insert(PlacementCenterInterface::DeleteIdempotentData);
                set
            };
        }
        FORWARD_SET.contains(self)
    }
}

#[allow(clippy::module_inception)]
pub mod inner;
pub mod journal;
pub mod kv;
pub mod mqtt;
pub mod openraft;

fn is_write_request(_req: &PlacementCenterRequest) -> bool {
    true
}

// NOTE: This is mostly similar to the `retry_call` function in the `utils.rs` file.
// However, it's hard to work around the lifetime issue if we were to return the leader addr
// as well. So we ended up duplicating the function here.
async fn retry_placement_center_call<'a>(
    client_pool: &'a ClientPool,
    addrs: &'a [String],
    request: PlacementCenterRequest,
) -> Result<PlacementCenterReply, CommonError> {
    if addrs.is_empty() {
        return Err(CommonError::CommonError(
            "Call address list cannot be empty".to_string(),
        ));
    }

    let mut times = 1;
    loop {
        let index = times % addrs.len();
        // let addr = &addrs[index];
        let mut addr = Cow::Borrowed(&addrs[index]);
        if is_write_request(&request) {
            if let Some(leader_addr) = client_pool.get_leader_addr(&addr) {
                addr = Cow::Owned(leader_addr.value().clone());
            }
        }

        let result = call_once(client_pool, &addr, request.clone()).await;

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                debug!("{}", e);
                if times > retry_times() {
                    return Err(e);
                }
                times += 1;
            }
        }

        sleep(Duration::from_secs(retry_sleep_time(times))).await;
    }
}

async fn call_once(
    client_pool: &ClientPool,
    addr: &str,
    request: PlacementCenterRequest,
) -> Result<PlacementCenterReply, CommonError> {
    match request {
        PlacementCenterRequest::Kv(request) => {
            let reply = kv::call_kv_service_once(client_pool, addr, request).await?;
            Ok(PlacementCenterReply::Kv(reply))
        }
        PlacementCenterRequest::Placement(request) => {
            let reply = inner::call_placement_service_once(client_pool, addr, request).await?;
            Ok(PlacementCenterReply::Placement(reply))
        }
        PlacementCenterRequest::Journal(request) => {
            let reply = journal::call_journal_service_once(client_pool, addr, request).await?;
            Ok(PlacementCenterReply::Journal(reply))
        }
        PlacementCenterRequest::Mqtt(request) => {
            let reply = mqtt::call_mqtt_service_once(client_pool, addr, request).await?;
            Ok(PlacementCenterReply::Mqtt(reply))
        }
        PlacementCenterRequest::OpenRaft(request) => {
            let reply = openraft::call_open_raft_service_once(client_pool, addr, request).await?;
            Ok(PlacementCenterReply::OpenRaft(reply))
        }
    }
}

#[cfg(test)]
mod test {
    use common_base::error::common::CommonError;
    use regex::Regex;

    // FIXME: The two functions below don't seem to be used anywhere in the codebase right now.

    ///Determines if the given error indicates that a request needs to be forwarded.
    ///
    /// Parameters:
    /// - `err: &CommonError`: error information
    ///
    /// Returns:
    /// - `Bool`: Returns 'true' if the error information indicates the request needs to be forwarded, 'false' otherwise.
    fn is_has_to_forward(err: &CommonError) -> bool {
        let error_info = err.to_string();

        error_info.contains("has to forward request to")
    }

    /// Extracts the forward address from given error information.
    ///
    /// Parameters:
    /// - `err: &CommonError`: error information
    ///
    /// Returns:
    /// - `String`: Returns the forward address if found,'None' if no address is found
    pub(crate) fn get_forward_addr(err: &CommonError) -> Option<String> {
        let error_info = err.to_string();
        let re = Regex::new(r"rpc_addr: ([^}]+)").unwrap();
        if let Some(caps) = re.captures(&error_info) {
            if let Some(rpc_addr) = caps.get(1) {
                let mut leader_addr = rpc_addr.as_str().to_string();
                leader_addr = leader_addr.replace("\\", "");
                leader_addr = leader_addr.replace("\"", "");
                leader_addr = leader_addr.replace(" ", "");
                return Some(leader_addr);
            }
        }

        None
    }

    #[tokio::test]
    pub async fn is_has_to_forward_test() {
        let err_info = r#"
        Grpc call of the node failed,Grpc status was status: Cancelled, message: "has to forward request to: Some(2), Some(Node { node_id: 2, rpc_addr: \"127.0.0.1:2228\" })", details: [], metadata: MetadataMap { headers: {"content-type": "application/grpc", "date": "Sun, 06 Oct 2024 10:25:36 GMT", "content-length": "0"} }
        "#;
        let err = CommonError::CommonError(err_info.to_string());
        assert!(is_has_to_forward(&err));

        let other_err_info = "Other error";
        let other_err = CommonError::CommonError(other_err_info.to_string());
        assert!(!is_has_to_forward(&other_err));
    }

    #[tokio::test]
    pub async fn get_forward_addr_test() {
        let err = r#"
        Grpc call of the node failed,Grpc status was status: Cancelled, message: "has to forward request to: Some(2), Some(Node { node_id: 2, rpc_addr: \"127.0.0.1:2228\" })", details: [], metadata: MetadataMap { headers: {"content-type": "application/grpc", "date": "Sun, 06 Oct 2024 10:25:36 GMT", "content-length": "0"} }
        "#;
        let err = CommonError::CommonError(err.to_string());
        let res = get_forward_addr(&err);
        assert_eq!("127.0.0.1:2228".to_string(), res.unwrap());
    }
}
