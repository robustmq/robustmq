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
use crate::core::write_client::write_nats_packet;
use crate::handler::command::NatsProcessContext;
use crate::jstream::ack::{
    process_ack, process_ack_next, process_ack_progress, process_ack_term, process_nak,
};
use crate::jstream::command::JsCommand;
use crate::jstream::consumer::{
    process_consumer_create, process_consumer_create_named, process_consumer_delete,
    process_consumer_durable_create, process_consumer_info, process_consumer_leader_stepdown,
    process_consumer_list, process_consumer_msg_next, process_consumer_names,
    process_consumer_pause,
};
use crate::jstream::direct::{
    process_direct_get, process_direct_get_by_subject, process_direct_get_last,
};
use crate::jstream::event::{
    process_event_api_audit, process_event_consumer_created, process_event_consumer_deleted,
    process_event_consumer_leader_elected, process_event_restore_complete,
    process_event_restore_create, process_event_snapshot_complete, process_event_snapshot_create,
    process_event_stream_created, process_event_stream_deleted,
    process_event_stream_leader_elected, process_event_stream_updated,
};
use crate::jstream::info::{process_account_info, process_info};
use crate::jstream::kv::{
    process_kv_delete, process_kv_get, process_kv_keys, process_kv_purge, process_kv_put,
    process_kv_watch,
};
use crate::jstream::object::{
    process_obj_delete, process_obj_get, process_obj_info, process_obj_list, process_obj_put,
    process_obj_watch,
};
use crate::jstream::stream::{
    process_stream_create, process_stream_delete, process_stream_info,
    process_stream_leader_stepdown, process_stream_list, process_stream_msg_delete,
    process_stream_msg_get, process_stream_names, process_stream_peer_remove, process_stream_purge,
    process_stream_restore, process_stream_snapshot, process_stream_update,
};
use bytes::Bytes;
use protocol::nats::packet::NatsPacket;

