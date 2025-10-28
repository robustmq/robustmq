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

use anyhow;
use governor::InsufficientCapacity;
use quinn::{ReadToEndError, StoppedError, WriteError};
use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use thiserror::Error;
use tonic::Status;
use valico::json_schema::SchemaError;

use crate::error::mqtt_protocol_error::MQTTProtocolError;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("{0}")]
    TokioBroadcastSendErrorBool(#[from] tokio::sync::broadcast::error::SendError<bool>),

    #[error("{0}")]
    FromTonicTransport(#[from] tonic::transport::Error),

    #[error("{0}")]
    FromErrorKind(#[from] Box<bincode::ErrorKind>),

    #[error("{0}")]
    FromDecodeError(#[from] prost::DecodeError),

    #[error("{0}")]
    QuinnWriteError(#[from] WriteError),

    #[error("{0}")]
    QuinnClosedStreamError(#[from] quinn::ClosedStream),

    #[error("{0}")]
    QuinnStoppedError(#[from] StoppedError),

    #[error("{0}")]
    QuinnReadToEndError(#[from] ReadToEndError),

    #[error("{0}")]
    QuinnReadError(#[from] quinn::ReadError),

    #[error("{0}")]
    QuinnReadExactError(#[from] quinn::ReadExactError),

    #[error("{0}")]
    FromSerdeJsonError(#[from] serde_json::Error),

    #[error("{0}")]
    FromRocksdbError(#[from] rocksdb::Error),

    #[error("{0}")]
    SchemaError(#[from] SchemaError),

    #[error("{0}")]
    FromIoError(#[from] io::Error),

    #[error("{0}")]
    FromMQTTProtocolError(#[from] MQTTProtocolError),

    #[error("{0}")]
    FromRustlsError(#[from] rustls::Error),

    #[error("{0}")]
    AnyHowError(#[from] anyhow::Error),

    #[error("{0}")]
    InsufficientCapacity(#[from] InsufficientCapacity),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    FromAddrParseError(#[from] AddrParseError),

    #[error("{0}")]
    ApacheAvroError(#[from] apache_avro::Error),

    #[error("R2D2 Pool Error: {0}")]
    R2d2PoolError(#[from] r2d2::Error),

    #[error("{0}")]
    FromR2d2MysqlError(#[from] r2d2_mysql::mysql::Error),

    #[error("{0}")]
    FromR2d2PostgresError(#[from] r2d2_postgres::postgres::Error),

    #[error("{0}")]
    RedisError(#[from] redis::RedisError),

    #[error("{0}")]
    FromParseIntError(#[from] ParseIntError),

    #[error("{0}")]
    CommonError(String),

    #[error("Connection ID [0] information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("Grpc call of the node failed,Grpc status was {0}")]
    GrpcServerStatus(#[from] Status),

    #[error("{0} connection pool has no connection information available. {1}")]
    NoAvailableGrpcConnection(String, String),

    #[error("Parameter cannot be empty, parameter name: {0}")]
    ParameterCannotBeNull(String),

    #[error("Invalid parameter format, parameter name: {0}, parameter_value: {1}")]
    InvalidParameterFormat(String, String),

    #[error("Module {0} does not support this feature {1}")]
    NotSupportFeature(String, String),

    #[error("Cannot recognize Kafka protocol {0}")]
    NotSupportKafkaRequest(i16),

    #[error("Kafka Encode cannot recognize package {0}")]
    NotSupportKafkaEncodePacket(String),

    #[error("Unavailable cluster type")]
    UnavailableClusterType,

    #[error("No available nodes in the cluster")]
    ClusterNoAvailableNode,

    #[error("RocksDB Family {0} not available")]
    RocksDBFamilyNotAvailable(String),

    #[error("CRC check for the message data failed")]
    CrcCheckByMessage,

    #[error("Failed to write data to the mqtt {0} client, error message: {1}")]
    FailedToWriteClient(String, String),

    #[error("[write_frame]Connection management could not obtain an available {0} connection. Connection ID: {1}")]
    NotObtainAvailableConnection(String, u64),

    #[error("{0} is an unavailable type of Connector.")]
    IneligibleConnectorType(String),

    #[error("{0}")]
    OpenDALError(#[from] opendal::Error),
}

impl From<CommonError> for Status {
    fn from(e: CommonError) -> Self {
        Status::cancelled(e.to_string())
    }
}
