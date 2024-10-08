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

// use super::OpenRaftServiceManager;
// use common_base::error::common::CommonError;
// use mobc::Connection;
// use prost::Message;
// use protocol::placement_center::generate::openraft::{
//     AppendReply, AppendRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
// };

// pub(crate) async fn inner_vote(
//     mut client: Connection<OpenRaftServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match VoteRequest::decode(request.as_ref()) {
//         Ok(request) => match client.vote(request).await {
//             Ok(result) => {
//                 return Ok(VoteReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_append(
//     mut client: Connection<OpenRaftServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match AppendRequest::decode(request.as_ref()) {
//         Ok(request) => match client.append(request).await {
//             Ok(result) => {
//                 return Ok(AppendReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_snapshot(
//     mut client: Connection<OpenRaftServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match SnapshotRequest::decode(request.as_ref()) {
//         Ok(request) => match client.snapshot(request).await {
//             Ok(result) => {
//                 return Ok(SnapshotReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }
