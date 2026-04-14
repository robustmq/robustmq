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
use crate::push::parse::{ParseAction, ParseSubscribeData, SubscribeSource};
use common_base::tools::now_second;
use metadata_struct::mq9::Priority;
use metadata_struct::nats::subscribe::NatsSubscribe;
use metadata_struct::tenant::DEFAULT_TENANT;

pub async fn process_sub(
    ctx: &NatsProcessContext,
    mail_id: &str,
    priority: Option<&Priority>,
    sid: &str,
    queue_group: Option<&str>,
) -> Result<(), NatsBrokerError> {
    // subject stored is the full subscription pattern, e.g. `mail-id.*` or `mail-id.high`
    let subject = match priority {
        Some(p) => format!("{}.{}", mail_id, p),
        None => format!("{}.*", mail_id),
    };

    let tenant = DEFAULT_TENANT.to_string();
    if ctx.cache_manager.get_email(&tenant, mail_id).is_none() {
        return Err(NatsBrokerError::CommonError(format!(
            "mailbox {} does not exist",
            mail_id
        )));
    }

    let subscribe = NatsSubscribe {
        tenant: tenant.clone(),
        connect_id: ctx.connect_id,
        sid: sid.to_string(),
        subject,
        queue_group: queue_group.unwrap_or_default().to_string(),
        priority: priority.cloned(),
        create_time: now_second(),
    };

    ctx.subscribe_manager.add_subscribe(subscribe.clone());
    ctx.subscribe_manager
        .send_parse_event(ParseSubscribeData::new_subscribe(
            ParseAction::Add,
            SubscribeSource::Mq9,
            subscribe,
        ))
        .await;

    Ok(())
}

pub async fn process_unsub(
    ctx: &NatsProcessContext,
    _mail_id: &str,
    sid: &str,
) -> Result<(), NatsBrokerError> {
    if let Some(subscribe) = ctx.subscribe_manager.get_subscribe(ctx.connect_id, sid) {
        ctx.subscribe_manager
            .send_parse_event(ParseSubscribeData::new_subscribe(
                ParseAction::Remove,
                SubscribeSource::Mq9,
                subscribe,
            ))
            .await;
    }

    ctx.subscribe_manager.remove_subscribe(ctx.connect_id, sid);
    Ok(())
}
