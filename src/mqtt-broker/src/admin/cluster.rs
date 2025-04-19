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

use crate::handler::cache::CacheManager;
use crate::handler::offline_message::enable_offline_message;
use crate::observability::slow::sub::enable_slow_sub;
use common_base::enum_type::feature_type::FeatureType;
use protocol::broker_mqtt::broker_mqtt_admin::{SetClusterConfigReply, SetClusterConfigRequest};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn set_cluster_config_by_req(
    cache_manager: &Arc<CacheManager>,
    request: Request<SetClusterConfigRequest>,
) -> Result<Response<SetClusterConfigReply>, Status> {
    let cluster_config_request = request.into_inner();
    match FeatureType::from_str(cluster_config_request.feature_name.as_str()) {
        Ok(FeatureType::SlowSubscribe) => {
            match enable_slow_sub(cache_manager, cluster_config_request.is_enable).await {
                Ok(_) => Ok(Response::new(SetClusterConfigReply {
                    feature_name: cluster_config_request.feature_name,
                    is_enable: cluster_config_request.is_enable,
                })),
                Err(e) => Err(Status::cancelled(e.to_string())),
            }
        }
        Ok(FeatureType::OfflineMessage) => {
            match enable_offline_message(cache_manager, cluster_config_request.is_enable).await {
                Ok(_) => Ok(Response::new(SetClusterConfigReply {
                    feature_name: cluster_config_request.feature_name,
                    is_enable: cluster_config_request.is_enable,
                })),
                Err(e) => Err(Status::cancelled(e.to_string())),
            }
        }
        Err(e) => Err(Status::invalid_argument(format!("Invalid feature : {}", e))),
    }
}
