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

use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use grpc_clients::{meta::mqtt::call::placement_get_share_sub_leader, pool::ClientPool};
use protocol::meta::meta_service_mqtt::{GetShareSubLeaderReply, GetShareSubLeaderRequest};
use std::sync::Arc;

pub const SHARE_SUB_PREFIX: &str = "$share";

pub fn is_mqtt_share_subscribe(sub_name: &str) -> bool {
    sub_name.starts_with(SHARE_SUB_PREFIX)
}

pub fn decode_share_info(sub_path: &str) -> (String, String) {
    let mut str_slice: Vec<&str> = sub_path.split("/").collect();
    str_slice.remove(0);
    let group_name = str_slice.remove(0).to_string();
    let sub_path = format!("/{}", str_slice.join("/"));
    (group_name, sub_path)
}

pub async fn is_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: &String,
) -> Result<bool, CommonError> {
    let conf = broker_config();
    let req = GetShareSubLeaderRequest {
        cluster_name: conf.cluster_name.to_owned(),
        group_name: group_name.to_owned(),
    };
    let reply =
        placement_get_share_sub_leader(client_pool, &conf.get_meta_service_addr(), req).await?;
    Ok(reply.broker_id == conf.broker_id)
}

pub async fn get_share_sub_leader(
    client_pool: &Arc<ClientPool>,
    group_name: &String,
) -> Result<GetShareSubLeaderReply, CommonError> {
    let conf = broker_config();
    let req = GetShareSubLeaderRequest {
        cluster_name: conf.cluster_name.to_owned(),
        group_name: group_name.to_owned(),
    };
    placement_get_share_sub_leader(client_pool, &conf.get_meta_service_addr(), req).await
}
