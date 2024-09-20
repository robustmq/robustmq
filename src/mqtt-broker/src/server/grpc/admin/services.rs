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

use crate::{
    handler::cache::CacheManager, storage::cluster::ClusterStorage,
    subscribe::subscribe_manager::SubscribeManager,
};
use clients::poll::ClientPool;
use common_base::config::{broker_mqtt::broker_mqtt_conf, placement_center::placement_center_conf};
use protocol::broker_server::generate::admin::{
    mqtt_broker_admin_service_server::MqttBrokerAdminService, ClusterStatusReply,
    ClusterStatusRequest,
};
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;
use tonic::{Request, Response, Status};

pub struct GrpcAdminServices<S> {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    client_poll: Arc<ClientPool>,
    message_storage_adapter: Arc<S>,
}

impl<S> GrpcAdminServices<S> {
    pub fn new(
        metadata_cache: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        client_poll: Arc<ClientPool>,
        message_storage_adapter: Arc<S>,
    ) -> Self {
        return GrpcAdminServices {
            cache_manager: metadata_cache,
            subscribe_manager,
            client_poll,
            message_storage_adapter,
        };
    }
}

#[tonic::async_trait]
impl<S> MqttBrokerAdminService for GrpcAdminServices<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        let mut reply = ClusterStatusReply::default();
        let config = broker_mqtt_conf();
        reply.cluster_name = config.cluster_name.clone();
        let mut broker_node_list = Vec::new();
        let cluster_storage = ClusterStorage::new(self.client_poll.clone());
        match cluster_storage.node_list().await {
            Ok(data) => {
                for node in data {
                    broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
                }
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
        reply.nodes = broker_node_list;
        return Ok(Response::new(reply));
    }
}
