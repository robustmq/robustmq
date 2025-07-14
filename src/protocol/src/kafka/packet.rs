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

use kafka_protocol::messages::{
    ApiVersionsRequest, ApiVersionsResponse, FetchRequest, FetchResponse, ListOffsetsRequest,
    ListOffsetsResponse, MetadataRequest, MetadataResponse, ProduceRequest, ProduceResponse,
    RequestHeader, ResponseHeader,
};

#[derive(Debug)]
pub struct KafkaPacketWrapper {
    pub api_version: i16,
    pub header: KafkaHeader,
    pub packet: KafkaPacket,
}

#[derive(Debug)]
pub enum KafkaHeader {
    Request(RequestHeader),
    Response(ResponseHeader),
}

#[derive(Debug)]
pub enum KafkaPacket {
    ProduceReq(ProduceRequest),
    ProduceResponse(ProduceResponse),
    FetchReq(FetchRequest),
    FetchResponse(FetchResponse),
    ListOffsetsReq(ListOffsetsRequest),
    ListOffsetsResponse(ListOffsetsResponse),
    MetadataReq(MetadataRequest),
    MetadataResponse(MetadataResponse),
    ApiVersionReq(ApiVersionsRequest),
    ApiVersionResponse(ApiVersionsResponse),
}
