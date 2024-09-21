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

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread::sleep, time::Duration};
    use clients::{
        placement::{mqtt::call::placement_get_share_sub_leader, placement::call::register_node},
        poll::ClientPool,
    };
    use common_base::tools::unique_id;
    use log::{error, info};
    use protocol::placement_center::generate::{
        common::ClusterType, mqtt::GetShareSubLeaderRequest, placement::RegisterNodeRequest,
    };

    #[tokio::test]
    async fn test_share_sub() {
        let client_poll = Arc::new(ClientPool::new(3));
        let mut addrs = Vec::new();
        addrs.push("127.0.0.1:1228".to_string());

        let cluster_type = ClusterType::MqttBrokerServer.try_into().unwrap();
        let cluster_name = unique_id();
        let node_ip = "127.0.0.1".to_string();
        let node_id = 7;
        let node_inner_addr = "127.0.0.1:8228".to_string();
        let extend_info = "".to_string();
        let request = RegisterNodeRequest {
            cluster_type,
            cluster_name: cluster_name.clone(),
            node_ip,
            node_id,
            node_inner_addr,
            extend_info,
        };

        sleep(Duration::from_secs(2));

        match register_node(client_poll.clone(), addrs.clone(), request).await {
            Ok(res) => {
                info!("{:?}", res);
            }
            Err(e) => {
                error!("{}", e);
                assert!(false);
            }
        }

        let group_name = "test".to_string();
        let req = GetShareSubLeaderRequest {
            cluster_name: cluster_name.clone(),
            group_name,
        };

        match placement_get_share_sub_leader(client_poll.clone(), addrs.clone(), req).await {
            Ok(rep) => {
                println!("rep broker_id:{}", rep.broker_id);
                println!("node_id:{}", node_id);
                assert_eq!(rep.broker_id, node_id);
            }
            Err(e) => {
                error!("{}", e);
                assert!(false);
            }
        }
    }
}
