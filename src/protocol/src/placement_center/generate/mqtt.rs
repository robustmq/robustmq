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

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShareSubLeaderRequest {
    /// The name of the group.
    #[prost(string, tag = "1")]
    pub group_name: ::prost::alloc::string::String,
    /// The name of the cluster.
    #[prost(string, tag = "3")]
    pub cluster_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShareSubLeaderReply {
    /// The id of the broker.
    #[prost(uint64, tag = "1")]
    pub broker_id: u64,
    /// The ip address of the broker
    #[prost(string, tag = "2")]
    pub broker_addr: ::prost::alloc::string::String,
    /// The parameter for the extended information of the broker node.
    #[prost(string, tag = "3")]
    pub extend_info: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUserRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the user.
    #[prost(string, tag = "2")]
    pub user_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListUserReply {
    /// The parameter contains a list of users, encoded from a `Vec<MQTTTopic>` into a binary format.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub users: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the user.
    #[prost(string, tag = "2")]
    pub user_name: ::prost::alloc::string::String,
    /// The parameter contains user information, encoded from a `MQTTUser` object into a binary format.
    #[prost(bytes = "vec", tag = "3")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the user.
    #[prost(string, tag = "2")]
    pub user_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTopicRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the topic.
    #[prost(string, tag = "2")]
    pub topic_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListTopicReply {
    /// The parameter contains a list of topics, encoded from a `Vec<MQTTTopic>` into a binary format.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub topics: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTopicRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the topic.
    #[prost(string, tag = "2")]
    pub topic_name: ::prost::alloc::string::String,
    /// The parameter contains topic information, encoded from a `MQTTTopic` object into a binary format.
    #[prost(bytes = "vec", tag = "3")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTopicRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the topic.
    #[prost(string, tag = "2")]
    pub topic_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetTopicRetainMessageRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The name of the topic.
    #[prost(string, tag = "2")]
    pub topic_name: ::prost::alloc::string::String,
    /// The parameter contains retain message, encoded from a `MQTTMessage` object into a binary format.
    #[prost(bytes = "vec", tag = "3")]
    pub retain_message: ::prost::alloc::vec::Vec<u8>,
    /// The parameter is the expiration time of the retain message. The unit is seconds.
    #[prost(uint64, tag = "4")]
    pub retain_message_expired_at: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSessionRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The id of the client.
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSessionReply {
    /// The parameter contains a list of sessions, encoded from a `Vec<MQTTSession>` into a binary format.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub sessions: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSessionRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The id of the client.
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// The parameter contains session information, encoded from a `MQTTSession` object into a binary format.
    #[prost(bytes = "vec", tag = "3")]
    pub session: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSessionRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The id of the client.
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// The id of the connection.
    #[prost(uint64, tag = "3")]
    pub connection_id: u64,
    /// The id of the broker.
    #[prost(uint64, tag = "4")]
    pub broker_id: u64,
    /// The parameter is the time when session reconnects. The unit is seconds.
    #[prost(uint64, tag = "5")]
    pub reconnect_time: u64,
    /// The parameter is the time when session disconnects. The unit is seconds.
    #[prost(uint64, tag = "6")]
    pub distinct_time: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSessionRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The id of the client.
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveLastWillMessageRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The id of the client.
    #[prost(string, tag = "2")]
    pub client_id: ::prost::alloc::string::String,
    /// The parameter contains last will message, encoded from a `LastWillData` object into a binary format.
    #[prost(bytes = "vec", tag = "3")]
    pub last_will_message: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAclRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListAclReply {
    /// The parameter contains a list of acls, encoded from a `Vec<MQTTAcl>` into a binary format.
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub acls: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteAclRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The parameter contains acl information, encoded from a `MQTTAcl` object into a binary format.
    #[prost(bytes = "vec", tag = "2")]
    pub acl: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateAclRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The parameter contains acl information, encoded from a `MQTTAcl` object into a binary format.
    #[prost(bytes = "vec", tag = "2")]
    pub acl: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBlacklistRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBlacklistReply {
    /// The parameter contains a list of blacklist, encoded from a `Vec<MQTTAclBlackList>` into a binary format.
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub blacklists: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateBlacklistRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The parameter contains blacklist information, encoded from a `MQTTAclBlackList` object into a binary format.
    #[prost(bytes = "vec", tag = "2")]
    pub blacklist: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteBlacklistRequest {
    /// The name of the cluster.
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    /// The type of blacklist. Refer to the `MQTTAclBlackListType` enum for specific values.
    #[prost(string, tag = "2")]
    pub blacklist_type: ::prost::alloc::string::String,
    /// The name of the resource.
    #[prost(string, tag = "3")]
    pub resource_name: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod mqtt_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct MqttServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MqttServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MqttServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MqttServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MqttServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Returns a list of users based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `user_name: String` (Option): The name of the user.
        ///
        /// Returns:
        /// - `users: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTUser>` into a binary format.
        pub async fn list_user(
            &mut self,
            request: impl tonic::IntoRequest<super::ListUserRequest>,
        ) -> std::result::Result<tonic::Response<super::ListUserReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/ListUser",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("mqtt.MqttService", "ListUser"));
            self.inner.unary(req, path, codec).await
        }
        /// Creates the corresponding user based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `user_name: String`: The name of the user.
        /// - `content: Vec<u8>`: The parameter contains user information, encoded from a `MQTTUser` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn create_user(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/CreateUser",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "CreateUser"));
            self.inner.unary(req, path, codec).await
        }
        /// Deletes the corresponding user based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `user_name: String`: The name of the user.
        ///
        /// Returns: An empty struct.
        pub async fn delete_user(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/DeleteUser",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "DeleteUser"));
            self.inner.unary(req, path, codec).await
        }
        /// Returns a list of sessions based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String` (Option): The id of the client.
        ///
        /// Returns:
        /// - `sessions: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTSession>` into a binary format.
        pub async fn list_session(
            &mut self,
            request: impl tonic::IntoRequest<super::ListSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListSessionReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/ListSession",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "ListSession"));
            self.inner.unary(req, path, codec).await
        }
        /// Creates the corresponding session based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        /// - `session: Vec<u8>`: The parameter contains session information, encoded from a `MQTTSession` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn create_session(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/CreateSession",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "CreateSession"));
            self.inner.unary(req, path, codec).await
        }
        /// Updates the corresponding session based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        /// - `connection_id: u64` (Option): The id of the connection.
        /// - `broker_id: u64` (Option): The id of the broker.
        /// - `reconnect_time: u64` (Option): The parameter is the time when session reconnects. The unit is seconds.
        /// - `distinct_time: u64` (Option): The parameter is the time when session disconnects. The unit is seconds.
        ///
        /// Returns: An empty struct.
        pub async fn update_session(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/UpdateSession",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "UpdateSession"));
            self.inner.unary(req, path, codec).await
        }
        /// Deletes the corresponding session based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        ///
        /// Returns: An empty struct.
        pub async fn delete_session(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/DeleteSession",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "DeleteSession"));
            self.inner.unary(req, path, codec).await
        }
        /// Returns a list of topics based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        ///
        /// Returns:
        /// - `topics: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTTopic>` into a binary format.
        pub async fn list_topic(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTopicRequest>,
        ) -> std::result::Result<tonic::Response<super::ListTopicReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/ListTopic",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "ListTopic"));
            self.inner.unary(req, path, codec).await
        }
        /// Creates the corresponding topic based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        /// - `content: Vec<u8>`: The parameter contains topic information, encoded from a `MQTTTopic` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn create_topic(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTopicRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/CreateTopic",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "CreateTopic"));
            self.inner.unary(req, path, codec).await
        }
        /// Deletes the corresponding topic based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        ///
        /// Returns: An empty struct.
        pub async fn delete_topic(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTopicRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/DeleteTopic",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "DeleteTopic"));
            self.inner.unary(req, path, codec).await
        }
        /// Sets the retain message for the corresponding topic based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        /// - `retain_message: Vec<u8>`: The parameter contains retain message, encoded from a `MQTTMessage` object into a binary format.
        /// - `retain_message_expired_at: u64`: The parameter is the expiration time of the retain message. The unit is seconds.
        ///
        /// Returns: An empty struct.
        pub async fn set_topic_retain_message(
            &mut self,
            request: impl tonic::IntoRequest<super::SetTopicRetainMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/SetTopicRetainMessage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "SetTopicRetainMessage"));
            self.inner.unary(req, path, codec).await
        }
        /// Gets the share sub leader based on the request
        ///
        /// Parameters:
        /// - `group_name: String`: The name of the group.
        /// - `cluster_name: String`: The name of the cluster.
        ///
        /// Returns:
        /// - `broker_id: u64`: The ip address of the broker
        /// - `broker_addr: String`: The ip address of the broker
        /// - `extend_info: String`: The parameter for the extended information of the broker node.
        pub async fn get_share_sub_leader(
            &mut self,
            request: impl tonic::IntoRequest<super::GetShareSubLeaderRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetShareSubLeaderReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/GetShareSubLeader",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "GetShareSubLeader"));
            self.inner.unary(req, path, codec).await
        }
        /// Saves the client's will message based on request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        /// - `last_will_message: Vec<u8>`: The parameter contains last will message, encoded from a `LastWillData` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn save_last_will_message(
            &mut self,
            request: impl tonic::IntoRequest<super::SaveLastWillMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/SaveLastWillMessage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "SaveLastWillMessage"));
            self.inner.unary(req, path, codec).await
        }
        /// Returns a list of ACLs based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        ///
        /// Returns:
        /// - `acls: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTAcl>` into a binary format.
        pub async fn list_acl(
            &mut self,
            request: impl tonic::IntoRequest<super::ListAclRequest>,
        ) -> std::result::Result<tonic::Response<super::ListAclReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/mqtt.MqttService/ListAcl");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("mqtt.MqttService", "ListAcl"));
            self.inner.unary(req, path, codec).await
        }
        /// Deletes the corresponding ACL based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `acl: Vec<u8>`: The parameter contains acl information, encoded from a `MQTTAcl` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn delete_acl(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteAclRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/DeleteAcl",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "DeleteAcl"));
            self.inner.unary(req, path, codec).await
        }
        /// Creates the corresponding ACL based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `acl: Vec<u8>`: The parameter contains acl information, encoded from a `MQTTAcl` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn create_acl(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateAclRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/CreateAcl",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "CreateAcl"));
            self.inner.unary(req, path, codec).await
        }
        /// Returns a list of blacklist based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        ///
        /// Returns:
        /// - `blacklists: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTAclBlackList>` into a binary format.
        pub async fn list_blacklist(
            &mut self,
            request: impl tonic::IntoRequest<super::ListBlacklistRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListBlacklistReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/ListBlacklist",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "ListBlacklist"));
            self.inner.unary(req, path, codec).await
        }
        /// Deletes the corresponding blacklist based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `blacklist_type: String`: The type of blacklist. Refer to the `MQTTAclBlackListType` enum for specific values.
        /// - `resource_name: String`: The name of the resource.
        ///
        /// Returns: An empty struct.
        pub async fn delete_blacklist(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteBlacklistRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/DeleteBlacklist",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "DeleteBlacklist"));
            self.inner.unary(req, path, codec).await
        }
        /// Creates the corresponding blacklist based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `blacklist: Vec<u8>`: The parameter contains blacklist information, encoded from a `MQTTAclBlackList` object into a binary format.
        ///
        /// Returns: An empty struct.
        pub async fn create_blacklist(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateBlacklistRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/mqtt.MqttService/CreateBlacklist",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("mqtt.MqttService", "CreateBlacklist"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod mqtt_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MqttServiceServer.
    #[async_trait]
    pub trait MqttService: Send + Sync + 'static {
        /// Returns a list of users based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `user_name: String` (Option): The name of the user.
        ///
        /// Returns:
        /// - `users: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTUser>` into a binary format.
        async fn list_user(
            &self,
            request: tonic::Request<super::ListUserRequest>,
        ) -> std::result::Result<tonic::Response<super::ListUserReply>, tonic::Status>;
        /// Creates the corresponding user based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `user_name: String`: The name of the user.
        /// - `content: Vec<u8>`: The parameter contains user information, encoded from a `MQTTUser` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn create_user(
            &self,
            request: tonic::Request<super::CreateUserRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Deletes the corresponding user based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `user_name: String`: The name of the user.
        ///
        /// Returns: An empty struct.
        async fn delete_user(
            &self,
            request: tonic::Request<super::DeleteUserRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Returns a list of sessions based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String` (Option): The id of the client.
        ///
        /// Returns:
        /// - `sessions: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTSession>` into a binary format.
        async fn list_session(
            &self,
            request: tonic::Request<super::ListSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListSessionReply>,
            tonic::Status,
        >;
        /// Creates the corresponding session based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        /// - `session: Vec<u8>`: The parameter contains session information, encoded from a `MQTTSession` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn create_session(
            &self,
            request: tonic::Request<super::CreateSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Updates the corresponding session based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        /// - `connection_id: u64` (Option): The id of the connection.
        /// - `broker_id: u64` (Option): The id of the broker.
        /// - `reconnect_time: u64` (Option): The parameter is the time when session reconnects. The unit is seconds.
        /// - `distinct_time: u64` (Option): The parameter is the time when session disconnects. The unit is seconds.
        ///
        /// Returns: An empty struct.
        async fn update_session(
            &self,
            request: tonic::Request<super::UpdateSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Deletes the corresponding session based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        ///
        /// Returns: An empty struct.
        async fn delete_session(
            &self,
            request: tonic::Request<super::DeleteSessionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Returns a list of topics based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        ///
        /// Returns:
        /// - `topics: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTTopic>` into a binary format.
        async fn list_topic(
            &self,
            request: tonic::Request<super::ListTopicRequest>,
        ) -> std::result::Result<tonic::Response<super::ListTopicReply>, tonic::Status>;
        /// Creates the corresponding topic based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        /// - `content: Vec<u8>`: The parameter contains topic information, encoded from a `MQTTTopic` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn create_topic(
            &self,
            request: tonic::Request<super::CreateTopicRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Deletes the corresponding topic based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        ///
        /// Returns: An empty struct.
        async fn delete_topic(
            &self,
            request: tonic::Request<super::DeleteTopicRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Sets the retain message for the corresponding topic based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `topic_name: String`: The name of the topic.
        /// - `retain_message: Vec<u8>`: The parameter contains retain message, encoded from a `MQTTMessage` object into a binary format.
        /// - `retain_message_expired_at: u64`: The parameter is the expiration time of the retain message. The unit is seconds.
        ///
        /// Returns: An empty struct.
        async fn set_topic_retain_message(
            &self,
            request: tonic::Request<super::SetTopicRetainMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Gets the share sub leader based on the request
        ///
        /// Parameters:
        /// - `group_name: String`: The name of the group.
        /// - `cluster_name: String`: The name of the cluster.
        ///
        /// Returns:
        /// - `broker_id: u64`: The ip address of the broker
        /// - `broker_addr: String`: The ip address of the broker
        /// - `extend_info: String`: The parameter for the extended information of the broker node.
        async fn get_share_sub_leader(
            &self,
            request: tonic::Request<super::GetShareSubLeaderRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetShareSubLeaderReply>,
            tonic::Status,
        >;
        /// Saves the client's will message based on request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `client_id: String`: The id of the client.
        /// - `last_will_message: Vec<u8>`: The parameter contains last will message, encoded from a `LastWillData` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn save_last_will_message(
            &self,
            request: tonic::Request<super::SaveLastWillMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Returns a list of ACLs based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        ///
        /// Returns:
        /// - `acls: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTAcl>` into a binary format.
        async fn list_acl(
            &self,
            request: tonic::Request<super::ListAclRequest>,
        ) -> std::result::Result<tonic::Response<super::ListAclReply>, tonic::Status>;
        /// Deletes the corresponding ACL based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `acl: Vec<u8>`: The parameter contains acl information, encoded from a `MQTTAcl` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn delete_acl(
            &self,
            request: tonic::Request<super::DeleteAclRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Creates the corresponding ACL based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `acl: Vec<u8>`: The parameter contains acl information, encoded from a `MQTTAcl` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn create_acl(
            &self,
            request: tonic::Request<super::CreateAclRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Returns a list of blacklist based on the parameters of the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        ///
        /// Returns:
        /// - `blacklists: Vec<Vec<u8>>`: It's the result of encoding a `Vec<MQTTAclBlackList>` into a binary format.
        async fn list_blacklist(
            &self,
            request: tonic::Request<super::ListBlacklistRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListBlacklistReply>,
            tonic::Status,
        >;
        /// Deletes the corresponding blacklist based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `blacklist_type: String`: The type of blacklist. Refer to the `MQTTAclBlackListType` enum for specific values.
        /// - `resource_name: String`: The name of the resource.
        ///
        /// Returns: An empty struct.
        async fn delete_blacklist(
            &self,
            request: tonic::Request<super::DeleteBlacklistRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        /// Creates the corresponding blacklist based on the request
        ///
        /// Parameters:
        /// - `cluster_name: String`: The name of the cluster.
        /// - `blacklist: Vec<u8>`: The parameter contains blacklist information, encoded from a `MQTTAclBlackList` object into a binary format.
        ///
        /// Returns: An empty struct.
        async fn create_blacklist(
            &self,
            request: tonic::Request<super::CreateBlacklistRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct MqttServiceServer<T: MqttService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: MqttService> MqttServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MqttServiceServer<T>
    where
        T: MqttService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/mqtt.MqttService/ListUser" => {
                    #[allow(non_camel_case_types)]
                    struct ListUserSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::ListUserRequest>
                    for ListUserSvc<T> {
                        type Response = super::ListUserReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::list_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/CreateUser" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::CreateUserRequest>
                    for CreateUserSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::create_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/DeleteUser" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::DeleteUserRequest>
                    for DeleteUserSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::delete_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/ListSession" => {
                    #[allow(non_camel_case_types)]
                    struct ListSessionSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::ListSessionRequest>
                    for ListSessionSvc<T> {
                        type Response = super::ListSessionReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListSessionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::list_session(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListSessionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/CreateSession" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSessionSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::CreateSessionRequest>
                    for CreateSessionSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSessionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::create_session(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateSessionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/UpdateSession" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateSessionSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::UpdateSessionRequest>
                    for UpdateSessionSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateSessionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::update_session(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateSessionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/DeleteSession" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteSessionSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::DeleteSessionRequest>
                    for DeleteSessionSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteSessionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::delete_session(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteSessionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/ListTopic" => {
                    #[allow(non_camel_case_types)]
                    struct ListTopicSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::ListTopicRequest>
                    for ListTopicSvc<T> {
                        type Response = super::ListTopicReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTopicRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::list_topic(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListTopicSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/CreateTopic" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTopicSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::CreateTopicRequest>
                    for CreateTopicSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateTopicRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::create_topic(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateTopicSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/DeleteTopic" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTopicSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::DeleteTopicRequest>
                    for DeleteTopicSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTopicRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::delete_topic(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteTopicSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/SetTopicRetainMessage" => {
                    #[allow(non_camel_case_types)]
                    struct SetTopicRetainMessageSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::SetTopicRetainMessageRequest>
                    for SetTopicRetainMessageSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetTopicRetainMessageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::set_topic_retain_message(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SetTopicRetainMessageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/GetShareSubLeader" => {
                    #[allow(non_camel_case_types)]
                    struct GetShareSubLeaderSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::GetShareSubLeaderRequest>
                    for GetShareSubLeaderSvc<T> {
                        type Response = super::GetShareSubLeaderReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetShareSubLeaderRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::get_share_sub_leader(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetShareSubLeaderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/SaveLastWillMessage" => {
                    #[allow(non_camel_case_types)]
                    struct SaveLastWillMessageSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::SaveLastWillMessageRequest>
                    for SaveLastWillMessageSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SaveLastWillMessageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::save_last_will_message(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SaveLastWillMessageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/ListAcl" => {
                    #[allow(non_camel_case_types)]
                    struct ListAclSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::ListAclRequest>
                    for ListAclSvc<T> {
                        type Response = super::ListAclReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListAclRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::list_acl(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListAclSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/DeleteAcl" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteAclSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::DeleteAclRequest>
                    for DeleteAclSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteAclRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::delete_acl(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteAclSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/CreateAcl" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAclSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::CreateAclRequest>
                    for CreateAclSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateAclRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::create_acl(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateAclSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/ListBlacklist" => {
                    #[allow(non_camel_case_types)]
                    struct ListBlacklistSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::ListBlacklistRequest>
                    for ListBlacklistSvc<T> {
                        type Response = super::ListBlacklistReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListBlacklistRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::list_blacklist(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListBlacklistSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/DeleteBlacklist" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteBlacklistSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::DeleteBlacklistRequest>
                    for DeleteBlacklistSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteBlacklistRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::delete_blacklist(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteBlacklistSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/mqtt.MqttService/CreateBlacklist" => {
                    #[allow(non_camel_case_types)]
                    struct CreateBlacklistSvc<T: MqttService>(pub Arc<T>);
                    impl<
                        T: MqttService,
                    > tonic::server::UnaryService<super::CreateBlacklistRequest>
                    for CreateBlacklistSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateBlacklistRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MqttService>::create_blacklist(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateBlacklistSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: MqttService> Clone for MqttServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: MqttService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: MqttService> tonic::server::NamedService for MqttServiceServer<T> {
        const NAME: &'static str = "mqtt.MqttService";
    }
}
