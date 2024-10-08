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

// use super::MQTTServiceManager;

// pub(crate) async fn inner_call<R, Resp, ClientFunction, Fut, DecodeFunction, EncodeFunction>(
//     client: Connection<MQTTServiceManager>,
//     request: Vec<u8>,
//     decode_fn: DecodeFunction,
//     client_fn: ClientFunction,
//     encode_fn: EncodeFunction,
// ) -> Result<Vec<u8>, CommonError>
// where
//     R: prost::Message + Default,
//     Resp: prost::Message,
//     DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
//     ClientFunction: FnOnce(Connection<MQTTServiceManager>, R) -> Fut,
//     Fut: std::future::Future<Output = Result<tonic::Response<Resp>, tonic::Status>>,
//     EncodeFunction: FnOnce(&Resp) -> Vec<u8>,
// {
//     match decode_fn(request.as_ref()) {
//         Ok(decoded_request) => match client_fn(client, decoded_request).await {
//             Ok(result) => {
//                 return Ok(encode_fn(&result.into_inner()));
//             }
//             Err(e) => return Err(CommonError::GrpcServerStatus(e)),
//         },
//         Err(e) => return Err(CommonError::CommmonError(e.to_string())),
//     }
// }
