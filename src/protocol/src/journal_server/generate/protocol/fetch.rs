#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchReqBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchRespBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchReq {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::header::Header>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<FetchReqBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchResp {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::header::Header>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<FetchRespBody>,
}