pub async fn js_command(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    headers: &Option<Bytes>,
    payload: &Bytes,
) -> Option<NatsPacket> {
    let _ = headers;

    let parsed = match JsCommand::parse(subject) {
        Some(c) => c,
        None => {
            return Some(NatsPacket::Err(format!(
                "unrecognized JetStream subject: {}",
                subject
            )))
        }
    };

    let result: Result<Option<String>, NatsBrokerError> = match parsed {
        // ── Info ──────────────────────────────────────────────────
        JsCommand::Info => process_info(ctx).await.map(Some),
        JsCommand::AccountInfo => process_account_info(ctx).await.map(Some),

        // ── Stream ────────────────────────────────────────────────
        JsCommand::StreamCreate { stream } => {
            process_stream_create(ctx, &stream, payload).await.map(Some)
        }
        JsCommand::StreamUpdate { stream } => {
            process_stream_update(ctx, &stream, payload).await.map(Some)
        }
        JsCommand::StreamDelete { stream } => process_stream_delete(ctx, &stream).await.map(Some),
        JsCommand::StreamInfo { stream } => process_stream_info(ctx, &stream).await.map(Some),
        JsCommand::StreamList => process_stream_list(ctx, payload).await.map(Some),
        JsCommand::StreamNames => process_stream_names(ctx, payload).await.map(Some),
        JsCommand::StreamPurge { stream } => {
            process_stream_purge(ctx, &stream, payload).await.map(Some)
        }
        JsCommand::StreamMsgGet { stream } => process_stream_msg_get(ctx, &stream, payload)
            .await
            .map(Some),
        JsCommand::StreamMsgDelete { stream } => process_stream_msg_delete(ctx, &stream, payload)
            .await
            .map(Some),
        JsCommand::StreamSnapshot { stream } => process_stream_snapshot(ctx, &stream, payload)
            .await
            .map(Some),
        JsCommand::StreamRestore { stream } => process_stream_restore(ctx, &stream, payload)
            .await
            .map(Some),
        JsCommand::StreamLeaderStepdown { stream } => {
            process_stream_leader_stepdown(ctx, &stream).await.map(Some)
        }
        JsCommand::StreamPeerRemove { stream } => process_stream_peer_remove(ctx, &stream, payload)
            .await
            .map(Some),

        // ── Consumer ──────────────────────────────────────────────
        JsCommand::ConsumerCreate { stream } => process_consumer_create(ctx, &stream, payload)
            .await
            .map(Some),
        JsCommand::ConsumerCreateNamed { stream, consumer } => {
            process_consumer_create_named(ctx, &stream, &consumer, payload)
                .await
                .map(Some)
        }
        JsCommand::ConsumerDurableCreate { stream, consumer } => {
            process_consumer_durable_create(ctx, &stream, &consumer, payload)
                .await
                .map(Some)
        }
        JsCommand::ConsumerDelete { stream, consumer } => {
            process_consumer_delete(ctx, &stream, &consumer)
                .await
                .map(Some)
        }
        JsCommand::ConsumerInfo { stream, consumer } => {
            process_consumer_info(ctx, &stream, &consumer)
                .await
                .map(Some)
        }
        JsCommand::ConsumerList { stream } => {
            process_consumer_list(ctx, &stream, payload).await.map(Some)
        }
        JsCommand::ConsumerNames { stream } => process_consumer_names(ctx, &stream, payload)
            .await
            .map(Some),
        JsCommand::ConsumerMsgNext { stream, consumer } => {
            process_consumer_msg_next(ctx, &stream, &consumer, payload)
                .await
                .map(Some)
        }
        JsCommand::ConsumerLeaderStepdown { stream, consumer } => {
            process_consumer_leader_stepdown(ctx, &stream, &consumer)
                .await
                .map(Some)
        }
        JsCommand::ConsumerPause { stream, consumer } => {
            process_consumer_pause(ctx, &stream, &consumer, payload)
                .await
                .map(Some)
        }

        // ── Direct get ────────────────────────────────────────────
        JsCommand::DirectGet { stream } => {
            process_direct_get(ctx, &stream, payload).await.map(Some)
        }
        JsCommand::DirectGetBySubject { stream, subject } => {
            process_direct_get_by_subject(ctx, &stream, &subject)
                .await
                .map(Some)
        }
        JsCommand::DirectGetLast { stream, subject } => {
            process_direct_get_last(ctx, &stream, &subject)
                .await
                .map(Some)
        }

        // ── ACK (fire-and-forget, no reply) ───────────────────────
        JsCommand::Ack { stream, consumer } => process_ack(ctx, &stream, &consumer, payload)
            .await
            .map(|_| None),
        JsCommand::Nak { stream, consumer } => process_nak(ctx, &stream, &consumer, payload)
            .await
            .map(|_| None),
        JsCommand::AckProgress { stream, consumer } => {
            process_ack_progress(ctx, &stream, &consumer)
                .await
                .map(|_| None)
        }
        JsCommand::AckTerm { stream, consumer } => process_ack_term(ctx, &stream, &consumer)
            .await
            .map(|_| None),
        JsCommand::AckNext { stream, consumer } => {
            process_ack_next(ctx, &stream, &consumer, payload)
                .await
                .map(|_| None)
        }

        // ── Advisory events (fire-and-forget) ─────────────────────
        JsCommand::EventStreamCreated { stream } => process_event_stream_created(ctx, &stream)
            .await
            .map(|_| None),
        JsCommand::EventStreamDeleted { stream } => process_event_stream_deleted(ctx, &stream)
            .await
            .map(|_| None),
        JsCommand::EventStreamUpdated { stream } => process_event_stream_updated(ctx, &stream)
            .await
            .map(|_| None),
        JsCommand::EventConsumerCreated { stream, consumer } => {
            process_event_consumer_created(ctx, &stream, &consumer)
                .await
                .map(|_| None)
        }
        JsCommand::EventConsumerDeleted { stream, consumer } => {
            process_event_consumer_deleted(ctx, &stream, &consumer)
                .await
                .map(|_| None)
        }
        JsCommand::EventApiAudit { subject } => {
            process_event_api_audit(ctx, &subject).await.map(|_| None)
        }
        JsCommand::EventSnapshotCreate { stream } => process_event_snapshot_create(ctx, &stream)
            .await
            .map(|_| None),
        JsCommand::EventSnapshotComplete { stream } => {
            process_event_snapshot_complete(ctx, &stream)
                .await
                .map(|_| None)
        }
        JsCommand::EventRestoreCreate { stream } => process_event_restore_create(ctx, &stream)
            .await
            .map(|_| None),
        JsCommand::EventRestoreComplete { stream } => process_event_restore_complete(ctx, &stream)
            .await
            .map(|_| None),
        JsCommand::EventConsumerLeaderElected { stream, consumer } => {
            process_event_consumer_leader_elected(ctx, &stream, &consumer)
                .await
                .map(|_| None)
        }
        JsCommand::EventStreamLeaderElected { stream } => {
            process_event_stream_leader_elected(ctx, &stream)
                .await
                .map(|_| None)
        }

        // ── KV store ──────────────────────────────────────────────
        JsCommand::KvPut { bucket, key } => {
            process_kv_put(ctx, &bucket, &key, payload).await.map(Some)
        }
        JsCommand::KvGet { bucket, key } => process_kv_get(ctx, &bucket, &key).await.map(Some),
        JsCommand::KvDelete { bucket, key } => {
            process_kv_delete(ctx, &bucket, &key).await.map(Some)
        }
        JsCommand::KvPurge { bucket, key } => process_kv_purge(ctx, &bucket, &key).await.map(Some),
        JsCommand::KvKeys { bucket } => process_kv_keys(ctx, &bucket).await.map(Some),
        JsCommand::KvWatch { bucket, pattern } => {
            process_kv_watch(ctx, &bucket, &pattern).await.map(|_| None)
        }

        // ── Object store ──────────────────────────────────────────
        JsCommand::ObjPut { bucket } => process_obj_put(ctx, &bucket, payload).await.map(Some),
        JsCommand::ObjGet { bucket, object } => {
            process_obj_get(ctx, &bucket, &object).await.map(Some)
        }
        JsCommand::ObjDelete { bucket, object } => {
            process_obj_delete(ctx, &bucket, &object).await.map(Some)
        }
        JsCommand::ObjInfo { bucket, object } => {
            process_obj_info(ctx, &bucket, &object).await.map(Some)
        }
        JsCommand::ObjList { bucket } => process_obj_list(ctx, &bucket).await.map(Some),
        JsCommand::ObjWatch { bucket } => process_obj_watch(ctx, &bucket).await.map(|_| None),
    };

    if let Some(reply_subject) = reply_to {
        let response = match result {
            Ok(Some(body)) => body,
            Ok(None) => return None,
            Err(e) => e.to_string(),
        };
        let _ = reply_nats_packet(ctx, reply_subject, Bytes::from(response)).await;
    }

    None
}

async fn reply_nats_packet(
    ctx: &NatsProcessContext,
    subject: &str,
    payload: Bytes,
) -> Result<(), NatsBrokerError> {
    let sid = ctx
        .cache_manager
        .get_inbox_sid(subject)
        .unwrap_or_else(|| "0".to_string());
    let packet = NatsPacket::Msg {
        subject: subject.to_string(),
        sid,
        reply_to: None,
        payload,
    };
    write_nats_packet(&ctx.connection_manager, ctx.connect_id, packet).await
}
