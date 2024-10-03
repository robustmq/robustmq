#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClusterStatusRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClusterStatusReply {
    #[prost(string, tag = "1")]
    pub leader: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub members: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "3")]
    pub votes: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartbeatRequest {
    #[prost(enumeration = "super::common::ClusterType", tag = "1")]
    pub cluster_type: i32,
    #[prost(string, tag = "2")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub node_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendRaftMessageRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub message: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendRaftMessageReply {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendRaftConfChangeRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub message: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendRaftConfChangeReply {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeListRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeListReply {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub nodes: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterNodeRequest {
    #[prost(enumeration = "super::common::ClusterType", tag = "1")]
    pub cluster_type: i32,
    #[prost(string, tag = "2")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub node_ip: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub node_id: u64,
    #[prost(string, tag = "5")]
    pub node_inner_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub extend_info: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnRegisterNodeRequest {
    #[prost(enumeration = "super::common::ClusterType", tag = "1")]
    pub cluster_type: i32,
    #[prost(string, tag = "2")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub node_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportMonitorRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub node_id: u64,
    #[prost(float, tag = "3")]
    pub cpu_rate: f32,
    #[prost(float, tag = "4")]
    pub memory_rate: f32,
    #[prost(float, tag = "5")]
    pub disk_rate: f32,
    #[prost(float, tag = "6")]
    pub network_rate: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetResourceConfigRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub resources: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bytes = "vec", tag = "3")]
    pub config: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourceConfigRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub resources: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourceConfigReply {
    #[prost(bytes = "vec", tag = "1")]
    pub config: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceConfigRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub resources: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetIdempotentDataRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub producer_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub seq_num: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExistsIdempotentDataRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub producer_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub seq_num: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExistsIdempotentDataReply {
    #[prost(bool, tag = "1")]
    pub exists: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteIdempotentDataRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub producer_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub seq_num: u64,
}
/// Generated client implementations.
pub mod placement_center_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct PlacementCenterServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PlacementCenterServiceClient<tonic::transport::Channel> {
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
    impl<T> PlacementCenterServiceClient<T>
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
        ) -> PlacementCenterServiceClient<InterceptedService<T, F>>
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
            PlacementCenterServiceClient::new(
                InterceptedService::new(inner, interceptor),
            )
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
        pub async fn cluster_status(
            &mut self,
            request: impl tonic::IntoRequest<super::ClusterStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ClusterStatusReply>,
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
                "/placement.PlacementCenterService/ClusterStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "ClusterStatus"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn node_list(
            &mut self,
            request: impl tonic::IntoRequest<super::NodeListRequest>,
        ) -> std::result::Result<tonic::Response<super::NodeListReply>, tonic::Status> {
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
                "/placement.PlacementCenterService/NodeList",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("placement.PlacementCenterService", "NodeList"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn register_node(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterNodeRequest>,
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
                "/placement.PlacementCenterService/RegisterNode",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "RegisterNode"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn un_register_node(
            &mut self,
            request: impl tonic::IntoRequest<super::UnRegisterNodeRequest>,
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
                "/placement.PlacementCenterService/UnRegisterNode",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "UnRegisterNode"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::HeartbeatRequest>,
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
                "/placement.PlacementCenterService/Heartbeat",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "Heartbeat"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn report_monitor(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportMonitorRequest>,
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
                "/placement.PlacementCenterService/ReportMonitor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "ReportMonitor"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn send_raft_message(
            &mut self,
            request: impl tonic::IntoRequest<super::SendRaftMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SendRaftMessageReply>,
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
                "/placement.PlacementCenterService/SendRaftMessage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "SendRaftMessage",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn send_raft_conf_change(
            &mut self,
            request: impl tonic::IntoRequest<super::SendRaftConfChangeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SendRaftConfChangeReply>,
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
                "/placement.PlacementCenterService/SendRaftConfChange",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "SendRaftConfChange",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_resource_config(
            &mut self,
            request: impl tonic::IntoRequest<super::SetResourceConfigRequest>,
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
                "/placement.PlacementCenterService/SetResourceConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "SetResourceConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_resource_config(
            &mut self,
            request: impl tonic::IntoRequest<super::GetResourceConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetResourceConfigReply>,
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
                "/placement.PlacementCenterService/GetResourceConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "GetResourceConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_resource_config(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteResourceConfigRequest>,
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
                "/placement.PlacementCenterService/DeleteResourceConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "DeleteResourceConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn set_idempotent_data(
            &mut self,
            request: impl tonic::IntoRequest<super::SetIdempotentDataRequest>,
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
                "/placement.PlacementCenterService/SetIdempotentData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "SetIdempotentData",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn exists_idempotent_data(
            &mut self,
            request: impl tonic::IntoRequest<super::ExistsIdempotentDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExistsIdempotentDataReply>,
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
                "/placement.PlacementCenterService/ExistsIdempotentData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "ExistsIdempotentData",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_idempotent_data(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteIdempotentDataRequest>,
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
                "/placement.PlacementCenterService/DeleteIdempotentData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "DeleteIdempotentData",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod placement_center_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with PlacementCenterServiceServer.
    #[async_trait]
    pub trait PlacementCenterService: Send + Sync + 'static {
        async fn cluster_status(
            &self,
            request: tonic::Request<super::ClusterStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ClusterStatusReply>,
            tonic::Status,
        >;
        async fn node_list(
            &self,
            request: tonic::Request<super::NodeListRequest>,
        ) -> std::result::Result<tonic::Response<super::NodeListReply>, tonic::Status>;
        async fn register_node(
            &self,
            request: tonic::Request<super::RegisterNodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn un_register_node(
            &self,
            request: tonic::Request<super::UnRegisterNodeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn heartbeat(
            &self,
            request: tonic::Request<super::HeartbeatRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn report_monitor(
            &self,
            request: tonic::Request<super::ReportMonitorRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn send_raft_message(
            &self,
            request: tonic::Request<super::SendRaftMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SendRaftMessageReply>,
            tonic::Status,
        >;
        async fn send_raft_conf_change(
            &self,
            request: tonic::Request<super::SendRaftConfChangeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SendRaftConfChangeReply>,
            tonic::Status,
        >;
        async fn set_resource_config(
            &self,
            request: tonic::Request<super::SetResourceConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn get_resource_config(
            &self,
            request: tonic::Request<super::GetResourceConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetResourceConfigReply>,
            tonic::Status,
        >;
        async fn delete_resource_config(
            &self,
            request: tonic::Request<super::DeleteResourceConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn set_idempotent_data(
            &self,
            request: tonic::Request<super::SetIdempotentDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        async fn exists_idempotent_data(
            &self,
            request: tonic::Request<super::ExistsIdempotentDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExistsIdempotentDataReply>,
            tonic::Status,
        >;
        async fn delete_idempotent_data(
            &self,
            request: tonic::Request<super::DeleteIdempotentDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct PlacementCenterServiceServer<T: PlacementCenterService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: PlacementCenterService> PlacementCenterServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for PlacementCenterServiceServer<T>
    where
        T: PlacementCenterService,
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
                "/placement.PlacementCenterService/ClusterStatus" => {
                    #[allow(non_camel_case_types)]
                    struct ClusterStatusSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::ClusterStatusRequest>
                    for ClusterStatusSvc<T> {
                        type Response = super::ClusterStatusReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ClusterStatusRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::cluster_status(
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
                        let method = ClusterStatusSvc(inner);
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
                "/placement.PlacementCenterService/NodeList" => {
                    #[allow(non_camel_case_types)]
                    struct NodeListSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::NodeListRequest>
                    for NodeListSvc<T> {
                        type Response = super::NodeListReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NodeListRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::node_list(&inner, request)
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
                        let method = NodeListSvc(inner);
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
                "/placement.PlacementCenterService/RegisterNode" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterNodeSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::RegisterNodeRequest>
                    for RegisterNodeSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RegisterNodeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::register_node(
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
                        let method = RegisterNodeSvc(inner);
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
                "/placement.PlacementCenterService/UnRegisterNode" => {
                    #[allow(non_camel_case_types)]
                    struct UnRegisterNodeSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::UnRegisterNodeRequest>
                    for UnRegisterNodeSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UnRegisterNodeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::un_register_node(
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
                        let method = UnRegisterNodeSvc(inner);
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
                "/placement.PlacementCenterService/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::HeartbeatRequest>
                    for HeartbeatSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HeartbeatRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::heartbeat(&inner, request)
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
                        let method = HeartbeatSvc(inner);
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
                "/placement.PlacementCenterService/ReportMonitor" => {
                    #[allow(non_camel_case_types)]
                    struct ReportMonitorSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::ReportMonitorRequest>
                    for ReportMonitorSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReportMonitorRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::report_monitor(
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
                        let method = ReportMonitorSvc(inner);
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
                "/placement.PlacementCenterService/SendRaftMessage" => {
                    #[allow(non_camel_case_types)]
                    struct SendRaftMessageSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::SendRaftMessageRequest>
                    for SendRaftMessageSvc<T> {
                        type Response = super::SendRaftMessageReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendRaftMessageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::send_raft_message(
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
                        let method = SendRaftMessageSvc(inner);
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
                "/placement.PlacementCenterService/SendRaftConfChange" => {
                    #[allow(non_camel_case_types)]
                    struct SendRaftConfChangeSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::SendRaftConfChangeRequest>
                    for SendRaftConfChangeSvc<T> {
                        type Response = super::SendRaftConfChangeReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SendRaftConfChangeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::send_raft_conf_change(
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
                        let method = SendRaftConfChangeSvc(inner);
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
                "/placement.PlacementCenterService/SetResourceConfig" => {
                    #[allow(non_camel_case_types)]
                    struct SetResourceConfigSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::SetResourceConfigRequest>
                    for SetResourceConfigSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetResourceConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::set_resource_config(
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
                        let method = SetResourceConfigSvc(inner);
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
                "/placement.PlacementCenterService/GetResourceConfig" => {
                    #[allow(non_camel_case_types)]
                    struct GetResourceConfigSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::GetResourceConfigRequest>
                    for GetResourceConfigSvc<T> {
                        type Response = super::GetResourceConfigReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetResourceConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::get_resource_config(
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
                        let method = GetResourceConfigSvc(inner);
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
                "/placement.PlacementCenterService/DeleteResourceConfig" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteResourceConfigSvc<T: PlacementCenterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::DeleteResourceConfigRequest>
                    for DeleteResourceConfigSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteResourceConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::delete_resource_config(
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
                        let method = DeleteResourceConfigSvc(inner);
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
                "/placement.PlacementCenterService/SetIdempotentData" => {
                    #[allow(non_camel_case_types)]
                    struct SetIdempotentDataSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::SetIdempotentDataRequest>
                    for SetIdempotentDataSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetIdempotentDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::set_idempotent_data(
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
                        let method = SetIdempotentDataSvc(inner);
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
                "/placement.PlacementCenterService/ExistsIdempotentData" => {
                    #[allow(non_camel_case_types)]
                    struct ExistsIdempotentDataSvc<T: PlacementCenterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::ExistsIdempotentDataRequest>
                    for ExistsIdempotentDataSvc<T> {
                        type Response = super::ExistsIdempotentDataReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ExistsIdempotentDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::exists_idempotent_data(
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
                        let method = ExistsIdempotentDataSvc(inner);
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
                "/placement.PlacementCenterService/DeleteIdempotentData" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteIdempotentDataSvc<T: PlacementCenterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::DeleteIdempotentDataRequest>
                    for DeleteIdempotentDataSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteIdempotentDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::delete_idempotent_data(
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
                        let method = DeleteIdempotentDataSvc(inner);
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
    impl<T: PlacementCenterService> Clone for PlacementCenterServiceServer<T> {
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
    impl<T: PlacementCenterService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PlacementCenterService> tonic::server::NamedService
    for PlacementCenterServiceServer<T> {
        const NAME: &'static str = "placement.PlacementCenterService";
    }
}
