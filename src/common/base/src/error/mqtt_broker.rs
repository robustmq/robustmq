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

use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum MqttBrokerError {
    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Topic alias is too long. alias is {0}")]
    TopicAliasTooLong(u16),

    #[error("Topic name cannot be empty")]
    TopicNameIsEmpty,

    #[error("topic name is not available")]
    TopicNameInvalid(),

    #[error("Topic {0} is incorrectly formatted")]
    TopicNameIncorrectlyFormatted(String),

    #[error("Connection ID [0] information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("There is a problem with the length [{0}] of the Packet. Please check the length of the request packet")]
    PacketLenthError(usize),

    #[error("Cluster is in self-protection state, please request later")]
    ClusterIsInSelfProtection,

    #[error(
        "Subscribe to push, send QOS2 message to client {0}, wait for PubRec message timeout."
    )]
    SubPublishWaitPubRecTimeout(String),

    #[error("Bad subscription Path [{0}] does not exist")]
    SubscriptionPathNotExists(String),

    #[error("User does not exist")]
    UserDoesNotExist,

    #[error("Session does not exist")]
    SessionDoesNotExist,

    #[error("Topic [{0}] does not exist")]
    TopicDoesNotExist(String),
}
