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
use mq9_core::command::Mq9Command;
use protocol::nats::packet::NatsPacket;

use crate::core::error::NatsProtocolError;
use crate::core::queue_name::{add_member_by_group, delete_member_by_group};
use crate::core::subject::is_inbox_subject;
use crate::handler::command::NatsProcessContext;
use crate::mq9::subscribe as mq9_subscribe;
use crate::push::parse::{ParseAction, ParseSubscribeData, SubscribeSource};
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

    if Mq9Command::is_mq9_subject(subject) {
        // Extract mail_address: subject is `$mq9.AI.MAILBOX.MSG.{mail_address}`
        let mail_address = subject
            .strip_prefix("$mq9.AI.MAILBOX.MSG.")
            .unwrap_or(subject);
        return mq9_subscribe::process_sub(ctx, mail_address, sid, queue_group)
            .await
            .map_err(|e| NatsPacket::Err(e.to_string()));
    }

    let tenant = DEFAULT_TENANT.to_string();
    let subscribe = NatsSubscribe {
        broker_id: broker_config().broker_id,
        tenant: tenant.clone(),
        connect_id: ctx.connect_id,
        sid: sid.to_string(),
        subject: subject.to_string(),
        queue_group: queue_group.map(|s| s.to_string()),
        create_time: now_second(),
    };

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
    } else {
        ctx.subscribe_manager
            .send_parse_event(ParseSubscribeData::new_subscribe(
                ParseAction::Add,
                SubscribeSource::NatsCore,
                subscribe,
            ))
            .await;
    }

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
        if Mq9Command::is_mq9_subject(&subscribe.subject) {
            return mq9_subscribe::process_unsub(ctx, &subscribe.subject, sid)
                .await
                .map_err(|e| NatsPacket::Err(e.to_string()));
        }

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
