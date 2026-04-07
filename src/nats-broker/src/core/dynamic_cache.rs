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

use crate::core::cache::NatsCacheManager;
use crate::push::manager::NatsSubscribeManager;
use crate::push::parse::{ParseAction, ParseSubscribeData, SubscribeSource};
use common_base::error::common::CommonError;
use common_base::utils::serialize;
use metadata_struct::mq9::email::MQ9Email;
use metadata_struct::nats::subscribe::NatsSubscribe;
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use std::sync::Arc;

pub async fn update_nats_cache_metadata(
    cache_manager: &Arc<NatsCacheManager>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    record: &UpdateCacheRecord,
) -> Result<(), CommonError> {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::NatsSubscribe => {
            let subscribe: NatsSubscribe = serialize::deserialize(&record.data)?;
            let data = match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    subscribe_manager.add_subscribe(subscribe.clone());
                    ParseSubscribeData::new_subscribe(
                        ParseAction::Add,
                        SubscribeSource::NatsCore,
                        subscribe,
                    )
                }
                BrokerUpdateCacheActionType::Delete => {
                    subscribe_manager.remove_subscribe(subscribe.connect_id, &subscribe.sid);
                    subscribe_manager.remove_push_by_sid(subscribe.connect_id, &subscribe.sid);
                    ParseSubscribeData::new_subscribe(
                        ParseAction::Remove,
                        SubscribeSource::NatsCore,
                        subscribe,
                    )
                }
            };
            subscribe_manager.send_parse_event(data).await;
        }

        BrokerUpdateCacheResourceType::Mq9Email => {
            let email: MQ9Email = serialize::deserialize(&record.data)?;
            match record.action_type() {
                BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                    cache_manager.add_email(email);
                }
                BrokerUpdateCacheActionType::Delete => {
                    cache_manager.remove_email(&email.tenant, &email.mail_id);
                }
            }
        }

        _ => {}
    }
    Ok(())
}
