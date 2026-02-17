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

use crate::raft::error::{to_bincode_error, to_grpc_error, to_rpc_error};
use crate::raft::type_config::{Node, NodeId, TypeConfig};
use bincode::{deserialize, serialize_into};
use common_metrics::meta::raft::{
    record_rpc_duration, record_rpc_failure, record_rpc_request, record_rpc_success,
};
use grpc_clients::pool::ClientPool;
use openraft::error::{InstallSnapshotError, RPCError, RaftError};
use openraft::network::RPCOption;
use openraft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use openraft::RaftNetwork;
use protocol::meta::meta_service_common::meta_service_service_client::MetaServiceServiceClient;
use protocol::meta::meta_service_common::{AppendRequest, SnapshotRequest};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tonic::transport::Channel;
use tracing::warn;

const SLOW_RPC_WARN_THRESHOLD_MS: f64 = 1000.0;
const RPC_TIMEOUT: Duration = Duration::from_secs(10);
const SNAPSHOT_RPC_TIMEOUT: Duration = Duration::from_secs(60);

pub struct NetworkConnection {
    addr: String,
    machine: String,
    client_pool: Arc<ClientPool>,
}

impl NetworkConnection {
    pub fn new(machine: String, addr: String, client_pool: Arc<ClientPool>) -> Self {
        NetworkConnection {
            addr,
            client_pool,
            machine,
        }
    }

    fn c(&self) -> MetaServiceServiceClient<Channel> {
        MetaServiceServiceClient::new(self.client_pool.get_channel(&self.addr))
    }

    fn serialize_to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
        let mut buf = Vec::with_capacity(4096);
        serialize_into(&mut buf, value)?;
        Ok(buf)
    }

    fn deserialize_from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, bincode::Error> {
        deserialize(bytes)
    }

    async fn append_entries_internal(
        &mut self,
        req: AppendEntriesRequest<TypeConfig>,
    ) -> Result<AppendEntriesResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        let mut c = self.c();

        let value = match Self::serialize_to_bytes(&req) {
            Ok(data) => data,
            Err(e) => {
                return Err(to_bincode_error(
                    e,
                    "Failed to serialize AppendEntriesRequest",
                ))
            }
        };

        let request = AppendRequest {
            machine: self.machine.clone(),
            value,
        };

        let reply = match timeout(RPC_TIMEOUT, c.append(request)).await {
            Ok(Ok(reply)) => reply.into_inner(),
            Ok(Err(e)) => return Err(to_grpc_error(e, "Failed to send AppendEntries RPC")),
            Err(_) => {
                warn!(
                    "Raft RPC timed out. machine={}, op=append_entries, target={}, timeout={}s",
                    self.machine,
                    self.addr,
                    RPC_TIMEOUT.as_secs()
                );
                return Err(to_rpc_error(format!(
                    "AppendEntries RPC to {} timed out after {}s",
                    self.addr,
                    RPC_TIMEOUT.as_secs()
                )));
            }
        };

        let result = match Self::deserialize_from_bytes(&reply.value) {
            Ok(data) => data,
            Err(e) => {
                return Err(to_bincode_error(
                    e,
                    "Failed to deserialize AppendEntriesResponse",
                ))
            }
        };

        Ok(result)
    }

    async fn install_snapshot_internal(
        &mut self,
        req: InstallSnapshotRequest<TypeConfig>,
    ) -> Result<
        InstallSnapshotResponse<NodeId>,
        RPCError<NodeId, Node, RaftError<NodeId, InstallSnapshotError>>,
    > {
        let mut c = self.c();

        let value = match Self::serialize_to_bytes(&req) {
            Ok(data) => data,
            Err(e) => {
                return Err(to_bincode_error(
                    e,
                    "Failed to serialize InstallSnapshotRequest",
                ))
            }
        };

        let request = SnapshotRequest {
            machine: self.machine.clone(),
            value,
        };

        let reply =
            match timeout(SNAPSHOT_RPC_TIMEOUT, c.snapshot(request)).await {
                Ok(Ok(reply)) => reply.into_inner(),
                Ok(Err(e)) => return Err(to_grpc_error(e, "Failed to send InstallSnapshot RPC")),
                Err(_) => {
                    warn!(
                    "Raft RPC timed out. machine={}, op=install_snapshot, target={}, timeout={}s",
                    self.machine, self.addr, SNAPSHOT_RPC_TIMEOUT.as_secs()
                );
                    return Err(to_rpc_error(format!(
                        "InstallSnapshot RPC to {} timed out after {}s",
                        self.addr,
                        SNAPSHOT_RPC_TIMEOUT.as_secs()
                    )));
                }
            };

        let result = match Self::deserialize_from_bytes(&reply.value) {
            Ok(data) => data,
            Err(e) => {
                return Err(to_bincode_error(
                    e,
                    "Failed to deserialize InstallSnapshotResponse",
                ))
            }
        };

        Ok(result)
    }

    async fn vote_internal(
        &mut self,
        req: VoteRequest<NodeId>,
    ) -> Result<VoteResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        let mut c = self.c();

        let value = match Self::serialize_to_bytes(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_bincode_error(e, "Failed to serialize VoteRequest")),
        };

        let request = protocol::meta::meta_service_common::VoteRequest {
            machine: self.machine.clone(),
            value,
        };

        let reply = match timeout(RPC_TIMEOUT, c.vote(request)).await {
            Ok(Ok(reply)) => reply.into_inner(),
            Ok(Err(e)) => return Err(to_grpc_error(e, "Failed to send Vote RPC")),
            Err(_) => {
                warn!(
                    "Raft RPC timed out. machine={}, op=vote, target={}, timeout={}s",
                    self.machine,
                    self.addr,
                    RPC_TIMEOUT.as_secs()
                );
                return Err(to_rpc_error(format!(
                    "Vote RPC to {} timed out after {}s",
                    self.addr,
                    RPC_TIMEOUT.as_secs()
                )));
            }
        };

        let result = match Self::deserialize_from_bytes(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_bincode_error(e, "Failed to deserialize VoteResponse")),
        };

        Ok(result)
    }
}

