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
use grpc_clients::pool::ClientPool;
use metadata_struct::mq9::email::MQ9Email;
use metadata_struct::nats::subscribe::NatsSubscribe;
use protocol::broker::broker::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use std::sync::Arc;

pub async fn update_nats_cache_metadata(
    cache_manager: &Arc<NatsCacheManager>,
    subscribe_manager: &Arc<NatsSubscribeManager>,
    client_pool: &Arc<ClientPool>,
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
                    cache_manager.add_mail(email);
                }
                BrokerUpdateCacheActionType::Delete => {
                    cache_manager.remove_mail(&email.tenant, &email.mail_id);
                    // mail id is globally unique. Once the metadata of a mail is deleted,
                    // there will be no duplicate mail ids in the future.
                    // That is to say, data will no longer be written to this mail and will not be consumed.
                    // At this point, the underlying data will naturally expire.
                }
            }
        }

        _ => {}
    }
    Ok(())
}
