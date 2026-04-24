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
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::tenant::DEFAULT_TENANT;
use mq9_core::command::Mq9Command;
use protocol::nats::packet::NatsPacket;

use crate::core::error::NatsProtocolError;
use crate::core::subject::is_inbox_subject;
use crate::handler::command::NatsProcessContext;
use crate::mq9::subscribe as mq9_subscribe;
use crate::push::parse::{ParseAction, ParseSubscribeData, SubscribeSource};

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
        // Extract mail_id: subject is `$mq9.AI.MAILBOX.MSG.{mail_id}`
        let mail_id = subject
            .strip_prefix("$mq9.AI.MAILBOX.MSG.")
            .unwrap_or(subject);
        return mq9_subscribe::process_sub(ctx, mail_id, sid, queue_group)
            .await
            .map_err(|e| NatsPacket::Err(e.to_string()));
    }

    let subscribe = NatsSubscribe {
        broker_id: broker_config().broker_id,
        tenant: DEFAULT_TENANT.to_string(),
        connect_id: ctx.connect_id,
        sid: sid.to_string(),
        subject: subject.to_string(),
        queue_group: queue_group.map(|s| s.to_string()),
        create_time: now_second(),
    };

    ctx.subscribe_manager.add_subscribe(subscribe.clone());

    if queue_group.is_some() {
        // save queue group
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

    // inbox: no subscribe_manager entry, remove directly by sid.
    ctx.cache_manager.remove_inbox_by_sid(sid);

    if let Some(subscribe) = ctx.subscribe_manager.get_subscribe(ctx.connect_id, sid) {
        if Mq9Command::is_mq9_subject(&subscribe.subject) {
            ctx.subscribe_manager.remove_subscribe(ctx.connect_id, sid);
            return mq9_subscribe::process_unsub(ctx, &subscribe.subject, sid)
                .await
                .map_err(|e| NatsPacket::Err(e.to_string()));
        }
        ctx.subscribe_manager
            .send_parse_event(ParseSubscribeData::new_subscribe(
                ParseAction::Remove,
                SubscribeSource::NatsCore,
                subscribe,
            ))
            .await;
    }

    ctx.subscribe_manager.remove_subscribe(ctx.connect_id, sid);
    Ok(())
}
