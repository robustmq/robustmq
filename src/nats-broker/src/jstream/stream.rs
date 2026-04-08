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

use crate::core::error::NatsBrokerError;
use crate::handler::command::NatsProcessContext;
use crate::jstream::protocol::{
    StreamCreateRequest, StreamDeleteResponse, StreamInfoResponse, StreamLeaderResponse,
    StreamListRequest, StreamListResponse, StreamMsgDeleteRequest, StreamMsgDeleteResponse,
    StreamMsgGetRequest, StreamMsgGetResponse, StreamNamesResponse, StreamPeerRemoveRequest,
    StreamPurgeRequest, StreamPurgeResponse, StreamSnapshotRequest,
};

/// `$JS.API.STREAM.CREATE.<stream>`
pub async fn process_stream_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamCreateRequest,
) -> Result<StreamInfoResponse, NatsBrokerError> {
    todo!("STREAM.CREATE")
}

/// `$JS.API.STREAM.UPDATE.<stream>`
pub async fn process_stream_update(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamCreateRequest,
) -> Result<StreamInfoResponse, NatsBrokerError> {
    todo!("STREAM.UPDATE")
}

/// `$JS.API.STREAM.DELETE.<stream>`
pub async fn process_stream_delete(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<StreamDeleteResponse, NatsBrokerError> {
    todo!("STREAM.DELETE")
}

/// `$JS.API.STREAM.INFO.<stream>`
pub async fn process_stream_info(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<StreamInfoResponse, NatsBrokerError> {
    todo!("STREAM.INFO")
}

/// `$JS.API.STREAM.LIST`
pub async fn process_stream_list(
    _ctx: &NatsProcessContext,
    _req: StreamListRequest,
) -> Result<StreamListResponse, NatsBrokerError> {
    todo!("STREAM.LIST")
}

/// `$JS.API.STREAM.NAMES`
pub async fn process_stream_names(
    _ctx: &NatsProcessContext,
    _req: StreamListRequest,
) -> Result<StreamNamesResponse, NatsBrokerError> {
    todo!("STREAM.NAMES")
}

/// `$JS.API.STREAM.PURGE.<stream>`
pub async fn process_stream_purge(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamPurgeRequest,
) -> Result<StreamPurgeResponse, NatsBrokerError> {
    todo!("STREAM.PURGE")
}

/// `$JS.API.STREAM.MSG.GET.<stream>`
pub async fn process_stream_msg_get(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamMsgGetRequest,
) -> Result<StreamMsgGetResponse, NatsBrokerError> {
    todo!("STREAM.MSG.GET")
}

/// `$JS.API.STREAM.MSG.DELETE.<stream>`
pub async fn process_stream_msg_delete(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamMsgDeleteRequest,
) -> Result<StreamMsgDeleteResponse, NatsBrokerError> {
    todo!("STREAM.MSG.DELETE")
}

/// `$JS.API.STREAM.SNAPSHOT.<stream>`
pub async fn process_stream_snapshot(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamSnapshotRequest,
) -> Result<StreamLeaderResponse, NatsBrokerError> {
    todo!("STREAM.SNAPSHOT")
}

/// `$JS.API.STREAM.RESTORE.<stream>`
pub async fn process_stream_restore(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<StreamLeaderResponse, NatsBrokerError> {
    todo!("STREAM.RESTORE")
}

/// `$JS.API.STREAM.LEADER.STEPDOWN.<stream>`
pub async fn process_stream_leader_stepdown(
    _ctx: &NatsProcessContext,
    _stream: &str,
) -> Result<StreamLeaderResponse, NatsBrokerError> {
    todo!("STREAM.LEADER.STEPDOWN")
}

/// `$JS.API.STREAM.PEER.REMOVE.<stream>`
pub async fn process_stream_peer_remove(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: StreamPeerRemoveRequest,
) -> Result<StreamLeaderResponse, NatsBrokerError> {
    todo!("STREAM.PEER.REMOVE")
}
