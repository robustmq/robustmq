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

use std::sync::Arc;

use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::ClusterStatusReply;

use crate::{handler::error::MqttBrokerError, storage::cluster::ClusterStorage};

pub async fn cluster_status_by_req(
    client_pool: &Arc<ClientPool>,
) -> Result<ClusterStatusReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

    let mut broker_node_list = Vec::new();
    let cluster_storage = ClusterStorage::new(client_pool.clone());
    let data = cluster_storage.node_list().await?;
    for node in data {
        broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
    }
    Ok(ClusterStatusReply {
        nodes: broker_node_list,
        cluster_name: config.cluster_name.clone(),
    })
}
