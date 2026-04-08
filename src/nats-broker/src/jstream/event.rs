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
use crate::jstream::protocol::AdvisoryEvent;

/// `$JS.EVENT.ADVISORY.STREAM.CREATED.<stream>`
pub async fn process_event_stream_created(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.CREATED")
}

/// `$JS.EVENT.ADVISORY.STREAM.DELETED.<stream>`
pub async fn process_event_stream_deleted(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.DELETED")
}

/// `$JS.EVENT.ADVISORY.STREAM.UPDATED.<stream>`
pub async fn process_event_stream_updated(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.UPDATED")
}

/// `$JS.EVENT.ADVISORY.CONSUMER.CREATED.<stream>.<consumer>`
pub async fn process_event_consumer_created(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.CONSUMER.CREATED")
}

/// `$JS.EVENT.ADVISORY.CONSUMER.DELETED.<stream>.<consumer>`
pub async fn process_event_consumer_deleted(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.CONSUMER.DELETED")
}

/// `$JS.EVENT.ADVISORY.API`
pub async fn process_event_api_audit(
    _ctx: &NatsProcessContext,
    _subject: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.API")
}

/// `$JS.EVENT.ADVISORY.STREAM.SNAPSHOT.CREATE.<stream>`
pub async fn process_event_snapshot_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.SNAPSHOT.CREATE")
}

/// `$JS.EVENT.ADVISORY.STREAM.SNAPSHOT.COMPLETE.<stream>`
pub async fn process_event_snapshot_complete(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.SNAPSHOT.COMPLETE")
}

/// `$JS.EVENT.ADVISORY.STREAM.RESTORE.CREATE.<stream>`
pub async fn process_event_restore_create(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.RESTORE.CREATE")
}

/// `$JS.EVENT.ADVISORY.STREAM.RESTORE.COMPLETE.<stream>`
pub async fn process_event_restore_complete(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.RESTORE.COMPLETE")
}

/// `$JS.EVENT.ADVISORY.CONSUMER.LEADER_ELECTED.<stream>.<consumer>`
pub async fn process_event_consumer_leader_elected(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.CONSUMER.LEADER_ELECTED")
}

/// `$JS.EVENT.ADVISORY.STREAM.LEADER_ELECTED.<stream>`
pub async fn process_event_stream_leader_elected(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _event: AdvisoryEvent,
) -> Result<(), NatsBrokerError> {
    todo!("EVENT.ADVISORY.STREAM.LEADER_ELECTED")
}
