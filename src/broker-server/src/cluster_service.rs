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

use broker_core::cache::BrokerCacheManager;
use common_base::tools::now_second;
use protocol::cluster::cluster_status::{
    cluster_service_server::ClusterService, ClusterStatusReply, ClusterStatusRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct ClusterInnerService {
    cache_manager: Arc<BrokerCacheManager>,
}

impl ClusterInnerService {
    pub fn new(cache_manager: Arc<BrokerCacheManager>) -> Self {
        ClusterInnerService { cache_manager }
    }
}

#[tonic::async_trait]
impl ClusterService for ClusterInnerService {
    async fn cluster_status(
        &self,
        _request: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        let node_list = self.cache_manager.node_list();
        let uptime = now_second() - self.cache_manager.get_start_time();

        let reply = ClusterStatusReply {
            node_count: node_list.len() as u32,
            uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
            active_nodes: node_list.iter().map(|n| n.node_id).collect(),
        };

        Ok(Response::new(reply))
    }
}
