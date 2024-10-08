// // Copyright 2023 RobustMQ Team
// //
// // Licensed under the Apache License, Version 2.0 (the "License");
// // you may not use this file except in compliance with the License.
// // You may obtain a copy of the License at
// //
// //     http://www.apache.org/licenses/LICENSE-2.0
// //
// // Unless required by applicable law or agreed to in writing, software
// // distributed under the License is distributed on an "AS IS" BASIS,
// // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// // See the License for the specific language governing permissions and
// // limitations under the License.

// use common_base::error::common::CommonError;
// use mobc::Connection;
// use prost::Message;
// use protocol::placement_center::generate::{
//     common::{CommonReply},
//     placement::{
//         ClusterStatusReply, ClusterStatusRequest, DeleteIdempotentDataRequest, DeleteResourceConfigRequest, ExistsIdempotentDataReply, ExistsIdempotentDataRequest, GetResourceConfigReply, GetResourceConfigRequest, HeartbeatRequest, NodeListReply, NodeListRequest, RegisterNodeRequest, SendRaftConfChangeReply, SendRaftConfChangeRequest, SendRaftMessageReply, SendRaftMessageRequest, SetIdempotentDataRequest, SetResourceConfigRequest, UnRegisterNodeRequest
//     },
// };

// use super::PlacementServiceManager;

// pub(crate) async fn inner_cluster_status(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match ClusterStatusRequest::decode(request.as_ref()) {
//         Ok(request) => match client.cluster_status(request).await {
//             Ok(result) => {
//                 return Ok(ClusterStatusReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_node_list(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match NodeListRequest::decode(request.as_ref()) {
//         Ok(request) => match client.node_list(request).await {
//             Ok(result) => {
//                 return Ok(NodeListReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_register_node(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match RegisterNodeRequest::decode(request.as_ref()) {
//         Ok(request) => match client.register_node(request).await {
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

// pub(crate) async fn inner_unregister_node(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match UnRegisterNodeRequest::decode(request.as_ref()) {
//         Ok(request) => match client.un_register_node(request).await {
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

// pub(crate) async fn inner_heartbeat(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match HeartbeatRequest::decode(request.as_ref()) {
//         Ok(request) => match client.heartbeat(request).await {
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

// pub(crate) async fn inner_send_raft_message(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match SendRaftMessageRequest::decode(request.as_ref()) {
//         Ok(request) => match client.send_raft_message(request).await {
//             Ok(result) => {
//                 return Ok(SendRaftMessageReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_send_raft_conf_change(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match SendRaftConfChangeRequest::decode(request.as_ref()) {
//         Ok(request) => match client.send_raft_conf_change(request).await {
//             Ok(result) => {
//                 return Ok(SendRaftConfChangeReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_set_resource_config(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match SetResourceConfigRequest::decode(request.as_ref()) {
//         Ok(request) => match client.set_resource_config(request).await {
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

// pub(crate) async fn inner_get_resource_config(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match GetResourceConfigRequest::decode(request.as_ref()) {
//         Ok(request) => match client.get_resource_config(request).await {
//             Ok(result) => {
//                 return Ok(GetResourceConfigReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_delete_resource_config(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match DeleteResourceConfigRequest::decode(request.as_ref()) {
//         Ok(request) => match client.delete_resource_config(request).await {
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

// pub(crate) async fn inner_set_idempotent(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match SetIdempotentDataRequest::decode(request.as_ref()) {
//         Ok(request) => match client.set_idempotent_data(request).await {
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

// pub(crate) async fn inner_exist_idempotent(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match ExistsIdempotentDataRequest::decode(request.as_ref()) {
//         Ok(request) => match client.exists_idempotent_data(request).await {
//             Ok(result) => {
//                 return Ok(ExistsIdempotentDataReply::encode_to_vec(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => {
//             return Err(CommonError::CommmonError(e.to_string()));
//         }
//     }
// }

// pub(crate) async fn inner_delete_idempotent(
//     mut client: Connection<PlacementServiceManager>,
//     request: Vec<u8>,
// ) -> Result<Vec<u8>, CommonError> {
//     match DeleteIdempotentDataRequest::decode(request.as_ref()) {
//         Ok(request) => match client.delete_idempotent_data(request).await {
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