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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MetaServiceInterface {
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

pub mod common;
pub mod journal;
pub mod mqtt;

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
