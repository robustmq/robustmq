// Copyright 2023 RobustMQ Team
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
    use common_base::{
        config::placement_center::{init_placement_center_conf_by_config, PlacementCenterConfig},
        logs::init_placement_center_log,
    };
    use log::{error, info};
    use placement_center::PlacementCenter;
    use protocol::placement_center::generate::{
        common::ClusterType, mqtt::GetShareSubLeaderRequest, placement::RegisterNodeRequest,
    };
    use std::{
        fs,
        thread::{self},
    };
    use tokio::sync::broadcast::Sender;
    use tokio::sync::broadcast::{self};
    use toml::map::Map;

    #[tokio::test]
    #[ignore]
    async fn test_share_sub() {
        let (stop_send, _) = broadcast::channel::<bool>(2);
        let cc = test_conf();
        test_start_placement_center(cc, stop_send.clone());

        let client_poll = Arc::new(ClientPool::new(3));
        let mut addrs = Vec::new();
        addrs.push("127.0.0.1:5228".to_string());

        let cluster_type = ClusterType::MqttBrokerServer.try_into().unwrap();
        let cluster_name = "test-cluster".to_string();
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
                assert_eq!(rep.broker_id, node_id);
            }
            Err(e) => {
                error!("{}", e);
                assert!(false);
            }
        }

        sleep(Duration::from_secs(2));
        test_clean(stop_send.clone());
    }
    pub fn test_start_placement_center(conf: PlacementCenterConfig, stop_send: Sender<bool>) {
        thread::spawn(move || {
            init_placement_center_conf_by_config(conf);
            init_placement_center_log();
            let mut pc = PlacementCenter::new();
            pc.start(stop_send);
        });
    }

    pub fn test_conf() -> PlacementCenterConfig {
        let mut conf = PlacementCenterConfig::default();
        conf.cluster_name = "placement-test".to_string();
        conf.node_id = 1;
        conf.addr = "127.0.0.1".to_string();
        conf.grpc_port = 5228;
        conf.http_port = 5227;
        conf.runtime_work_threads = 8;
        conf.rocksdb.max_open_files = Some(100);
        conf.log.log_config = "../../config/log4rs.yaml".to_string();

        let mut nodes = Map::new();
        nodes.insert(
            "1".to_string(),
            toml::Value::String("127.0.0.1:5228".to_string()),
        );
        conf.nodes = nodes;
        conf.data_path = "/tmp/robust-test/data".to_string();
        conf.log.log_path = "/tmp/robust-test/log".to_string();
        return conf;
    }

    pub fn test_clean(stop_send: Sender<bool>) {
        let conf = test_conf();
        fs::remove_dir_all(conf.log.log_path).unwrap();
        fs::remove_dir_all(conf.data_path).unwrap();
        stop_send.send(true).unwrap();
    }
}