#[allow(clippy::blocks_in_conditions)]
impl RaftNetwork<TypeConfig> for NetworkConnection {
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        record_rpc_request(&self.machine, "append_entries");
        let start = Instant::now();

        let result = self.append_entries_internal(req).await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_rpc_duration(&self.machine, "append_entries", duration_ms);
        if duration_ms > SLOW_RPC_WARN_THRESHOLD_MS {
            warn!(
                "Raft RPC is slow. machine={}, op=append_entries, target={}, duration_ms={:.2}",
                self.machine, self.addr, duration_ms
            );
        }

        match result {
            Ok(response) => {
                record_rpc_success(&self.machine, "append_entries");
                Ok(response)
            }
            Err(e) => {
                record_rpc_failure(&self.machine, "append_entries");
                Err(e)
            }
        }
    }

    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<
        InstallSnapshotResponse<NodeId>,
        RPCError<NodeId, Node, RaftError<NodeId, InstallSnapshotError>>,
    > {
        record_rpc_request(&self.machine, "install_snapshot");
        let start = Instant::now();

        let result = self.install_snapshot_internal(req).await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_rpc_duration(&self.machine, "install_snapshot", duration_ms);
        if duration_ms > SLOW_RPC_WARN_THRESHOLD_MS {
            warn!(
                "Raft RPC is slow. machine={}, op=install_snapshot, target={}, duration_ms={:.2}",
                self.machine, self.addr, duration_ms
            );
        }

        match result {
            Ok(response) => {
                record_rpc_success(&self.machine, "install_snapshot");
                Ok(response)
            }
            Err(e) => {
                record_rpc_failure(&self.machine, "install_snapshot");
                Err(e)
            }
        }
    }

    async fn vote(
        &mut self,
        req: VoteRequest<NodeId>,
        _option: RPCOption,
    ) -> Result<VoteResponse<NodeId>, RPCError<NodeId, Node, RaftError<NodeId>>> {
        record_rpc_request(&self.machine, "vote");
        let start = Instant::now();

        let result = self.vote_internal(req).await;

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_rpc_duration(&self.machine, "vote", duration_ms);
        if duration_ms > SLOW_RPC_WARN_THRESHOLD_MS {
            warn!(
                "Raft RPC is slow. machine={}, op=vote, target={}, duration_ms={:.2}",
                self.machine, self.addr, duration_ms
            );
        }

        match result {
            Ok(response) => {
                record_rpc_success(&self.machine, "vote");
                Ok(response)
            }
            Err(e) => {
                record_rpc_failure(&self.machine, "vote");
                Err(e)
            }
        }
    }
}
