#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::Arc,
        thread::{self, sleep},
        time::Duration,
    };

    use clients::{
        placement::{mqtt::call::placement_get_share_sub, placement::call::register_node},
        poll::ClientPool,
    };
    use common_base::{
        config::placement_center::{init_placement_center_conf_by_config, PlacementCenterConfig},
        log::{error, info, init_placement_center_log},
    };
    use placement_center::PlacementCenter;
    use protocol::placement_center::generate::{
        common::ClusterType, mqtt::GetShareSubRequest, placement::RegisterNodeRequest,
    };
    use tokio::sync::broadcast::{self, Sender};
    use toml::map::Map;

    #[tokio::test]
    async fn test_share_sub() {
        let (stop_send, _) = broadcast::channel::<bool>(2);
        let cc = conf();
        start_placement_center(cc, stop_send.clone());

        let client_poll = Arc::new(ClientPool::new(3));
        let mut addrs = Vec::new();
        addrs.push("127.0.0.1:5228".to_string());

        let cluster_type = ClusterType::MqttBrokerServer.try_into().unwrap();
        let cluster_name = "test-cluster".to_string();
        let node_ip = "127.0.0.1".to_string();
        let node_id = 3;
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
                info(format!("{:?}", res));
            }
            Err(e) => {
                error(e.to_string());
                assert!(false);
            }
        }

        let group_name = "test".to_string();
        let sub_name = "test-group".to_string();
        let req = GetShareSubRequest {
            cluster_name: cluster_name.clone(),
            group_name,
            sub_name,
        };

        match placement_get_share_sub(client_poll.clone(), addrs.clone(), req).await {
            Ok(rep) => {
                assert_eq!(rep.broker_id, 3);
            }
            Err(e) => {
                error(e.to_string());
                assert!(false);
            }
        }

        sleep(Duration::from_secs(2));
        clean(stop_send.clone());
    }

    fn start_placement_center(conf: PlacementCenterConfig, stop_send: Sender<bool>) {
        thread::spawn(move || {
            init_placement_center_conf_by_config(conf);
            init_placement_center_log();
            let mut pc = PlacementCenter::new();
            pc.start(stop_send);
        });
    }

    fn conf() -> PlacementCenterConfig {
        let mut conf = PlacementCenterConfig::default();
        conf.cluster_name = "placement-test".to_string();
        conf.node_id = 1;
        conf.addr = "127.0.0.1".to_string();
        conf.grpc_port = 5228;
        conf.http_port = 5227;

        let mut nodes = Map::new();
        nodes.insert(
            "1".to_string(),
            toml::Value::String("127.0.0.1:5228".to_string()),
        );
        conf.nodes = nodes;
        conf.data_path = "/tmp/robust-test/data".to_string();
        conf.log_path = "/tmp/robust-test/log".to_string();
        return conf;
    }

    fn clean(stop_send: Sender<bool>) {
        let conf = conf();
        fs::remove_dir_all(conf.log_path).unwrap();
        fs::remove_dir_all(conf.data_path).unwrap();
        stop_send.send(true).unwrap();
    }
}
