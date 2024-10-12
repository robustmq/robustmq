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
pub struct ReqHeader {
    #[prost(enumeration = "ApiKey", tag = "1")]
    pub api_key: i32,
    #[prost(enumeration = "ApiVersion", tag = "2")]
    pub api_version: i32,
}
/// *  Read Request *
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RespHeader {
    #[prost(enumeration = "ApiKey", tag = "1")]
    pub api_key: i32,
    #[prost(enumeration = "ApiVersion", tag = "2")]
    pub api_version: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadReqBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadRespBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadReq {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<ReqHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<ReadReqBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadResp {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<RespHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<ReadRespBody>,
}
/// * Write Request *
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteReqBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteRespBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteReq {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<ReqHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<WriteReqBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteResp {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<RespHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<WriteRespBody>,
}
/// * Get the Segment where the Shard is active *
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetActiveSegmentReqBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetActiveSegmentRespBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetActiveSegmentReq {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<ReqHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<GetActiveSegmentReqBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetActiveSegmentResp {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<RespHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<GetActiveSegmentRespBody>,
}
/// * Get the Segment where the Shard is active *
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OffsetCommitReqBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OffsetCommitRespBody {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OffsetCommitReq {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<ReqHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<OffsetCommitReqBody>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OffsetCommitResp {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<RespHeader>,
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<OffsetCommitRespBody>,
}
/// * Header *
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ApiKey {
    Read = 0,
    Write = 1,
    GetActiveSegment = 2,
    OffsetCommit = 3,
}
impl ApiKey {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ApiKey::Read => "Read",
            ApiKey::Write => "Write",
            ApiKey::GetActiveSegment => "GetActiveSegment",
            ApiKey::OffsetCommit => "OffsetCommit",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Read" => Some(Self::Read),
            "Write" => Some(Self::Write),
            "GetActiveSegment" => Some(Self::GetActiveSegment),
            "OffsetCommit" => Some(Self::OffsetCommit),
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
