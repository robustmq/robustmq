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

use crate::mqtt::cache::MqttCacheManager;
use log::warn;
use protocol::placement_center::placement_center_mqtt::{
    ConnectorHeartbeatReply, ConnectorHeartbeatRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
pub struct ConnectorHeartbeat {
    pub cluster_name: String,
    pub connector_name: String,
    pub last_heartbeat: u64,
}

pub fn connector_heartbeat_by_req(
    mqtt_cache: &Arc<MqttCacheManager>,
    request: Request<ConnectorHeartbeatRequest>,
) -> Result<Response<ConnectorHeartbeatReply>, Status> {
    let req = request.into_inner();
    for raw in req.heatbeats {
        if let Some(connector) = mqtt_cache.get_connector(&req.cluster_name, &raw.connector_name) {
            if connector.broker_id.is_none() {
                warn!("connector:{} not register", raw.connector_name);
                continue;
            }

            if connector.broker_id.unwrap() != raw.broker_id {
                warn!("connector:{} not register", raw.connector_name);
                continue;
            }

            mqtt_cache.report_connector_heartbeat(
                &req.cluster_name,
                &raw.connector_name,
                raw.heartbeat_time,
            );
        }
    }
    Ok(Response::new(ConnectorHeartbeatReply::default()))
}
