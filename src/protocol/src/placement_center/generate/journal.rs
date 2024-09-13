// Copyright 2023 RobustMQ Team
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
pub struct CreateSegmentRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSegmentRequest {
    #[prost(string, tag = "1")]
    pub cluster_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub shard_name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub segment_seq: u64,
}
/// Generated client implementations.
pub mod engine_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct EngineServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EngineServiceClient<tonic::transport::Channel> {
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
    impl<T> EngineServiceClient<T>
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
        ) -> EngineServiceClient<InterceptedService<T, F>>
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
            EngineServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_shard(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateShardRequest>,
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
                "/journal.EngineService/CreateShard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("journal.EngineService", "CreateShard"));
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
                "/journal.EngineService/GetShard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("journal.EngineService", "GetShard"));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn delete_shard(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteShardRequest>,
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
                "/journal.EngineService/DeleteShard",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("journal.EngineService", "DeleteShard"));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn create_segment(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSegmentRequest>,
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
                "/journal.EngineService/CreateSegment",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("journal.EngineService", "CreateSegment"));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn delete_segment(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteSegmentRequest>,
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
                "/journal.EngineService/DeleteSegment",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("journal.EngineService", "DeleteSegment"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod engine_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with EngineServiceServer.
    #[async_trait]
    pub trait EngineService: Send + Sync + 'static {
        ///
        async fn create_shard(
            &self,
            request: tonic::Request<super::CreateShardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        ///
        async fn get_shard(
            &self,
            request: tonic::Request<super::GetShardRequest>,
        ) -> std::result::Result<tonic::Response<super::GetShardReply>, tonic::Status>;
        ///
        async fn delete_shard(
            &self,
            request: tonic::Request<super::DeleteShardRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        ///
        async fn create_segment(
            &self,
            request: tonic::Request<super::CreateSegmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
        ///
        async fn delete_segment(
            &self,
            request: tonic::Request<super::DeleteSegmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::common::CommonReply>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct EngineServiceServer<T: EngineService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: EngineService> EngineServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EngineServiceServer<T>
    where
        T: EngineService,
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
                "/journal.EngineService/CreateShard" => {
                    #[allow(non_camel_case_types)]
                    struct CreateShardSvc<T: EngineService>(pub Arc<T>);
                    impl<
                        T: EngineService,
                    > tonic::server::UnaryService<super::CreateShardRequest>
                    for CreateShardSvc<T> {
                        type Response = super::super::common::CommonReply;
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
                                <T as EngineService>::create_shard(&inner, request).await
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
                "/journal.EngineService/GetShard" => {
                    #[allow(non_camel_case_types)]
                    struct GetShardSvc<T: EngineService>(pub Arc<T>);
                    impl<
                        T: EngineService,
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
                                <T as EngineService>::get_shard(&inner, request).await
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
                "/journal.EngineService/DeleteShard" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteShardSvc<T: EngineService>(pub Arc<T>);
                    impl<
                        T: EngineService,
                    > tonic::server::UnaryService<super::DeleteShardRequest>
                    for DeleteShardSvc<T> {
                        type Response = super::super::common::CommonReply;
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
                                <T as EngineService>::delete_shard(&inner, request).await
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
                "/journal.EngineService/CreateSegment" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSegmentSvc<T: EngineService>(pub Arc<T>);
                    impl<
                        T: EngineService,
                    > tonic::server::UnaryService<super::CreateSegmentRequest>
                    for CreateSegmentSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSegmentRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EngineService>::create_segment(&inner, request).await
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
                        let method = CreateSegmentSvc(inner);
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
                "/journal.EngineService/DeleteSegment" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteSegmentSvc<T: EngineService>(pub Arc<T>);
                    impl<
                        T: EngineService,
                    > tonic::server::UnaryService<super::DeleteSegmentRequest>
                    for DeleteSegmentSvc<T> {
                        type Response = super::super::common::CommonReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteSegmentRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EngineService>::delete_segment(&inner, request).await
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
                        let method = DeleteSegmentSvc(inner);
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
    impl<T: EngineService> Clone for EngineServiceServer<T> {
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
    impl<T: EngineService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: EngineService> tonic::server::NamedService for EngineServiceServer<T> {
        const NAME: &'static str = "journal.EngineService";
    }
}
