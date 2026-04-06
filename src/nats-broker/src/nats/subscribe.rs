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
use protocol::nats::packet::NatsPacket;

use crate::core::error::NatsProtocolError;
use crate::handler::command::NatsProcessContext;
use crate::nats_push::parse::{ParseAction, ParseSubscribeData};

pub fn subject_message_tag(tenant: &str, subject: &str) -> String {
    format!("{}_{}", tenant, subject)
}

pub fn process_sub(
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

    let subscribe = NatsSubscribe {
        tenant: DEFAULT_TENANT.to_string(),
        connect_id: ctx.connect_id,
        sid: sid.to_string(),
        subject: subject.to_string(),
        queue_group: queue_group.unwrap_or_default().to_string(),
        create_time: now_second(),
    };

    ctx.subscribe_manager.add_subscribe(subscribe.clone());

    let data = ParseSubscribeData::new_subscribe(ParseAction::Add, subscribe);
    let subscribe_manager = ctx.subscribe_manager.clone();
    tokio::spawn(async move {
        subscribe_manager.send_parse_event(data).await;
    });

    Ok(())
}

pub fn process_unsub(
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
        let data = ParseSubscribeData::new_subscribe(ParseAction::Remove, subscribe);
        let subscribe_manager = ctx.subscribe_manager.clone();
        tokio::spawn(async move {
            subscribe_manager.send_parse_event(data).await;
        });
    }

    ctx.subscribe_manager.remove_subscribe(ctx.connect_id, sid);
    Ok(())
}
