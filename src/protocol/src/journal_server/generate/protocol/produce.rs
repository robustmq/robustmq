#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TopicData {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub partition_data: ::core::option::Option<PartitionData>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartitionData {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProduceReqBody {
    #[prost(uint32, tag = "1")]
    pub transactional_id: u32,
    #[prost(uint32, tag = "2")]
    pub acks: u32,
    #[prost(uint32, tag = "3")]
    pub timeout_ms: u32,
    #[prost(message, optional, tag = "4")]
    pub topic_data: ::core::option::Option<TopicData>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProduceRespBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProduceReq {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::header::Header>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<ProduceReqBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProduceResp {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::header::Header>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<ProduceRespBody>,
}
