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
    ConsumerCreateRequest, ConsumerDeleteResponse, ConsumerInfoResponse, ConsumerLeaderResponse,
    ConsumerListRequest, ConsumerListResponse, ConsumerMsgNextRequest, ConsumerNamesResponse,
    ConsumerPauseRequest, ConsumerPauseResponse,
};

/// `$JS.API.CONSUMER.CREATE.<stream>` — ephemeral consumer
pub async fn process_consumer_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: ConsumerCreateRequest,
) -> Result<ConsumerInfoResponse, NatsBrokerError> {
    todo!("CONSUMER.CREATE")
}

/// `$JS.API.CONSUMER.CREATE.<stream>.<consumer>` — named ephemeral consumer
pub async fn process_consumer_create_named(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _req: ConsumerCreateRequest,
) -> Result<ConsumerInfoResponse, NatsBrokerError> {
    todo!("CONSUMER.CREATE (named)")
}

/// `$JS.API.CONSUMER.DURABLE.CREATE.<stream>.<consumer>`
pub async fn process_consumer_durable_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _req: ConsumerCreateRequest,
) -> Result<ConsumerInfoResponse, NatsBrokerError> {
    todo!("CONSUMER.DURABLE.CREATE")
}

/// `$JS.API.CONSUMER.DELETE.<stream>.<consumer>`
pub async fn process_consumer_delete(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<ConsumerDeleteResponse, NatsBrokerError> {
    todo!("CONSUMER.DELETE")
}

/// `$JS.API.CONSUMER.INFO.<stream>.<consumer>`
pub async fn process_consumer_info(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<ConsumerInfoResponse, NatsBrokerError> {
    todo!("CONSUMER.INFO")
}

/// `$JS.API.CONSUMER.LIST.<stream>`
pub async fn process_consumer_list(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: ConsumerListRequest,
) -> Result<ConsumerListResponse, NatsBrokerError> {
    todo!("CONSUMER.LIST")
}

/// `$JS.API.CONSUMER.NAMES.<stream>`
pub async fn process_consumer_names(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _req: ConsumerListRequest,
) -> Result<ConsumerNamesResponse, NatsBrokerError> {
    todo!("CONSUMER.NAMES")
}

/// `$JS.API.CONSUMER.MSG.NEXT.<stream>.<consumer>`
pub async fn process_consumer_msg_next(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _req: ConsumerMsgNextRequest,
) -> Result<(), NatsBrokerError> {
    // Messages are pushed directly to the client's reply-to subject,
    // not returned as a single response body.
    todo!("CONSUMER.MSG.NEXT")
}

/// `$JS.API.CONSUMER.LEADER.STEPDOWN.<stream>.<consumer>`
pub async fn process_consumer_leader_stepdown(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<ConsumerLeaderResponse, NatsBrokerError> {
    todo!("CONSUMER.LEADER.STEPDOWN")
}

/// `$JS.API.CONSUMER.PAUSE.<stream>.<consumer>`
pub async fn process_consumer_pause(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _req: ConsumerPauseRequest,
) -> Result<ConsumerPauseResponse, NatsBrokerError> {
    todo!("CONSUMER.PAUSE")
}
