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

use thiserror::Error;

use crate::async_writer::DataSenderPkg;

#[derive(Error, Debug)]
pub enum JournalClientError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    TokioTimeErrorElapsed(#[from] tokio::time::error::Elapsed),

    #[error("{0}")]
    OneshotRecvError(#[from] tokio::sync::broadcast::error::RecvError),

    #[error("{0}")]
    BroadcastSendErrorBool(#[from] tokio::sync::broadcast::error::SendError<bool>),

    #[error("{0}")]
    MpscSendErrorDataSenderPkg(#[from] tokio::sync::mpsc::error::SendError<DataSenderPkg>),

    #[error("{0}")]
    MpscSendErrorBool(#[from] tokio::sync::mpsc::error::SendError<bool>),

    #[error("Node {0} has no available access address, may be cache data inconsistency, ready to trigger update node cache.")]
    NodeNoAvailableAddr(i64),

    #[error("Failed to send request to node {0} with error message :{1}")]
    SendRequestError(i64, String),

    #[error("request is sent to node {0} and the received return packet is empty")]
    ReceivedPacketIsEmpty(i64),

    #[error("Send request to node {0}, received wrong packet, error message :{1}")]
    ReceivedPacketError(i64, String),

    #[error("Sending a request to node {0}, obtaining a connection failed, and the connection is occupied for a long time.")]
    ConnectionIsOccupied(i64),

    #[error("Sending a request to node {0} failed to get a connection, possibly to create a connection.")]
    NoAvailableConn(i64),

    #[error("Received return packet type error, need {0}, received {1}.")]
    ReceivedPacketTypeError(String, String),

    #[error("{0}:{1}")]
    JournalEngineError(String, String),

    #[error("Request {0}, received return packet does not contain Header information and is an invalid packet")]
    ReceivedPacketNotContainHeader(String),

    #[error("Request {0}, received return packet does not contain body information and is an invalid packet")]
    ReceivedPacketNotContainBody(String),

    #[error("{0} has no Leader available")]
    NotLeader(String),

    #[error("Shard {0} has no active segment")]
    NotActiveSegment(String),

    #[error("Shard {0} metadata no exists")]
    NotShardMetadata(String),

    #[error("The write request returns empty")]
    WriteReqReturnEmpty,

    #[error("Addrs cannot be null")]
    AddrsNotEmpty,

    #[error("Sending a request packet, receiving a request returns a timeout")]
    SendPacketTimeout,

    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
}
