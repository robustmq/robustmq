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

use common_base::tools::now_second;
use common_config::broker::broker_config;
use metadata_struct::mqtt::share_group::{
    ShareGroupMember, ShareGroupParams, ShareGroupParamsNats,
};
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::tenant::DEFAULT_TENANT;
use protocol::nats::packet::NatsPacket;

use crate::core::error::NatsProtocolError;
use crate::core::queue_name::{add_member_by_group, delete_member_by_group};
use crate::core::subject::is_inbox_subject;
use crate::handler::command::NatsProcessContext;
use crate::push::parse::{
    snapshot_known_topic_offsets, ParseAction, ParseSubscribeData, SubscribeSource,
};
use crate::storage::subscribe::NatsSubscribeStorage;

pub fn subject_message_tag(tenant: &str, subject: &str) -> String {
    format!("{}_{}", tenant, subject)
}

pub async fn process_sub(
    ctx: &NatsProcessContext,
    subject: &str,
    queue_group: Option<&str>,
    sid: &str,
) -> Result<(), NatsPacket> {
    if broker_config().nats_runtime.auth_required && !ctx.cache_manager.is_login(ctx.connect_id) {
        return Err(NatsPacket::Err(
            NatsProtocolError::AuthorizationViolation.message(),
        ));
    }

    if is_inbox_subject(subject) {
        ctx.cache_manager
            .add_inbox(subject.to_string(), sid.to_string());
        return Ok(());
    }

    let tenant = DEFAULT_TENANT.to_string();

    // Snapshot offsets for every already-existing topic this subject/pattern
    // matches, right now, while "now" still reliably means "subscribe time".
    // Everything after this point — replicating this subscribe intent via
    // raft, matching it against topics, the fanout push loop noticing the
    // resulting subscriber — is asynchronous and can be arbitrarily delayed,
    // so resolving "latest" any later than this would risk skipping messages
    // published in the gap. Queue-group subscribers don't need this (their
    // push path already starts from Earliest, not Latest).
    let known_topic_offsets = if queue_group.is_none() {
        snapshot_known_topic_offsets(
            &ctx.cache_manager,
            &ctx.storage_driver_manager,
            &tenant,
            subject,
        )
        .await
    } else {
        Default::default()
    };

    let subscribe = NatsSubscribe {
        broker_id: broker_config().broker_id,
        tenant: tenant.clone(),
        connect_id: ctx.connect_id,
        sid: sid.to_string(),
        subject: subject.to_string(),
        queue_group: queue_group.map(|s| s.to_string()),
        create_time: now_second(),
        known_topic_offsets,
    };

    // Make this subscribe visible in this node's own subscribe_list right
    // now, synchronously, *before* the raft write below — not after it, and
    // not by waiting for that write to round-trip back through the
    // UpdateCache broadcast (dynamic_cache.rs's `update_nats_cache_metadata`,
    // which normally does this). That broadcast is what makes the subscribe
    // visible to *other* nodes and is still needed for that, but relying on
    // it alone here (or even doing this after the raft write) leaves a
    // window where a publish to a brand-new subject on this same connection
    // — NATS has no SUB ack, so a client can fire SUB then PUB back-to-back
    // without waiting on the server at all — creates the topic and fires its
    // one-shot `parse_by_new_topic` match before subscribe_list has this
    // entry. That match finds nothing and nothing ever retries it: the
    // message is dropped for good, not just delayed. Doing this first, before
    // any `.await` point, closes that race for the common case (subscribe
    // and publish on the same node). `update_nats_cache_metadata` doing the
    // same insert again once the broadcast arrives is a harmless no-op.
    ctx.subscribe_manager.add_subscribe(subscribe.clone());

    // save subscribe
    let storage = NatsSubscribeStorage::new(ctx.client_pool.clone());
    storage
        .save(vec![subscribe.clone()])
        .await
        .map_err(|e| NatsPacket::Err(e.to_string()))?;

    if let Some(queue_name) = queue_group {
        // save queue name
        let conf = broker_config();
        let sub = ShareGroupMember {
            broker_id: conf.broker_id,
            tenant: tenant.clone(),
            group_name: queue_name.to_string(),
            sub_path: subject.to_string(),
            sid: sid.to_string(),
            params: ShareGroupParams::NATS(ShareGroupParamsNats {}),
            connect_id: ctx.connect_id,
            create_time: now_second(),
        };
        add_member_by_group(&ctx.client_pool, &sub)
            .await
            .map_err(|e| NatsPacket::Err(e.to_string()))?;
    }

    ctx.subscribe_manager
        .send_parse_event(ParseSubscribeData::new_subscribe(
            ParseAction::Add,
            SubscribeSource::NatsCore,
            subscribe,
        ))
        .await;

    Ok(())
}

pub async fn process_unsub(
    ctx: &NatsProcessContext,
    sid: &str,
    _max_msgs: Option<u32>,
) -> Result<(), NatsPacket> {
    if broker_config().nats_runtime.auth_required && !ctx.cache_manager.is_login(ctx.connect_id) {
        return Err(NatsPacket::Err(
            NatsProtocolError::AuthorizationViolation.message(),
        ));
    }

    if let Some(subscribe) = ctx.subscribe_manager.get_subscribe(ctx.connect_id, sid) {
        let conf = broker_config();
        if subscribe.queue_group.is_some() {
            delete_member_by_group(&ctx.client_pool, conf.broker_id, ctx.connect_id, sid)
                .await
                .map_err(|e| NatsPacket::Err(e.to_string()))?;
        } else {
            ctx.subscribe_manager
                .send_parse_event(ParseSubscribeData::new_subscribe(
                    ParseAction::Remove,
                    SubscribeSource::NatsCore,
                    subscribe,
                ))
                .await;
        }
    }

    ctx.subscribe_manager.remove_subscribe(ctx.connect_id, sid);
    ctx.cache_manager.remove_inbox_by_sid(sid);
    Ok(())
}
