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

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use common_base::error::common::CommonError;
use lazy_static::lazy_static;
use log::error;
use openraft::openraft_interface_call;
use regex::Regex;
use tokio::time::sleep;

use self::journal::journal_interface_call;
use self::kv::kv_interface_call;
use self::mqtt::mqtt_interface_call;
use self::placement::placement_interface_call;
use crate::poll::ClientPool;
use crate::{retry_sleep_time, retry_times};

#[derive(Clone, Debug)]
pub enum PlacementCenterService {
    Journal,
    Kv,
    Placement,
    Mqtt,
    OpenRaft,
}

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
    CreateShard,
    DeleteShard,
    CreateSegment,
    DeleteSegment,

    // mqtt service interface
    GetShareSubLeader,
    CreateUser,
    DeleteUser,
    ListUser,
    CreateTopic,
    DeleteTopic,
    ListTopic,
    SetTopicRetainMessage,
    CreateSession,
    DeleteSession,
    ListSession,
    UpdateSession,
    SaveLastWillMessage,
    SetReourceConfig,
    GetReourceConfig,
    DeleteReourceConfig,
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
                set.insert(PlacementCenterInterface::SetReourceConfig);
                set.insert(PlacementCenterInterface::DeleteReourceConfig);
                set.insert(PlacementCenterInterface::SetIdempotentData);
                set.insert(PlacementCenterInterface::DeleteIdempotentData);
                set
            };
        }
        FORWARD_SET.contains(self)
    }
}

pub mod journal;
pub mod kv;
pub mod mqtt;
pub mod openraft;
#[allow(clippy::module_inception)]
pub mod placement;

async fn retry_call(
    service: PlacementCenterService,
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    let mut times = 1;
    loop {
        let (addr, new_times) = calc_addr(&client_poll, &addrs, times, &service, &interface);
        times = new_times;

        let result = match service {
            PlacementCenterService::Journal => {
                journal_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Kv => {
                kv_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Placement => {
                placement_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }

            PlacementCenterService::Mqtt => {
                mqtt_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }
            PlacementCenterService::OpenRaft => {
                openraft_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr.clone(),
                    request.clone(),
                )
                .await
            }
        };

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                if is_has_to_forward(&e) {
                    if let Some(leader_addr) = get_forward_addr(&e) {
                        client_poll.set_leader_addr(&addr, &leader_addr);
                    }
                } else {
                    error!(
                        "{:?}@{:?}@{},{},",
                        service.clone(),
                        interface.clone(),
                        addr.clone(),
                        e
                    );
                    sleep(Duration::from_secs(retry_sleep_time(times))).await;
                }
                if times > retry_times() {
                    return Err(e);
                }
            }
        }
    }
}

fn calc_addr(
    client_poll: &Arc<ClientPool>,
    addrs: &[String],
    times: usize,
    service: &PlacementCenterService,
    interface: &PlacementCenterInterface,
) -> (String, usize) {
    let index = times % addrs.len();
    let addr = addrs.get(index).unwrap().clone();
    if is_write_request(service, interface) {
        if let Some(leader_addr) = client_poll.get_leader_addr(&addr) {
            return (leader_addr, times + 1);
        }
    }
    (addr, times + 1)
}

fn is_write_request(
    _service: &PlacementCenterService,
    _interface: &PlacementCenterInterface,
) -> bool {
    true
}

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

#[cfg(test)]
mod test {
    use common_base::error::common::CommonError;

    use crate::placement::{get_forward_addr, is_has_to_forward};

    #[tokio::test]
    pub async fn is_has_to_forward_test() {
        let err_info = r#"
        Grpc call of the node failed,Grpc status was status: Cancelled, message: "has to forward request to: Some(2), Some(Node { node_id: 2, rpc_addr: \"127.0.0.1:2228\" })", details: [], metadata: MetadataMap { headers: {"content-type": "application/grpc", "date": "Sun, 06 Oct 2024 10:25:36 GMT", "content-length": "0"} }
        "#;
        let err = CommonError::CommmonError(err_info.to_string());
        assert!(is_has_to_forward(&err));

        let other_err_info = "Other error";
        let other_err = CommonError::CommmonError(other_err_info.to_string());
        assert!(!is_has_to_forward(&other_err));
    }

    #[tokio::test]
    pub async fn get_forward_addr_test() {
        let err = r#"
        Grpc call of the node failed,Grpc status was status: Cancelled, message: "has to forward request to: Some(2), Some(Node { node_id: 2, rpc_addr: \"127.0.0.1:2228\" })", details: [], metadata: MetadataMap { headers: {"content-type": "application/grpc", "date": "Sun, 06 Oct 2024 10:25:36 GMT", "content-length": "0"} }
        "#;
        let err = CommonError::CommmonError(err.to_string());
        let res = get_forward_addr(&err);
        assert_eq!("127.0.0.1:2228".to_string(), res.unwrap());
    }
}
