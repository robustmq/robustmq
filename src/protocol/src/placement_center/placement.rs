#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonReply {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartbeatRequest {
    #[prost(enumeration = "ClusterType", tag = "1")]
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
pub struct RegisterNodeRequest {
    #[prost(enumeration = "ClusterType", tag = "1")]
    pub cluster_type: i32,
    #[prost(string, tag = "2")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub node_ip: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub node_id: u64,
    #[prost(uint32, tag = "5")]
    pub node_port: u32,
    #[prost(string, tag = "6")]
    pub extend_info: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnRegisterNodeRequest {
    #[prost(enumeration = "ClusterType", tag = "1")]
    pub cluster_type: i32,
    #[prost(string, tag = "2")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub node_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateShardRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
    #[prost(uint32, tag = "3")]
    pub replica: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShardRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShardReply {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shard_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub shard_name: ::prost::alloc::string::String,
    #[prost(uint32, tag = "4")]
    pub replica: u32,
    #[prost(bytes = "vec", tag = "5")]
    pub replicas: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "6")]
    pub status: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteShardRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SegmentInfo {}
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ClusterType {
    BrokerServer = 0,
    StorageEngine = 1,
    PlacementCenter = 2,
}
impl ClusterType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ClusterType::BrokerServer => "BrokerServer",
            ClusterType::StorageEngine => "StorageEngine",
            ClusterType::PlacementCenter => "PlacementCenter",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BrokerServer" => Some(Self::BrokerServer),
            "StorageEngine" => Some(Self::StorageEngine),
            "PlacementCenter" => Some(Self::PlacementCenter),
            _ => None,
        }
    }
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
        ///
        pub async fn register_node(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterNodeRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status> {
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
        ///
        pub async fn un_register_node(
            &mut self,
            request: impl tonic::IntoRequest<super::UnRegisterNodeRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status> {
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
                "/placement.PlacementCenterService/UnRegister_node",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "placement.PlacementCenterService",
                        "UnRegister_node",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn create_shard(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateShardRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status> {
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
                "/placement.PlacementCenterService/CreateShard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "CreateShard"),
                );
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn get_shard(
            &mut self,
            request: impl tonic::IntoRequest<super::GetShardRequest>,
        ) -> std::result::Result<tonic::Response<super::GetShardReply>, tonic::Status> {
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
                "/placement.PlacementCenterService/GetShard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("placement.PlacementCenterService", "GetShard"));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn delete_shard(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteShardRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status> {
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
                "/placement.PlacementCenterService/DeleteShard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "DeleteShard"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Broker node reports a heartbeat, notifying Meta Server that the node is alive
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::HeartbeatRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status> {
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
                "/placement.PlacementCenterService/heartbeat",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("placement.PlacementCenterService", "heartbeat"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn report_monitor(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportMonitorRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status> {
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
        /// Raft messages are sent between nodes
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
        /// Send ConfChange messages to Raft with other nodes
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
    }
}
/// Generated server implementations.
pub mod placement_center_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with PlacementCenterServiceServer.
    #[async_trait]
    pub trait PlacementCenterService: Send + Sync + 'static {
        ///
        async fn register_node(
            &self,
            request: tonic::Request<super::RegisterNodeRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status>;
        ///
        async fn un_register_node(
            &self,
            request: tonic::Request<super::UnRegisterNodeRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status>;
        ///
        async fn create_shard(
            &self,
            request: tonic::Request<super::CreateShardRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status>;
        ///
        async fn get_shard(
            &self,
            request: tonic::Request<super::GetShardRequest>,
        ) -> std::result::Result<tonic::Response<super::GetShardReply>, tonic::Status>;
        ///
        async fn delete_shard(
            &self,
            request: tonic::Request<super::DeleteShardRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status>;
        /// Broker node reports a heartbeat, notifying Meta Server that the node is alive
        async fn heartbeat(
            &self,
            request: tonic::Request<super::HeartbeatRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status>;
        async fn report_monitor(
            &self,
            request: tonic::Request<super::ReportMonitorRequest>,
        ) -> std::result::Result<tonic::Response<super::CommonReply>, tonic::Status>;
        /// Raft messages are sent between nodes
        async fn send_raft_message(
            &self,
            request: tonic::Request<super::SendRaftMessageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SendRaftMessageReply>,
            tonic::Status,
        >;
        /// Send ConfChange messages to Raft with other nodes
        async fn send_raft_conf_change(
            &self,
            request: tonic::Request<super::SendRaftConfChangeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SendRaftConfChangeReply>,
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
                "/placement.PlacementCenterService/RegisterNode" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterNodeSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::RegisterNodeRequest>
                    for RegisterNodeSvc<T> {
                        type Response = super::CommonReply;
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
                "/placement.PlacementCenterService/UnRegister_node" => {
                    #[allow(non_camel_case_types)]
                    struct UnRegister_nodeSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::UnRegisterNodeRequest>
                    for UnRegister_nodeSvc<T> {
                        type Response = super::CommonReply;
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
                        let method = UnRegister_nodeSvc(inner);
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
                "/placement.PlacementCenterService/CreateShard" => {
                    #[allow(non_camel_case_types)]
                    struct CreateShardSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::CreateShardRequest>
                    for CreateShardSvc<T> {
                        type Response = super::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateShardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::create_shard(&inner, request)
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
                        let method = CreateShardSvc(inner);
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
                "/placement.PlacementCenterService/GetShard" => {
                    #[allow(non_camel_case_types)]
                    struct GetShardSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::GetShardRequest>
                    for GetShardSvc<T> {
                        type Response = super::GetShardReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetShardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::get_shard(&inner, request)
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
                        let method = GetShardSvc(inner);
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
                "/placement.PlacementCenterService/DeleteShard" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteShardSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::DeleteShardRequest>
                    for DeleteShardSvc<T> {
                        type Response = super::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteShardRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as PlacementCenterService>::delete_shard(&inner, request)
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
                        let method = DeleteShardSvc(inner);
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
                "/placement.PlacementCenterService/heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct heartbeatSvc<T: PlacementCenterService>(pub Arc<T>);
                    impl<
                        T: PlacementCenterService,
                    > tonic::server::UnaryService<super::HeartbeatRequest>
                    for heartbeatSvc<T> {
                        type Response = super::CommonReply;
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
                        let method = heartbeatSvc(inner);
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
                        type Response = super::CommonReply;
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
