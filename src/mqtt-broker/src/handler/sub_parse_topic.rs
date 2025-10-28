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

use super::{
    cache::MQTTCacheManager,
    subscribe::{parse_subscribe, ParseSubscribeContext},
};
use crate::subscribe::manager::SubscribeManager;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use std::{sync::Arc, time::Duration};
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info};

pub async fn start_parse_subscribe_by_new_topic_thread(
    client_pool: &Arc<ClientPool>,
    metadata_cache: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    stop_send: broadcast::Sender<bool>,
) {
    info!("Subscribe manager thread started successfully.");
    let mut last_update_time: u64 = 0;
    loop {
        let mut stop_rx = stop_send.subscribe();
        select! {
            val = stop_rx.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        info!("{}","Subscribe manager thread stopped successfully.");
                        break;
                    }
                }
            }
            _ = parse_subscribe_by_new_topic(
                client_pool,
                metadata_cache,
                subscribe_manager,
                last_update_time) =>{
                    last_update_time = now_second();
                    sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

async fn parse_subscribe_by_new_topic(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<MQTTCacheManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    last_update_time: u64,
) {
    let conf = broker_config();

    for (_, subscribe) in subscribe_manager.list_subscribe() {
        if subscribe.broker_id != conf.broker_id {
            continue;
        }
        let rewrite_sub_path = cache_manager.get_new_rewrite_name(&subscribe.path);
        for (_, topic) in cache_manager.topic_info.clone() {
            if topic.create_time < last_update_time {
                continue;
            }
            if let Err(e) = parse_subscribe(ParseSubscribeContext {
                client_pool: client_pool.clone(),
                subscribe_manager: subscribe_manager.clone(),
                client_id: subscribe.client_id.clone(),
                topic: topic.clone(),
                protocol: subscribe.protocol.clone(),
                pkid: subscribe.pkid,
                filter: subscribe.filter.clone(),
                subscribe_properties: subscribe.subscribe_properties.clone(),
                rewrite_sub_path: rewrite_sub_path.clone(),
            })
            .await
            {
                error!("Failed to parse subscribe, error message: {}", e);
            }
        }
    }
}
