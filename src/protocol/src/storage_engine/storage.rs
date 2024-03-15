#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestHeader {
    #[prost(enumeration = "ApiKey", tag = "1")]
    pub request_api_key: i32,
    #[prost(enumeration = "ApiVersion", tag = "2")]
    pub request_api_version: i32,
    #[prost(uint32, tag = "3")]
    pub correlation_id: u32,
    #[prost(string, tag = "4")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "5")]
    pub tagged_fields: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseHeader {
    #[prost(uint32, tag = "1")]
    pub correlation_id: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub tagged_fields: ::prost::alloc::vec::Vec<u8>,
}
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
pub struct ProduceReuest {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<RequestHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<ProduceBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProduceBody {
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
pub struct ProduceResponse {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ApiKey {
    Produce = 0,
    Consume = 1,
}
impl ApiKey {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ApiKey::Produce => "produce",
            ApiKey::Consume => "consume",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "produce" => Some(Self::Produce),
            "consume" => Some(Self::Consume),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ApiVersion {
    V0 = 0,
}
impl ApiVersion {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ApiVersion::V0 => "V0",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "V0" => Some(Self::V0),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
}
impl ErrorCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ErrorCode::Success => "Success",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Success" => Some(Self::Success),
            _ => None,
        }
    }
}
