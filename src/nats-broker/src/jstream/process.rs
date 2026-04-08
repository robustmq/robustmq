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
use crate::jstream::protocol::{
    AckNextRequest, AdvisoryEvent, ConsumerCreateRequest, ConsumerListRequest,
    ConsumerMsgNextRequest, ConsumerPauseRequest, DirectGetRequest, NakRequest, ObjectMeta,
    ObjectRequest, StreamCreateRequest, StreamListRequest, StreamMsgDeleteRequest,
    StreamMsgGetRequest, StreamPeerRemoveRequest, StreamPurgeRequest, StreamSnapshotRequest,
};
use crate::jstream::stream::{
    process_stream_create, process_stream_delete, process_stream_info,
    process_stream_leader_stepdown, process_stream_list, process_stream_msg_delete,
    process_stream_msg_get, process_stream_names, process_stream_peer_remove, process_stream_purge,
    process_stream_restore, process_stream_snapshot, process_stream_update,
};
use bytes::Bytes;
use protocol::nats::packet::NatsPacket;
use serde::Serialize;

pub async fn js_command(
    ctx: &NatsProcessContext,
    subject: &str,
    reply_to: Option<&str>,
    _headers: &Option<Bytes>,
    payload: &Bytes,
) -> Option<NatsPacket> {
    let parsed = match JsCommand::parse(subject) {
        Some(c) => c,
        None => {
            return Some(NatsPacket::Err(format!(
                "unrecognized JetStream subject: {}",
                subject
            )))
        }
    };

    // Helper: deserialize payload into T; return ERR packet on failure.
    macro_rules! parse_req {
        ($t:ty) => {
            match serde_json::from_slice::<$t>(payload) {
                Ok(v) => v,
                Err(e) => return Some(NatsPacket::Err(format!("invalid request body: {}", e))),
            }
        };
        // Zero-byte / optional payload: use Default when empty.
        ($t:ty, default) => {
            if payload.is_empty() {
                <$t>::default()
            } else {
                match serde_json::from_slice::<$t>(payload) {
                    Ok(v) => v,
                    Err(e) => return Some(NatsPacket::Err(format!("invalid request body: {}", e))),
                }
            }
        };
    }

    // Most commands return a serializable response; ACK/event commands return ().
    // We use an Option<String> so fire-and-forget paths return None (no reply sent).
    let result: Result<Option<String>, NatsBrokerError> = match parsed {
        // ── Info ──────────────────────────────────────────────────
        JsCommand::Info => process_info(ctx).await.and_then(to_json),
        JsCommand::AccountInfo => process_account_info(ctx).await.and_then(to_json),

        // ── Stream ────────────────────────────────────────────────
        JsCommand::StreamCreate { stream } => {
            let req = parse_req!(StreamCreateRequest);
            process_stream_create(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::StreamUpdate { stream } => {
            let req = parse_req!(StreamCreateRequest);
            process_stream_update(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::StreamDelete { stream } => {
            process_stream_delete(ctx, &stream).await.and_then(to_json)
        }
        JsCommand::StreamInfo { stream } => {
            process_stream_info(ctx, &stream).await.and_then(to_json)
        }
        JsCommand::StreamList => {
            let req = parse_req!(StreamListRequest, default);
            process_stream_list(ctx, req).await.and_then(to_json)
        }
        JsCommand::StreamNames => {
            let req = parse_req!(StreamListRequest, default);
            process_stream_names(ctx, req).await.and_then(to_json)
        }
        JsCommand::StreamPurge { stream } => {
            let req = parse_req!(StreamPurgeRequest, default);
            process_stream_purge(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::StreamMsgGet { stream } => {
            let req = parse_req!(StreamMsgGetRequest);
            process_stream_msg_get(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::StreamMsgDelete { stream } => {
            let req = parse_req!(StreamMsgDeleteRequest);
            process_stream_msg_delete(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::StreamSnapshot { stream } => {
            let req = parse_req!(StreamSnapshotRequest);
            process_stream_snapshot(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::StreamRestore { stream } => {
            process_stream_restore(ctx, &stream).await.and_then(to_json)
        }
        JsCommand::StreamLeaderStepdown { stream } => process_stream_leader_stepdown(ctx, &stream)
            .await
            .and_then(to_json),
        JsCommand::StreamPeerRemove { stream } => {
            let req = parse_req!(StreamPeerRemoveRequest);
            process_stream_peer_remove(ctx, &stream, req)
                .await
                .and_then(to_json)
        }

        // ── Consumer ──────────────────────────────────────────────
        JsCommand::ConsumerCreate { stream } => {
            let req = parse_req!(ConsumerCreateRequest);
            process_consumer_create(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerCreateNamed { stream, consumer } => {
            let req = parse_req!(ConsumerCreateRequest);
            process_consumer_create_named(ctx, &stream, &consumer, req)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerDurableCreate { stream, consumer } => {
            let req = parse_req!(ConsumerCreateRequest);
            process_consumer_durable_create(ctx, &stream, &consumer, req)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerDelete { stream, consumer } => {
            process_consumer_delete(ctx, &stream, &consumer)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerInfo { stream, consumer } => {
            process_consumer_info(ctx, &stream, &consumer)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerList { stream } => {
            let req = parse_req!(ConsumerListRequest, default);
            process_consumer_list(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerNames { stream } => {
            let req = parse_req!(ConsumerListRequest, default);
            process_consumer_names(ctx, &stream, req)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerMsgNext { stream, consumer } => {
            let req = parse_req!(ConsumerMsgNextRequest, default);
            process_consumer_msg_next(ctx, &stream, &consumer, req)
                .await
                .map(|_| None)
        }
        JsCommand::ConsumerLeaderStepdown { stream, consumer } => {
            process_consumer_leader_stepdown(ctx, &stream, &consumer)
                .await
                .and_then(to_json)
        }
        JsCommand::ConsumerPause { stream, consumer } => {
            let req = parse_req!(ConsumerPauseRequest);
            process_consumer_pause(ctx, &stream, &consumer, req)
                .await
                .and_then(to_json)
        }

        // ── Direct Get ────────────────────────────────────────────
        JsCommand::DirectGet { stream } => {
            let req = parse_req!(DirectGetRequest);
            // Direct Get returns raw bytes + headers, not JSON — handled separately.
            process_direct_get(ctx, &stream, req).await.map(|_| None)
        }
        JsCommand::DirectGetBySubject { stream, subject } => {
            process_direct_get_by_subject(ctx, &stream, &subject)
                .await
                .map(|_| None)
        }
        JsCommand::DirectGetLast { stream, subject } => {
            process_direct_get_last(ctx, &stream, &subject)
                .await
                .map(|_| None)
        }

        // ── ACK (fire-and-forget) ─────────────────────────────────
        JsCommand::Ack { stream, consumer } => {
            process_ack(ctx, &stream, &consumer).await.map(|_| None)
        }
        JsCommand::Nak { stream, consumer } => {
            let req = parse_req!(NakRequest, default);
            process_nak(ctx, &stream, &consumer, req)
                .await
                .map(|_| None)
        }
        JsCommand::AckProgress { stream, consumer } => {
            process_ack_progress(ctx, &stream, &consumer)
                .await
                .map(|_| None)
        }
        JsCommand::AckTerm { stream, consumer } => process_ack_term(ctx, &stream, &consumer)
            .await
            .map(|_| None),
        JsCommand::AckNext { stream, consumer } => {
            let req = parse_req!(AckNextRequest, default);
            process_ack_next(ctx, &stream, &consumer, req)
                .await
                .map(|_| None)
        }

        // ── Advisory Events (fire-and-forget) ─────────────────────
        JsCommand::EventStreamCreated { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_stream_created(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventStreamDeleted { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_stream_deleted(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventStreamUpdated { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_stream_updated(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventConsumerCreated { stream, consumer } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_consumer_created(ctx, &stream, &consumer, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventConsumerDeleted { stream, consumer } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_consumer_deleted(ctx, &stream, &consumer, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventApiAudit { subject } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_api_audit(ctx, &subject, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventSnapshotCreate { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_snapshot_create(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventSnapshotComplete { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_snapshot_complete(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventRestoreCreate { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_restore_create(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventRestoreComplete { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_restore_complete(ctx, &stream, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventConsumerLeaderElected { stream, consumer } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_consumer_leader_elected(ctx, &stream, &consumer, event)
                .await
                .map(|_| None)
        }
        JsCommand::EventStreamLeaderElected { stream } => {
            let event = parse_req!(AdvisoryEvent);
            process_event_stream_leader_elected(ctx, &stream, event)
                .await
                .map(|_| None)
        }

        // ── KV Store ──────────────────────────────────────────────
        JsCommand::KvPut { bucket, key } => process_kv_put(ctx, &bucket, &key, payload.clone())
            .await
            .and_then(to_json),
        JsCommand::KvGet { bucket, key } => process_kv_get(ctx, &bucket, &key).await.map(|_| None),
        JsCommand::KvDelete { bucket, key } => process_kv_delete(ctx, &bucket, &key)
            .await
            .and_then(to_json),
        JsCommand::KvPurge { bucket, key } => {
            process_kv_purge(ctx, &bucket, &key).await.and_then(to_json)
        }
        JsCommand::KvKeys { bucket } => process_kv_keys(ctx, &bucket).await.and_then(to_json),
        JsCommand::KvWatch { bucket, pattern } => {
            process_kv_watch(ctx, &bucket, &pattern).await.map(|_| None)
        }

        // ── Object Store ──────────────────────────────────────────
        JsCommand::ObjPut { bucket } => {
            let meta = parse_req!(ObjectMeta);
            process_obj_put(ctx, &bucket, meta).await.and_then(to_json)
        }
        JsCommand::ObjGet { bucket, object: _ } => {
            let req = parse_req!(ObjectRequest);
            process_obj_get(ctx, &bucket, req).await.map(|_| None)
        }
        JsCommand::ObjDelete { bucket, object: _ } => {
            let req = parse_req!(ObjectRequest);
            process_obj_delete(ctx, &bucket, req)
                .await
                .and_then(to_json)
        }
        JsCommand::ObjInfo { bucket, object: _ } => {
            let req = parse_req!(ObjectRequest);
            process_obj_info(ctx, &bucket, req).await.and_then(to_json)
        }
        JsCommand::ObjList { bucket } => process_obj_list(ctx, &bucket).await.and_then(to_json),
        JsCommand::ObjWatch { bucket } => process_obj_watch(ctx, &bucket).await.map(|_| None),
    };

    if let Some(reply_subject) = reply_to {
        let body = match result {
            Ok(Some(json)) => Bytes::from(json),
            Ok(None) => return None,
            Err(e) => Bytes::from(e.to_string()),
        };
        let _ = reply_nats_packet(ctx, reply_subject, body).await;
    }

    None
}

fn to_json<T: Serialize>(v: T) -> Result<Option<String>, NatsBrokerError> {
    serde_json::to_string(&v)
        .map(Some)
        .map_err(|e| NatsBrokerError::CommonError(e.to_string()))
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
