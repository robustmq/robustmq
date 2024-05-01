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

use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum RobustMQError {
    #[error("{0}")]
    CommmonError(String),

    #[error("Operation cannot be initiated because the Leader exists in the cluster")]
    LeaderExistsNotAllowElection,

    #[error("Node is currently in the voting state. The target node ID is : {node_id:?}")]
    NodeBeingVotedOn { node_id: u64 },

    #[error(
        "Node ID is unavailable. The data format may be incorrect. The node id is : {node_id:?}"
    )]
    UnavailableNodeId { node_id: u64 },

    #[error("Multiple leaders exist in a cluster, Node:{0} diff {1}")]
    MultipleLeaders(String, String),

    #[error(
        "The service connection is incorrect, possibly because the service port is not started"
    )]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("Grpc call of the node failed,Grpc status was {0}")]
    MetaGrpcStatus(Status),

    #[error("Leader node does not exist in the Meta cluster, which may be due to the election process or the election failure.")]
    MetaClusterNotLeaderNode,

    #[error("Description The interface {0} submitted logs to the commit log")]
    MetaLogCommitTimeout(String),

    #[error("Failed to submit Raft message to Raft state machine, error message: {0}")]
    RaftStepCommitFail(String),

    #[error("Failed to propose Raft message to Raft state machine, error message: {0}")]
    RaftProposeCommitFail(String),

    #[error("Failed to ConfChange Raft message to Raft state machine, error message: {0}")]
    RaftConfChangeCommitFail(String),

    #[error("Connection pool connecting to IP {0} is missing connections")]
    MissingConnectionAvailable(String),

    #[error("No connection information available, {0}")]
    NoAvailableConnection(String),

    #[error("Parameter cannot be empty, parameter name: {0}")]
    ParameterCannotBeNull(String),

    #[error("Module {0} does not support this feature {1}")]
    NotSupportFeature(String, String),

    #[error("Connection ID [0] information not found in cache.")]
    NotFoundConnectionInCache(u64),

    #[error("Session ID [0] information not found in cache.")]
    NotFoundSessionInCache(u64),

    #[error("Client [*] information not found in cache.")]
    NotFoundClientInCache(String),

    #[error("There is a problem with the length [{0}] of the Packet. Please check the length of the request packet")]
    PacketLenthError(usize),

    #[error("topic name is not available")]
    TopicNameInvalid(),

    #[error("client id [{0}] Format error")]
    ClientIdFormatError(String),

    #[error("Cluster does not exist")]
    ClusterDoesNotExist,

    #[error("Resource does not exist")]
    ResourceDoesNotExist,

    #[error("No available nodes in the cluster")]
    ClusterNoAvailableNode,
}

#[cfg(test)]
mod tests {
    use crate::errors::RobustMQError;

    #[test]
    fn thiserror_to_string() {
        println!(
            "{}",
            RobustMQError::LeaderExistsNotAllowElection.to_string()
        );
        println!(
            "{}",
            RobustMQError::NodeBeingVotedOn { node_id: 18 }.to_string()
        );
    }
}
