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

use std::{num::ParseIntError, string::FromUtf8Error};

use common_base::error::{common::CommonError, mqtt_protocol_error::MQTTProtocolError};
use pulsar::Error as PulsarError;
use quinn::{ReadToEndError, StoppedError, WriteError};
use r2d2;
use rdkafka::error::KafkaError;
use reqwest::Error as RequestError;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum MqttBrokerError {
    #[error("{0}")]
    FromIoError(#[from] std::io::Error),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    FromCommonError(#[from] CommonError),

    #[error("{0}")]
    RegexError(#[from] regex::Error),

    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("{0}")]
    R2d2PoolError(#[from] r2d2::Error),

    #[error("{0}")]
    FromR2d2MysqlError(#[from] r2d2_mysql::mysql::Error),

    #[error("{0}")]
    FromR2d2PostgresError(#[from] r2d2_postgres::postgres::Error),

    #[error("{0}")]
    FromRedisError(#[from] redis::RedisError),

    #[error("{0}")]
    FromRustlsError(#[from] rustls::Error),

    #[error("{0}")]
    TokioTimeErrorElapsed(#[from] tokio::time::error::Elapsed),

    #[error("{0}")]
    TokioBroadcastSendError(#[from] tokio::sync::broadcast::error::SendError<bool>),

    #[error("{0}")]
    TokioSyncBroadcastErrorRecvError(#[from] tokio::sync::broadcast::error::RecvError),

    #[error("{0}")]
    RequestError(#[from] RequestError),

    #[error("{0}")]
    StrumParseError(#[from] strum::ParseError),

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("{0}")]
    GrepError(#[from] grep::regex::Error),

    #[error("{0}")]
    QuinnWriteError(#[from] WriteError),

    #[error("{0}")]
    QuinnClosedStreamError(#[from] quinn::ClosedStream),

    #[error("{0}")]
    QuinnStoppedError(#[from] StoppedError),

    #[error("{0}")]
    QuinnReadToEndError(#[from] ReadToEndError),

    #[error("{0}")]
    FromProtocolMQTTCommonError(#[from] MQTTProtocolError),

    #[error("Topic alias is too long. alias is {0}")]
    TopicAliasTooLong(u16),

    #[error("Topic name cannot be empty")]
    TopicNameIsEmpty,

    #[error("Topic alias ** did not find the corresponding actual Topic")]
    TopicAliasInvalid(Option<u16>),

    #[error("Topic {0} is incorrectly formatted")]
    TopicNameIncorrectlyFormatted(String),

    #[error("Connection ID [0] information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("Client {0} has no connection available")]
    ClientNoAvailableConnection(String),

    #[error("Client ID cannot be empty")]
    ClientIDIsEmpty,

    #[error("Cluster name does not match")]
    ClusterNotMatch(String),

    #[error("Message content length exceeds limit, Max :{0}, current :{1}")]
    PacketLengthError(usize, usize),

    #[error("Cluster is in self-protection state, please request later")]
    ClusterIsInSelfProtection,

    #[error("message is not in UTF8 format")]
    PayloadFormatInvalid,

    #[error(
        "Subscribe to push, send QOS2 message to client {0}, wait for PubRec message timeout."
    )]
    SubPublishWaitPubRecTimeout(String),

    #[error("Bad subscription Path [{0}] does not exist")]
    SubscriptionPathNotExists(String),

    #[error("User does not exist")]
    UserDoesNotExist,

    #[error("user has been existed")]
    UserAlreadyExist,

    #[error("Session does not exist")]
    SessionDoesNotExist,

    #[error("Topic [{0}] does not exist")]
    TopicDoesNotExist(String),

    #[error("Unavailable storage type")]
    UnavailableStorageType,

    #[error("Unsupported authentication type: {0}")]
    UnsupportedAuthType(String),

    #[error("Password configuration not found")]
    PasswordConfigNotFound,

    #[error("{0}")]
    CommonError(String),

    #[error("Invalid acl action")]
    InvalidAclAction,

    #[error("Subscription path {0} is not available")]
    InvalidSubPath(String),

    #[error("invalid acl permission")]
    InvalidAclPermission,

    #[error("topicRewriteRule has been existed")]
    TopicRewriteRuleAlreadyExist,

    #[error("Failed to build Message")]
    FailedToBuildMessage,

    #[error("topic {0} does not conform to the format of deferred topic")]
    NotConformDeferredTopic(String),

    #[error("Publish message was delayed, the target Topic failed to resolve, Topic name {0}")]
    DelayPublishDecodeTopicNameFail(String),

    #[error("Invalid schema type {0}")]
    InvalidSchemaType(String),

    #[error("Session {0} is null, skip push message")]
    SessionNullSkipPushMessage(String),

    #[error("Connection {0} is null, skip push message")]
    ConnectionNullSkipPushMessage(String),

    #[error("kafka error: {0}")]
    KafkaError(#[from] KafkaError),

    #[error("[write_frame]Connection management could not obtain an available {0} connection. Connection ID: {1}")]
    NotObtainAvailableConnection(String, u64),

    #[error("[write_frame]Encountered a DashMap lock and failed to obtain {0} connection information, connection ID: {1}")]
    FailedObtailConnectionByLock(String, u64),

    #[error("Failed to write data to the mqtt {0} client, error message: {1}")]
    FailedToWriteClient(String, String),

    #[error("Websocket encode packet failed, error message: {0}")]
    WebsocketEncodePacketFailed(String),

    #[error("Websocket decode packet failed, error message: {0}")]
    WebsocketDecodePacketFailed(String),

    #[error("Client {0}, the size of the subscription sent packets exceeds the limit. Packet size :{1}, Limit size :{2}")]
    PacketsExceedsLimitBySubPublish(String, usize, u32),

    #[error("Operation timeout, timeout time :{0}, operation: {1}")]
    OperationTimeout(u64, String),

    #[error("gRPC error: {0}")]
    RpcError(#[from] Status),

    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),

    #[error("Password verification error: {0}")]
    PasswordVerificationError(String),

    #[error("Unsupported MAC function: {0}")]
    UnsupportedMacFunction(String),

    #[error("Pulsar error: {0}")]
    PulsarError(#[from] PulsarError),
}

impl From<MqttBrokerError> for Status {
    fn from(e: MqttBrokerError) -> Self {
        Status::cancelled(e.to_string())
    }
}
