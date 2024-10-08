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
//     kv::{DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetRequest},
// };

// use super::KvServiceManager;

// pub(crate) async fn inner_get(
//     mut client: Connection<KvServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match GetRequest::decode(request.as_ref()) {
//         Ok(request) => match client.get(request).await {
//             Ok(result) => {
//                 return Ok(GetReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_set(
//     mut client: Connection<KvServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match SetRequest::decode(request.as_ref()) {
//         Ok(request) => match client.set(request).await {
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

// pub(crate) async fn inner_delete(
//     mut client: Connection<KvServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match DeleteRequest::decode(request.as_ref()) {
//         Ok(request) => match client.delete(request).await {
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

// pub(crate) async fn inner_exists(
//     mut client: Connection<KvServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match ExistsRequest::decode(request.as_ref()) {
//         Ok(request) => match client.exists(request).await {
//             Ok(result) => {
//                 return Ok(ExistsReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }
