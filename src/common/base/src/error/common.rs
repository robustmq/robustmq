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

/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io;
use std::net::AddrParseError;
use std::string::FromUtf8Error;
use thiserror::Error;
use tonic::Status;

use crate::error::mqtt_broker::MQTTBrokerError;
use crate::error::placement_center::PlacementCenterError;

#[derive(Error, Debug)]
pub enum CommonError {
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
    IoError(#[from] io::Error),

    #[error("{0}")]
    MQTTBrokerError(#[from] MQTTBrokerError),

    #[error("{0}")]
    PlacementCenterError(#[from] PlacementCenterError),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("{0}")]
    AddrParseError(#[from] AddrParseError),

    #[error("{0}")]
    CommmonError(String),

    #[error("Grpc call of the node failed,Grpc status was {0}")]
    GrpcServerStatus(Status),

    #[error("{0} connection pool has no connection information available. {1}")]
    NoAvailableGrpcConnection(String, String),

    #[error("Parameter cannot be empty, parameter name: {0}")]
    ParameterCannotBeNull(String),

    #[error("Module {0} does not support this feature {1}")]
    NotSupportFeature(String, String),

    #[error("Unavailable storage type")]
    UnavailableStorageType,

    #[error("No available nodes in the cluster")]
    ClusterNoAvailableNode,

    #[error("RocksDB Family {0} not available")]
    RocksDBFamilyNotAvailable(String),
}
