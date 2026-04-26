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
use crate::core::queue_name::{add_member_by_group, delete_member_by_group};
use crate::handler::command::NatsProcessContext;
use crate::push::parse::{ParseAction, ParseSubscribeData, SubscribeSource};
use common_base::tools::now_second;
use common_config::broker::broker_config;
use metadata_struct::mqtt::share_group::{
    ShareGroupMember, ShareGroupParams, ShareGroupParamsNats,
};
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::tenant::DEFAULT_TENANT;

pub async fn process_sub(
    ctx: &NatsProcessContext,
    mail_address: &str,
    sid: &str,
    queue_group: Option<&str>,
) -> Result<(), NatsBrokerError> {
    let tenant = DEFAULT_TENANT.to_string();
    if ctx.cache_manager.get_mail(&tenant, mail_address).is_none() {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} does not exist",
            mail_address
        )));
    }

    let subscribe = NatsSubscribe {
        broker_id: broker_config().broker_id,
        tenant: tenant.clone(),
        connect_id: ctx.connect_id,
        sid: sid.to_string(),
        subject: mail_address.to_string(),
        queue_group: queue_group.map(|s| s.to_string()),
        create_time: now_second(),
    };

    ctx.subscribe_manager.add_subscribe(subscribe.clone());

    if let Some(queue_name) = queue_group {
        let conf = broker_config();
        let sub = ShareGroupMember {
            broker_id: conf.broker_id,
            tenant: tenant.clone(),
            group_name: queue_name.to_string(),
            sub_path: mail_address.to_string(),
            sid: sid.to_string(),
            params: ShareGroupParams::MQ9(ShareGroupParamsNats {}),
            connect_id: ctx.connect_id,
            create_time: now_second(),
        };
        add_member_by_group(&ctx.client_pool, &sub).await?;
    } else {
        ctx.subscribe_manager
            .send_parse_event(ParseSubscribeData::new_subscribe(
                ParseAction::Add,
                SubscribeSource::Mq9,
                subscribe,
            ))
            .await;
    }

    Ok(())
}

pub async fn process_unsub(
    ctx: &NatsProcessContext,
    _mail_address: &str,
    sid: &str,
) -> Result<(), NatsBrokerError> {
    if let Some(subscribe) = ctx.subscribe_manager.get_subscribe(ctx.connect_id, sid) {
        let conf = broker_config();
        if subscribe.queue_group.is_some() {
            delete_member_by_group(&ctx.client_pool, conf.broker_id, ctx.connect_id, sid).await?;
        } else {
            ctx.subscribe_manager
                .send_parse_event(ParseSubscribeData::new_subscribe(
                    ParseAction::Remove,
                    SubscribeSource::Mq9,
                    subscribe,
                ))
                .await;
        }
    }

    ctx.subscribe_manager.remove_subscribe(ctx.connect_id, sid);
    ctx.cache_manager.remove_inbox_by_sid(sid);

    Ok(())
}
