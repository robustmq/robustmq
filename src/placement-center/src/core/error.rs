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

use std::io;
use std::net::AddrParseError;
use std::string::FromUtf8Error;

use common_base::error::common::CommonError;
use openraft::error::{ClientWriteError, RaftError};
use thiserror::Error;

use crate::raft::typeconfig::TypeConfig;

#[derive(Error, Debug)]
pub enum PlacementCenterError {
    #[error("{0}")]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("{0}")]
    ErrorKind(#[from] Box<bincode::ErrorKind>),

    #[error("{0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("{0}")]
    RocksdbError(#[from] rocksdb::Error),

    #[error("{0}")]
    BaseCommonError(#[from] CommonError),

    #[error("{0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    AddrParseError(#[from] AddrParseError),

    #[error("{0}")]
    TokioTimeErrorElapsed(#[from] tokio::time::error::Elapsed),

    #[error("{0}")]
    OpenRaftError(#[from] RaftError<TypeConfig, ClientWriteError<TypeConfig>>),

    #[error("Description The interface {0} submitted logs to the commit log")]
    RaftLogCommitTimeout(String),

    #[error("{0}")]
    CommonError(String),

    #[error("Cluster {0} does not exist")]
    ClusterDoesNotExist(String),

    #[error("No Broker nodes are available")]
    NoAvailableBrokerNode,

    #[error("Node {0} does not exist")]
    NodeDoesNotExist(u64),

    #[error("Shard {0} does not exist")]
    ShardDoesNotExist(String),

    #[error("Segment {0} does not exist")]
    SegmentDoesNotExist(String),

    #[error("Segment Meta {0} does not exist")]
    SegmentMetaDoesNotExist(String),

    #[error("segment {0} state cache error, server current state {1}, passed state {2}")]
    SegmentStateError(String, String, String),

    #[error("Segment {0} state is {1} and no deletion is allowed")]
    NoAllowDeleteSegment(String, String),

    #[error("Shard {0} already has enough segments, there is no need to create new segments")]
    ShardHasEnoughSegment(String),

    #[error(
        "There are not enough nodes available in the cluster, {0} is needed, and currently {1}."
    )]
    NotEnoughNodes(u32, u32),

    #[error("Execution result is empty, please check whether the server logic is normal")]
    ExecutionResultIsEmpty,

    #[error("RocksDB Family {0} not available")]
    RocksDBFamilyNotAvailable(String),

    #[error("Invalid Segment, Segment {0} is greater than Start Segment {1}")]
    InvalidSegmentGreaterThan(u32, u32),

    #[error("Invalid Segment, Segment {0} is less than Start Segment {1}")]
    InvalidSegmentLessThan(u32, u32),

    #[error("Request parameters [{0}] cannot be null")]
    RequestParamsNotEmpty(String),

    #[error("Session {0} does not exist")]
    SessionDoesNotExist(String),

    #[error("Number of replicas is incorrect; {0} is needed, but {1} is obtained")]
    NumberOfReplicasIsIncorrect(u32, usize),

    #[error("Topic [{0}] does not exist")]
    TopicDoesNotExist(String),

    #[error("Topic [{0}] already exist")]
    TopicAlreadyExist(String),

    #[error("Segment {0} is in the wrong state. It should not be sealed.")]
    SegmentWrongState(String),

    #[error("Connector {0} Not found")]
    ConnectorNotFound(String),

    #[error("Connector [{0}] already exist")]
    ConnectorAlreadyExist(String),

    #[error("Schema [{0}] does not exist")]
    SchemaDoesNotExist(String),

    #[error("Subscribe [{0}] does not exist")]
    SubscribeDoesNotExist(String),

    #[error("Schema {0} Not found")]
    SchemaNotFound(String),

    #[error("Will Message {0} does not exist")]
    WillMessageDoesNotExist(String),

    #[error("Schema [{0}] already exist")]
    SchemaAlreadyExist(String),
}
