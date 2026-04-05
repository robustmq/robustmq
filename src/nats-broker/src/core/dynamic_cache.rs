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

use crate::subscribe::parse::{ParseAction, ParseSubscribeData};
use crate::subscribe::NatsSubscribeManager;
use common_base::error::common::CommonError;
use common_base::utils::serialize;
use metadata_struct::nats::subscribe::NatsSubscribe;
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use std::sync::Arc;

pub async fn update_nats_cache_metadata(
    subscribe_manager: &Arc<NatsSubscribeManager>,
    record: &UpdateCacheRecord,
) -> Result<(), CommonError> {
    if record.resource_type() == BrokerUpdateCacheResourceType::NatsSubscribe {
        let subscribe: NatsSubscribe = serialize::deserialize(&record.data)?;
        let data = match record.action_type() {
            BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
                subscribe_manager.add_subscribe(subscribe.clone());
                ParseSubscribeData::new_subscribe(ParseAction::Add, subscribe)
            }
            BrokerUpdateCacheActionType::Delete => {
                subscribe_manager.remove_subscribe(subscribe.connect_id, &subscribe.sid);
                subscribe_manager.remove_push_by_sid(subscribe.connect_id, &subscribe.sid);
                ParseSubscribeData::new_subscribe(ParseAction::Remove, subscribe)
            }
        };
        subscribe_manager.send_parse_event(data).await;
    }
    Ok(())
}
