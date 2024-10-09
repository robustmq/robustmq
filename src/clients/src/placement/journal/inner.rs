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

// use common_base::error::common::CommonError;
// use mobc::Connection;
// use prost::Message;
// use protocol::placement_center::generate::{
//     common::CommonReply,
//     journal::{CreateSegmentRequest, CreateShardRequest, DeleteSegmentRequest, DeleteShardRequest},
// };

// use super::JournalServiceManager;

// pub(crate) async fn inner_create_shard(
//     mut client: Connection<JournalServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match CreateShardRequest::decode(request.as_ref()) {
//         Ok(request) => match client.create_shard(request).await {
//             Ok(result) => {
//                 return Ok(CommonReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_delete_shard(
//     mut client: Connection<JournalServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match DeleteShardRequest::decode(request.as_ref()) {
//         Ok(request) => match client.delete_shard(request).await {
//             Ok(result) => {
//                 return Ok(CommonReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_create_segment(
//     mut client: Connection<JournalServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match CreateSegmentRequest::decode(request.as_ref()) {
//         Ok(request) => match client.create_segment(request).await {
//             Ok(result) => {
//                 return Ok(CommonReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_delete_segment(
//     mut client: Connection<JournalServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match DeleteSegmentRequest::decode(request.as_ref()) {
//         Ok(request) => match client.delete_segment(request).await {
//             Ok(result) => {
//                 return Ok(CommonReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }
